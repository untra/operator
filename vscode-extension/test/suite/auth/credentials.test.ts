/**
 * Tests for src/auth/credentials.ts and src/auth/token-store.ts
 *
 * Group 2: Service Logic - fetch and the filesystem are faked.
 */

import * as assert from "node:assert";
import * as sinon from "sinon";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import {
  CLIENT_ID,
  EXPIRY_SKEW_MS,
  LOCAL_TOKEN_FILENAME,
  OperatorCredentials,
  SESSION_FILENAME,
  clearCredentialProvider,
  credentialProvider,
  isLoopbackUrl,
  readLocalToken,
  setCredentialProvider,
} from "../../../src/auth/credentials";
import { TokenStore, credentialKey } from "../../../src/auth/token-store";
import { MemorySecrets, fakeCredentials } from "../helpers/credentials";

const API_URL = "http://localhost:7008";
const REMOTE_URL = "http://build-box.internal:7008";

function tokenResponse(accessToken: string, refreshToken: string, expiresIn = 900): Response {
  return new Response(
    JSON.stringify({
      access_token: accessToken,
      token_type: "Bearer",
      expires_in: expiresIn,
      refresh_token: refreshToken,
      scopes: ["read", "write", "execute", "admin"],
    }),
    { status: 200 },
  );
}

function oauthError(error: string): Response {
  return new Response(JSON.stringify({ error }), { status: 400 });
}

