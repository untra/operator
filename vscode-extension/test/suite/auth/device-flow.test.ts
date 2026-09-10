/**
 * Tests for src/auth/device-flow.ts
 *
 * Group 2: Service Logic - fetch is faked; time is driven by injected hooks.
 */

import * as assert from 'assert';
import * as sinon from 'sinon';
import {
  DEVICE_CODE_PATH,
  DeviceFlowHooks,
  IDE_SCOPES,
  SLOW_DOWN_INCREMENT_SECS,
  pollForToken,
  requestDeviceCode,
  runDeviceFlow,
} from '../../../src/auth/device-flow';
import { CLIENT_ID, TOKEN_PATH } from '../../../src/auth/credentials';
import { TokenStore } from '../../../src/auth/token-store';
import type { DeviceAuthorizationResponse } from '../../../src/generated';
import { MemorySecrets } from '../helpers/credentials';

const API_URL = 'http://build-box.internal:7008';

// ts-rs types the u64 seconds as bigint, but a parsed JSON body carries
// numbers, which is what the code under test sees at runtime.
const AUTHORIZATION = {
  device_code: 'device-secret',
  user_code: 'ABCD-EFGH',
  verification_uri: `${API_URL}/#/device`,
  verification_uri_complete: `${API_URL}/#/device?user_code=ABCD-EFGH`,
  expires_in: 900,
  interval: 5,
} as unknown as DeviceAuthorizationResponse;

function json(body: unknown, status: number): Response {
  return new Response(JSON.stringify(body), { status });
}

const pending = () => json({ error: 'authorization_pending' }, 400);
const approved = () =>
  json(
    {
      access_token: 'access',
      token_type: 'Bearer',
      expires_in: 900,
      refresh_token: 'refresh',
      scopes: IDE_SCOPES,
    },
    200
  );

/** Hooks whose clock only advances when `sleep` is awaited. */
class FakeClock {
  time = 0;
  sleeps: number[] = [];
  cancelled = false;

  hooks(onCode: DeviceFlowHooks['onCode'] = () => Promise.resolve()): DeviceFlowHooks {
    return {
      onCode,
      isCancelled: () => this.cancelled,
      sleep: (ms) => {
        this.sleeps.push(ms);
        this.time += ms;
        return Promise.resolve();
      },
      now: () => this.time,
    };
  }
}

