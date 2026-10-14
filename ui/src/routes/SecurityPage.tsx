// Security settings: browser sessions, connected devices, and service access keys.

import { ApiError } from "../api-client";
import { useState } from "react";
import { AccessKeysPanel } from "../components/AccessKeysPanel";
import { useApiMutation, useApiQuery } from "../api";
import { revokeSessionMutation, sessionsQuery } from "../api/definitions";
import { PageHeader } from "../components/PageHeader";
import styles from "./SecurityPage.module.css";

const PAGE_SIZE = 10;

function formatDate(iso: string | null | undefined): string {
  if (!iso) {
    return "-";
  }
  return new Date(iso).toLocaleString();
}

export function SecurityPage() {
  const sessions = useApiQuery(sessionsQuery());
  const revokeSession = useApiMutation(revokeSessionMutation);

  const [formError, setFormError] = useState<string | null>(null);
  const [sessionPage, setSessionPage] = useState(0);
  const sessionRows = sessions.data?.sessions ?? [];
  const deviceRows = sessions.data?.devices ?? [];
  const visibleSessions = sessionRows.slice(sessionPage * PAGE_SIZE, (sessionPage + 1) * PAGE_SIZE);

  const error = formError ?? sessions.error?.message ?? revokeSession.error?.message ?? null;

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

  return (
    <div className={styles.page}>
      <PageHeader
        title="Security"
        summary="Sessions, connected devices, and service access keys for this workspace."
        docsUrl="https://operator.untra.io/security/authentication/"
      />

      {error && <p className={styles.error}>{error}</p>}
      {sessions.error && (
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

      <AccessKeysPanel />
    </div>
  );
}