suite("Credentials Test Suite", () => {
  let fetchStub: sinon.SinonStub;
  let secrets: MemorySecrets;
  let store: TokenStore;
  let credentials: OperatorCredentials;
  let ticketsDir: string;

  setup(async () => {
    fetchStub = sinon.stub(global, "fetch");
    secrets = new MemorySecrets();
    store = new TokenStore(secrets);
    credentials = new OperatorCredentials(store);
    ticketsDir = await fs.mkdtemp(path.join(os.tmpdir(), "operator-credentials-"));
    credentials.setTicketsDir(ticketsDir);
  });

  teardown(async () => {
    sinon.restore();
    clearCredentialProvider();
    await fs.rm(ticketsDir, { recursive: true, force: true });
  });

  async function writeLocalToken(token: string, stateDir = path.join(ticketsDir, "operator")) {
    await fs.mkdir(stateDir, { recursive: true });
    await fs.writeFile(path.join(stateDir, LOCAL_TOKEN_FILENAME), `${token}\n`);
  }

  async function storeCredential(expiresInMs: number, accessToken = "stored-access") {
    await store.save(API_URL, {
      access_token: accessToken,
      refresh_token: "stored-refresh",
      expires_at: Date.now() + expiresInMs,
      scopes: ["read"],
    });
  }

  suite("module-level provider", () => {
    test("throws when no provider has been configured", () => {
      assert.throws(() => credentialProvider(), /activate\(\) must call setCredentialProvider/);
    });

    test("returns the configured provider", () => {
      const fake = fakeCredentials();
      setCredentialProvider(fake);
      assert.strictEqual(credentialProvider(), fake);
    });

    test("clear removes the provider again", () => {
      setCredentialProvider(fakeCredentials());
      clearCredentialProvider();
      assert.throws(() => credentialProvider());
    });
  });

  suite("isLoopbackUrl()", () => {
    test("accepts localhost and loopback literals", () => {
      assert.ok(isLoopbackUrl("http://localhost:7008"));
      assert.ok(isLoopbackUrl("http://127.0.0.1:7008"));
      assert.ok(isLoopbackUrl("http://127.0.0.5"));
      assert.ok(isLoopbackUrl("http://[::1]:7008"));
    });

    test("rejects other hosts and garbage", () => {
      assert.ok(!isLoopbackUrl(REMOTE_URL));
      assert.ok(!isLoopbackUrl("http://0.0.0.0:7008"));
      assert.ok(!isLoopbackUrl("not a url"));
    });
  });

  suite("local token discovery", () => {
    test("reads the token from the default state dir next to the session file", async () => {
      await writeLocalToken("local-secret");
      assert.strictEqual(await readLocalToken(ticketsDir), "local-secret");
    });

    test("follows state_dir advertised in api-session.json", async () => {
      const customState = path.join(ticketsDir, "elsewhere");
      await writeLocalToken("custom-secret", customState);
      await fs.mkdir(path.join(ticketsDir, "operator"), { recursive: true });
      await fs.writeFile(
        path.join(ticketsDir, "operator", SESSION_FILENAME),
        JSON.stringify({
          port: 7008,
          pid: 1,
          started_at: "",
          version: "0",
          state_dir: customState,
        }),
      );
      assert.strictEqual(await readLocalToken(ticketsDir), "custom-secret");
    });

    test("is absent when the file is missing or blank", async () => {
      assert.strictEqual(await readLocalToken(ticketsDir), undefined);
      await writeLocalToken("   ");
      assert.strictEqual(await readLocalToken(ticketsDir), undefined);
    });
  });

  suite("bearer()", () => {
    test("prefers the local token over a stored device credential", async () => {
      await writeLocalToken("local-secret");
      await storeCredential(60_000);
      assert.strictEqual(await credentials.bearer(API_URL), "local-secret");
      assert.ok(fetchStub.notCalled);
    });

    test("never sends the local token to a non-loopback daemon", async () => {
      await writeLocalToken("local-secret");
      assert.strictEqual(await credentials.bearer(REMOTE_URL), undefined);
    });

    test("falls back to the stored access token when there is no local token", async () => {
      await storeCredential(60_000);
      assert.strictEqual(await credentials.bearer(API_URL), "stored-access");
      assert.ok(fetchStub.notCalled);
    });

    test("refreshes when the stored access token is within the expiry skew", async () => {
      await storeCredential(EXPIRY_SKEW_MS - 1000);
      fetchStub.resolves(tokenResponse("fresh-access", "fresh-refresh"));

      assert.strictEqual(await credentials.bearer(API_URL), "fresh-access");
      const init = fetchStub.firstCall.args[1] as { body: string };
      const body = JSON.parse(init.body) as Record<string, string>;
      assert.strictEqual(body.grant_type, "refresh_token");
      assert.strictEqual(body.refresh_token, "stored-refresh");
      assert.strictEqual(body.client_id, CLIENT_ID);
    });

    test("is undefined with nothing stored and no local token", async () => {
      assert.strictEqual(await credentials.bearer(API_URL), undefined);
    });

    test("ignores the local token when no tickets dir is known", async () => {
      await writeLocalToken("local-secret");
      credentials.setTicketsDir(undefined);
      assert.strictEqual(await credentials.bearer(API_URL), undefined);
    });
  });

  suite("refresh()", () => {
    test("stores the rotated pair", async () => {
      await storeCredential(0);
      fetchStub.resolves(tokenResponse("fresh-access", "fresh-refresh", 900));

      await credentials.refresh(API_URL);

      const stored = await store.load(API_URL);
      assert.ok(stored);
      assert.strictEqual(stored.access_token, "fresh-access");
      assert.strictEqual(stored.refresh_token, "fresh-refresh");
      assert.ok(stored.expires_at > Date.now() + 800_000);
    });

    test("shares one in-flight request between concurrent callers", async () => {
      await storeCredential(0);
      let release: (r: Response) => void = () => {};
      fetchStub.returns(
        new Promise<Response>((resolve) => {
          release = resolve;
        }),
      );

      const first = credentials.refresh(API_URL);
      const second = credentials.refresh(API_URL);
      release(tokenResponse("fresh-access", "fresh-refresh"));

      assert.deepStrictEqual(await Promise.all([first, second]), ["fresh-access", "fresh-access"]);
      assert.strictEqual(fetchStub.callCount, 1, "a second redemption would revoke the family");
    });

    test("allows a new refresh once the previous one settled", async () => {
      await storeCredential(0);
      fetchStub.onFirstCall().resolves(tokenResponse("a1", "r1"));
      fetchStub.onSecondCall().resolves(tokenResponse("a2", "r2"));

      assert.strictEqual(await credentials.refresh(API_URL), "a1");
      assert.strictEqual(await credentials.refresh(API_URL), "a2");
      assert.strictEqual(fetchStub.callCount, 2);
    });

    test("clears storage on invalid_grant", async () => {
      await storeCredential(0);
      fetchStub.resolves(oauthError("invalid_grant"));

      assert.strictEqual(await credentials.refresh(API_URL), undefined);
      assert.strictEqual(await store.load(API_URL), undefined);
    });

    test("keeps storage on a network failure", async () => {
      await storeCredential(0);
      fetchStub.rejects(new TypeError("fetch failed"));

      assert.strictEqual(await credentials.refresh(API_URL), undefined);
      assert.ok(await store.load(API_URL), "a transient failure must not sign the user out");
    });

    test("does nothing when nothing is stored", async () => {
      assert.strictEqual(await credentials.refresh(API_URL), undefined);
      assert.ok(fetchStub.notCalled);
    });
  });

  suite("TokenStore", () => {
    test("keys credentials per daemon url", () => {
      assert.notStrictEqual(credentialKey(API_URL), credentialKey(REMOTE_URL));
      assert.strictEqual(credentialKey("http://localhost:7008/"), credentialKey(API_URL));
    });

    test("round-trips and clears", async () => {
      await storeCredential(1000, "abc");
      assert.strictEqual((await store.load(API_URL))?.access_token, "abc");
      await store.clear(API_URL);
      assert.strictEqual(await store.load(API_URL), undefined);
    });

    test("treats corrupt storage as absent and removes it", async () => {
      secrets.values.set(credentialKey(API_URL), "{not json");
      assert.strictEqual(await store.load(API_URL), undefined);
      assert.ok(!secrets.values.has(credentialKey(API_URL)));
    });
  });
});
