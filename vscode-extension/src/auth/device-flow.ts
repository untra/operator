/**
 * OAuth device authorization, client half.
 *
 * The server side (`/api/v1/auth/device/code`, `/token`) is complete; the
 * extension requests a code, sends the human to the daemon's approval page,
 * and polls until the code is approved, denied, expired, or cancelled.
 */

import type {
  DeviceAuthorizationRequest,
  DeviceAuthorizationResponse,
  OAuthErrorResponse,
  Scope,
  TokenRequest,
  TokenResponse,
} from '../generated';
import { CLIENT_ID, TOKEN_PATH, secondsToMs } from './credentials';
import type { TokenStore } from './token-store';

export const DEVICE_CODE_PATH = '/api/v1/auth/device/code';
/** RFC 8628 §3.5: on `slow_down` the client adds 5 seconds to its interval. */
export const SLOW_DOWN_INCREMENT_SECS = 5;
/** IDE clients act as the human admin, so they request every scope. */
export const IDE_SCOPES: Scope[] = ['read', 'write', 'execute', 'admin'];

export type DeviceFlowOutcome =
  | { status: 'approved'; scopes: Scope[] }
  | { status: 'denied' }
  | { status: 'expired' }
  | { status: 'cancelled' }
  | { status: 'error'; message: string };

export interface DeviceFlowHooks {
  /** Called once with the issued code so the UI can show it and open the browser. */
  onCode(authorization: DeviceAuthorizationResponse): Promise<void>;
  isCancelled(): boolean;
  /** Injected so tests can run the poll loop without real time passing. */
  sleep(ms: number): Promise<void>;
  now(): number;
}

async function readError(response: Response): Promise<Partial<OAuthErrorResponse>> {
  return (await response.json().catch(() => ({}))) as Partial<OAuthErrorResponse>;
}

export async function requestDeviceCode(apiUrl: string): Promise<DeviceAuthorizationResponse> {
  const body: DeviceAuthorizationRequest = { client_id: CLIENT_ID, scopes: IDE_SCOPES };
  const response = await fetch(`${apiUrl}${DEVICE_CODE_PATH}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    const error = await readError(response);
    throw new Error(
      error.error_description ?? error.error ?? `HTTP ${response.status}: ${response.statusText}`
    );
  }
  return (await response.json()) as DeviceAuthorizationResponse;
}

/**
 * Poll the token endpoint until the device code resolves.
 *
 * The advertised `interval` is a floor the server enforces, so the first poll
 * also waits for it rather than firing immediately.
 */
export async function pollForToken(
  apiUrl: string,
  authorization: DeviceAuthorizationResponse,
  store: TokenStore,
  hooks: Pick<DeviceFlowHooks, 'isCancelled' | 'sleep' | 'now'>
): Promise<DeviceFlowOutcome> {
  let intervalSecs = Number(authorization.interval);
  const deadline = hooks.now() + secondsToMs(authorization.expires_in);
  const body: TokenRequest = {
    grant_type: 'urn:ietf:params:oauth:grant-type:device_code',
    device_code: authorization.device_code,
    client_id: CLIENT_ID,
  };

  for (;;) {
    if (hooks.isCancelled()) {
      return { status: 'cancelled' };
    }
    if (hooks.now() >= deadline) {
      return { status: 'expired' };
    }
    await hooks.sleep(secondsToMs(intervalSecs));
    if (hooks.isCancelled()) {
      return { status: 'cancelled' };
    }

    let response: Response;
    try {
      response = await fetch(`${apiUrl}${TOKEN_PATH}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
    } catch (err) {
      return { status: 'error', message: err instanceof Error ? err.message : String(err) };
    }

    if (response.ok) {
      const token = (await response.json()) as TokenResponse;
      if (!token.refresh_token) {
        return { status: 'error', message: 'token response did not include a refresh token' };
      }
      await store.save(apiUrl, {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        expires_at: hooks.now() + secondsToMs(token.expires_in),
        scopes: token.scopes,
      });
      return { status: 'approved', scopes: token.scopes };
    }

    const error = await readError(response);
    switch (error.error) {
      case 'authorization_pending':
        continue;
      case 'slow_down':
        intervalSecs += SLOW_DOWN_INCREMENT_SECS;
        continue;
      case 'access_denied':
        return { status: 'denied' };
      case 'expired_token':
        return { status: 'expired' };
      default:
        return {
          status: 'error',
          message: error.error_description ?? error.error ?? `HTTP ${response.status}`,
        };
    }
  }
}

/** Request a code, hand it to the UI, and poll to completion. */
export async function runDeviceFlow(
  apiUrl: string,
  store: TokenStore,
  hooks: DeviceFlowHooks
): Promise<DeviceFlowOutcome> {
  let authorization: DeviceAuthorizationResponse;
  try {
    authorization = await requestDeviceCode(apiUrl);
  } catch (err) {
    return { status: 'error', message: err instanceof Error ? err.message : String(err) };
  }
  await hooks.onCode(authorization);
  return pollForToken(apiUrl, authorization, store, hooks);
}
