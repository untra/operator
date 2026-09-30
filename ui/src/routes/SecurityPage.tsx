// Security settings: browser sessions, connected devices, and service access keys.

import { useState } from "react";
import { ApiError } from "../api-client";
import type { Scope } from "@operator/bindings/Scope";
import { useApiMutation, useApiQuery } from "../api";
import {
  accessKeysQuery,
  createAccessKeyMutation,
  revokeAccessKeyMutation,
  revokeSessionMutation,
  sessionsQuery,
} from "../api/definitions";
import { PageHeader } from "../components/PageHeader";
import styles from "./SecurityPage.module.css";

const ALL_SCOPES: Scope[] = ["read", "write", "execute", "admin"];
const PAGE_SIZE = 10;

function formatDate(iso: string | null | undefined): string {
  if (!iso) {
    return "-";
  }
  return new Date(iso).toLocaleString();
}

export function SecurityPage() {
  const sessions = useApiQuery(sessionsQuery());
  const keys = useApiQuery(accessKeysQuery());
  const createKey = useApiMutation(createAccessKeyMutation);
  const revokeKey = useApiMutation(revokeAccessKeyMutation);
  const revokeSession = useApiMutation(revokeSessionMutation);

  const [keyName, setKeyName] = useState("");
  const [keyScopes, setKeyScopes] = useState<Scope[]>(["read"]);
  const [keyDays, setKeyDays] = useState(90);
  const [formError, setFormError] = useState<string | null>(null);
  const [sessionPage, setSessionPage] = useState(0);
  const [keyPage, setKeyPage] = useState(0);
  const sessionRows = sessions.data?.sessions ?? [];
  const deviceRows = sessions.data?.devices ?? [];
  const keyRows = keys.data?.keys ?? [];
  const visibleSessions = sessionRows.slice(sessionPage * PAGE_SIZE, (sessionPage + 1) * PAGE_SIZE);
  const visibleKeys = keyRows.slice(keyPage * PAGE_SIZE, (keyPage + 1) * PAGE_SIZE);

  const error =
    formError ??
    sessions.error?.message ??
    keys.error?.message ??
    revokeKey.error?.message ??
    revokeSession.error?.message ??
    null;

  async function onCreateKey(event: React.SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setFormError(null);
    try {
      await createKey.mutateAsync({
        name: keyName,
        scopes: keyScopes,
        expires_in_days: BigInt(keyDays),
      });
      setKeyName("");
      setKeyPage(0);
    } catch (e) {
      setFormError(e instanceof ApiError ? e.message : "Failed to create the access key.");
    }
  }

  function onRevokeKey(id: string) {
    setFormError(null);
    revokeKey.mutate(
      { id },
      {
        onError: (e) => {
          setFormError(e instanceof ApiError ? e.message : "Failed to revoke the key.");
        },
      },
    );
  }

  function onRevokeSession(id: string) {
    setFormError(null);
    revokeSession.mutate(
      { id },
      {
        onError: (e) => {
          setFormError(e instanceof ApiError ? e.message : "Failed to revoke the session.");
        },
      },
    );
  }

  function toggleScope(scope: Scope) {
    setKeyScopes((current) =>
      current.includes(scope) ? current.filter((s) => s !== scope) : [...current, scope],
    );
  }

  return (
    <div className={styles.page}>
      <PageHeader
        title="Security"
        summary="Sessions, connected devices, and service access keys for this workspace."
        docsUrl="https://operator.untra.io/security/authentication/"
      />

      {error && <p className={styles.error}>{error}</p>}
      {(sessions.error ?? keys.error) && (
        <p className={styles.sectionHint}>Check the server connection, then reload this page.</p>
      )}

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Browser sessions</h2>
        <p className={styles.sectionHint}>
          Signed-in browsers. Revoking a session signs it out immediately.
        </p>
        {sessions.isLoading && <p className={styles.sectionHint}>Loading browser sessions…</p>}
        {!sessions.isLoading && !sessions.error && sessionRows.length === 0 && (
          <p className={styles.sectionHint}>No browser sessions were found.</p>
        )}
        {!sessions.isLoading && sessionRows.length > 0 && (
          <table className={styles.table}>
            <thead>
              <tr>
                <th>Started</th>
                <th>Expires</th>
                <th>Last used</th>
                <th aria-label="Actions" />
              </tr>
            </thead>
            <tbody>
              {visibleSessions.map((s) => (
                <tr key={s.id} className={s.revoked_at ? styles.revoked : undefined}>
                  <td>
                    {formatDate(s.created_at)}
                    {s.current && <span className={styles.current}>this browser</span>}
                  </td>
                  <td>{formatDate(s.expires_at)}</td>
                  <td>{formatDate(s.last_used_at)}</td>
                  <td>
                    <button
                      className={styles.danger}
                      onClick={() => onRevokeSession(s.id)}
                      disabled={!!s.revoked_at}
                    >
                      Revoke
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {sessionRows.length > PAGE_SIZE && (
          <div className={styles.pagination}>
            <button
              type="button"
              disabled={sessionPage === 0}
              onClick={() => setSessionPage((page) => Math.max(0, page - 1))}
            >
              Previous
            </button>
            <span>Page {sessionPage + 1}</span>
            <button
              type="button"
              disabled={(sessionPage + 1) * PAGE_SIZE >= sessionRows.length}
              onClick={() => setSessionPage((page) => page + 1)}
            >
              Next
            </button>
          </div>
        )}
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Connected devices</h2>
        <p className={styles.sectionHint}>
          Editors and other clients authorized through the device flow.
        </p>
        {sessions.isLoading && <p className={styles.sectionHint}>Loading connected devices…</p>}
        {!sessions.isLoading && !sessions.error && deviceRows.length === 0 && (
          <p className={styles.sectionHint}>No connected devices were found.</p>
        )}
        {!sessions.isLoading && deviceRows.length > 0 && (
          <table className={styles.table}>
            <thead>
              <tr>
                <th>Client</th>
                <th>Scopes</th>
                <th>Approved</th>
                <th>Last used</th>
              </tr>
            </thead>
            <tbody>
              {deviceRows.map((d) => (
                <tr key={d.id} className={d.revoked_at ? styles.revoked : undefined}>
                  <td>{d.client_id}</td>
                  <td>{d.scopes.join(", ")}</td>
                  <td>{formatDate(d.created_at)}</td>
                  <td>{formatDate(d.last_used_at)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Service access keys</h2>
        <p className={styles.sectionHint}>
          For integrations. Grant only the scopes the integration needs; every key expires.
        </p>

        {createKey.data?.secret && (
          <div className={styles.secretBox}>
            <strong>Copy this now - it is shown once and cannot be retrieved.</strong>
            <code className={styles.secret}>{createKey.data.secret}</code>
          </div>
        )}

        <form className={styles.form} onSubmit={onCreateKey}>
          <label>
            Name
            <input
              value={keyName}
              onChange={(e) => setKeyName(e.target.value)}
              placeholder="ci-pipeline"
              required
            />
          </label>
          <label>
            Expires in (days)
            <input
              type="number"
              min={1}
              max={365}
              value={keyDays}
              onChange={(e) => setKeyDays(Number(e.target.value))}
              required
            />
          </label>
          <div className={styles.scopes}>
            {ALL_SCOPES.map((scope) => (
              <label key={scope}>
                <input
                  type="checkbox"
                  checked={keyScopes.includes(scope)}
                  onChange={() => toggleScope(scope)}
                />
                {scope}
              </label>
            ))}
          </div>
          <button
            className={styles.primary}
            type="submit"
            disabled={createKey.isPending || !keyName || keyScopes.length === 0}
          >
            Create key
          </button>
        </form>

        {keys.isLoading && <p className={styles.sectionHint}>Loading access keys…</p>}
        {!keys.isLoading && !keys.error && keyRows.length === 0 && (
          <p className={styles.sectionHint}>No service access keys have been created.</p>
        )}
        {!keys.isLoading && keyRows.length > 0 && (
          <table className={styles.table}>
            <thead>
              <tr>
                <th>Name</th>
                <th>Scopes</th>
                <th>Expires</th>
                <th>Last used</th>
                <th aria-label="Actions" />
              </tr>
            </thead>
            <tbody>
              {visibleKeys.map((k) => (
                <tr key={k.id} className={k.revoked_at ? styles.revoked : undefined}>
                  <td>{k.name}</td>
                  <td>{k.scopes.join(", ")}</td>
                  <td>{formatDate(k.expires_at)}</td>
                  <td>{formatDate(k.last_used_at)}</td>
                  <td>
                    <button
                      className={styles.danger}
                      onClick={() => onRevokeKey(k.id)}
                      disabled={!!k.revoked_at}
                    >
                      Revoke
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {keyRows.length > PAGE_SIZE && (
          <div className={styles.pagination}>
            <button
              type="button"
              disabled={keyPage === 0}
              onClick={() => setKeyPage((page) => Math.max(0, page - 1))}
            >
              Previous
            </button>
            <span>Page {keyPage + 1}</span>
            <button
              type="button"
              disabled={(keyPage + 1) * PAGE_SIZE >= keyRows.length}
              onClick={() => setKeyPage((page) => page + 1)}
            >
              Next
            </button>
          </div>
        )}
      </section>
    </div>
  );
}
