/**
 * Test doubles for the credential layer.
 */

import * as vscode from "vscode";
import type { CredentialProvider } from "../../../src/auth/credentials";

/** In-memory SecretStorage: enough of the interface for the token store. */
export class MemorySecrets implements vscode.SecretStorage {
  readonly values = new Map<string, string>();
  private readonly emitter = new vscode.EventEmitter<vscode.SecretStorageChangeEvent>();
  readonly onDidChange = this.emitter.event;

  get(key: string): Thenable<string | undefined> {
    return Promise.resolve(this.values.get(key));
  }

  store(key: string, value: string): Thenable<void> {
    this.values.set(key, value);
    this.emitter.fire({ key });
    return Promise.resolve();
  }

  delete(key: string): Thenable<void> {
    this.values.delete(key);
    this.emitter.fire({ key });
    return Promise.resolve();
  }

  keys(): Thenable<string[]> {
    return Promise.resolve([...this.values.keys()]);
  }
}

export interface FakeCredentials extends CredentialProvider {
  /** What `bearer()` returns; set to `undefined` to simulate no credential. */
  token: string | undefined;
  /** What `refresh()` returns. */
  refreshed: string | undefined;
  refreshCalls: number;
}

/** A provider whose answers tests control directly. */
export function fakeCredentials(token: string | undefined = "test-token"): FakeCredentials {
  const fake: FakeCredentials = {
    token,
    refreshed: undefined,
    refreshCalls: 0,
    bearer: () => Promise.resolve(fake.token),
    refresh: () => {
      fake.refreshCalls += 1;
      return Promise.resolve(fake.refreshed);
    },
    setTicketsDir: () => {},
  };
  return fake;
}