suite('Device Flow Test Suite', () => {
  let fetchStub: sinon.SinonStub;
  let store: TokenStore;
  let clock: FakeClock;

  setup(() => {
    fetchStub = sinon.stub(global, 'fetch');
    store = new TokenStore(new MemorySecrets());
    clock = new FakeClock();
  });

  teardown(() => {
    sinon.restore();
  });

  suite('requestDeviceCode()', () => {
    test('asks for every IDE scope as the vscode client', async () => {
      fetchStub.resolves(json(AUTHORIZATION, 200));

      const result = await requestDeviceCode(API_URL);

      assert.strictEqual(fetchStub.firstCall.args[0], `${API_URL}${DEVICE_CODE_PATH}`);
      const init = fetchStub.firstCall.args[1] as { method: string; body: string };
      assert.strictEqual(init.method, 'POST');
      assert.deepStrictEqual(JSON.parse(init.body), { client_id: CLIENT_ID, scopes: IDE_SCOPES });
      assert.strictEqual(result.user_code, 'ABCD-EFGH');
    });

    test('surfaces the server error description', async () => {
      fetchStub.resolves(json({ error: 'invalid_client', error_description: 'nope' }, 400));
      await assert.rejects(requestDeviceCode(API_URL), /nope/);
    });
  });

  suite('pollForToken()', () => {
    test('waits the advertised interval, then stores tokens on approval', async () => {
      fetchStub.onCall(0).resolves(pending());
      fetchStub.onCall(1).resolves(approved());

      const outcome = await pollForToken(API_URL, AUTHORIZATION, store, clock.hooks());

      assert.deepStrictEqual(outcome, { status: 'approved', scopes: IDE_SCOPES });
      assert.deepStrictEqual(clock.sleeps, [5000, 5000]);
      assert.strictEqual(fetchStub.firstCall.args[0], `${API_URL}${TOKEN_PATH}`);
      const init = fetchStub.firstCall.args[1] as { body: string };
      assert.deepStrictEqual(JSON.parse(init.body), {
        grant_type: 'urn:ietf:params:oauth:grant-type:device_code',
        device_code: 'device-secret',
        client_id: CLIENT_ID,
      });
      const stored = await store.load(API_URL);
      assert.strictEqual(stored?.access_token, 'access');
      assert.strictEqual(stored?.refresh_token, 'refresh');
      assert.strictEqual(stored?.expires_at, clock.time + 900_000);
    });

    test('lengthens the interval on slow_down', async () => {
      fetchStub.onCall(0).resolves(json({ error: 'slow_down' }, 400));
      fetchStub.onCall(1).resolves(approved());

      await pollForToken(API_URL, AUTHORIZATION, store, clock.hooks());

      assert.deepStrictEqual(clock.sleeps, [5000, (5 + SLOW_DOWN_INCREMENT_SECS) * 1000]);
    });

    test('reports expiry from the server', async () => {
      fetchStub.resolves(json({ error: 'expired_token' }, 400));
      assert.deepStrictEqual(
        await pollForToken(API_URL, AUTHORIZATION, store, clock.hooks()),
        { status: 'expired' }
      );
    });

    test('gives up locally once expires_in has elapsed', async () => {
      // A Response body reads once, so each poll needs its own.
      fetchStub.callsFake(() => Promise.resolve(pending()));
      const outcome = await pollForToken(API_URL, AUTHORIZATION, store, clock.hooks());
      assert.deepStrictEqual(outcome, { status: 'expired' });
      assert.strictEqual(fetchStub.callCount, 900 / 5);
    });

    test('reports denial', async () => {
      fetchStub.resolves(json({ error: 'access_denied' }, 400));
      assert.deepStrictEqual(
        await pollForToken(API_URL, AUTHORIZATION, store, clock.hooks()),
        { status: 'denied' }
      );
    });

    test('stops polling when cancelled', async () => {
      fetchStub.callsFake(() => {
        clock.cancelled = true;
        return Promise.resolve(pending());
      });

      const outcome = await pollForToken(API_URL, AUTHORIZATION, store, clock.hooks());

      assert.deepStrictEqual(outcome, { status: 'cancelled' });
      assert.strictEqual(fetchStub.callCount, 1);
      assert.strictEqual(await store.load(API_URL), undefined);
    });

    test('treats an unexpected error code as a failure', async () => {
      fetchStub.resolves(json({ error: 'invalid_grant', error_description: 'bad code' }, 400));
      assert.deepStrictEqual(
        await pollForToken(API_URL, AUTHORIZATION, store, clock.hooks()),
        { status: 'error', message: 'bad code' }
      );
    });
  });

  suite('runDeviceFlow()', () => {
    test('hands the code to the UI before polling', async () => {
      fetchStub.onCall(0).resolves(json(AUTHORIZATION, 200));
      fetchStub.onCall(1).resolves(approved());
      const shown: string[] = [];

      const outcome = await runDeviceFlow(
        API_URL,
        store,
        clock.hooks((auth) => {
          shown.push(auth.user_code);
          return Promise.resolve();
        })
      );

      assert.deepStrictEqual(shown, ['ABCD-EFGH']);
      assert.strictEqual(outcome.status, 'approved');
    });

    test('reports a failed code request without polling', async () => {
      fetchStub.rejects(new TypeError('fetch failed'));
      const outcome = await runDeviceFlow(API_URL, store, clock.hooks());
      assert.deepStrictEqual(outcome, { status: 'error', message: 'fetch failed' });
      assert.strictEqual(fetchStub.callCount, 1);
    });
  });
});
