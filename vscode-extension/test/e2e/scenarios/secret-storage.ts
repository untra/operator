import assert from "node:assert/strict";
import * as vscode from "vscode";
import { TokenStore } from "../../../src/auth/token-store";
import { requiredEnv } from "../daemon";

export async function run(): Promise<void> {
  const fixture = vscode.extensions.getExtension<vscode.SecretStorage>(
    "operator-e2e.secret-storage-fixture",
  );
  assert.ok(fixture);
  const store = new TokenStore(await fixture.activate());
  const origin = "https://secret-storage.example.invalid";
  const credentials = {
    access_token: "fixture-access",
    refresh_token: "fixture-refresh",
    scopes: [],
    expires_at: 1,
  };
  if (requiredEnv("OPERATOR_E2E_STORAGE_PHASE") === "write") {
    assert.equal(await store.load(origin), undefined);
    await store.save(origin, credentials);
  } else {
    assert.deepEqual(await store.load(origin), credentials);
    await store.clear(origin);
    assert.equal(await store.load(origin), undefined);
  }
}
