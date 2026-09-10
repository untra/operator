/**
 * Device-flow credentials, kept only in VS Code's SecretStorage.
 *
 * Settings sync across machines and workspace files land in Git; neither is an
 * acceptable home for a refresh token, so nothing here touches `globalState`
 * or configuration.
 */

import type * as vscode from 'vscode';
import type { Scope } from '../generated/Scope';

const KEY_PREFIX = 'operator.auth.';

export interface StoredCredential {
  access_token: string;
  refresh_token: string;
  /** Epoch milliseconds at which `access_token` stops being usable. */
  expires_at: number;
  scopes: Scope[];
}

/** Storage key for one daemon, so several daemons can be signed in at once. */
export function credentialKey(apiUrl: string): string {
  return `${KEY_PREFIX}${apiUrl.replace(/\/+$/, '')}`;
}

export class TokenStore {
  constructor(private readonly secrets: vscode.SecretStorage) {}

  async load(apiUrl: string): Promise<StoredCredential | undefined> {
    const raw = await this.secrets.get(credentialKey(apiUrl));
    if (!raw) {
      return undefined;
    }
    try {
      return JSON.parse(raw) as StoredCredential;
    } catch {
      // Unparseable state is treated as absent rather than trusted.
      await this.secrets.delete(credentialKey(apiUrl));
      return undefined;
    }
  }

  async save(apiUrl: string, credential: StoredCredential): Promise<void> {
    await this.secrets.store(credentialKey(apiUrl), JSON.stringify(credential));
  }

  async clear(apiUrl: string): Promise<void> {
    await this.secrets.delete(credentialKey(apiUrl));
  }
}
