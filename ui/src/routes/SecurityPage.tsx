// Security settings: browser sessions, connected devices, and service access keys.

import { useCallback, useEffect, useState } from 'react';
import { useHost } from '../host';
import { OperatorApi, ApiError } from '../api-client';
import type {
  AccessKeyListResponse,
  SessionListResponse,
} from '../api-client';
import type { Scope } from '@operator/bindings/Scope';
import { PageHeader } from '../components/PageHeader';
import styles from './SecurityPage.module.css';

const ALL_SCOPES: Scope[] = ['read', 'write', 'execute', 'admin'];

function formatDate(iso: string | null | undefined): string {
  if (!iso) {return '—';}
  return new Date(iso).toLocaleString();
}

export function SecurityPage() {
  const host = useHost();
  const [sessions, setSessions] = useState<SessionListResponse | null>(null);
  const [keys, setKeys] = useState<AccessKeyListResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Shown exactly once, right after creation: the server stores only a hash,
  // so there is no second chance to display it.
  const [newSecret, setNewSecret] = useState<string | null>(null);

  const [keyName, setKeyName] = useState('');
  const [keyScopes, setKeyScopes] = useState<Scope[]>(['read']);
  const [keyDays, setKeyDays] = useState(90);
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    const api = new OperatorApi(host);
    try {
      // A page load leaves no CSRF token in memory even though the session
      // cookie survived, so re-issue one before any mutation is possible.
      await api.refreshCsrf().catch(() => undefined);
      const [s, k] = await Promise.all([api.listSessions(), api.listAccessKeys()]);
      setSessions(s);
      setKeys(k);
      setError(null);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : 'Failed to load security settings.');
    }
  }, [host]);

  useEffect(() => {
    let cancelled = false;
    const api = new OperatorApi(host);
    void api
      .refreshCsrf()
      .catch(() => undefined)
      .then(() => Promise.all([api.listSessions(), api.listAccessKeys()]))
      .then(([s, k]) => {
        if (!cancelled) {
          setSessions(s);
          setKeys(k);
          setError(null);
        }
        return undefined;
      })
      .catch(() => {
        if (!cancelled) {
          setError('Failed to load security settings.');
        }
      });
    return () => {
      cancelled = true;
    };
  }, [host]);

  async function createKey(event: React.SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const res = await new OperatorApi(host).createAccessKey({
        name: keyName,
        scopes: keyScopes,
        expires_in_days: BigInt(keyDays),
      });
      setNewSecret(res.secret);
      setKeyName('');
      await load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : 'Failed to create the access key.');
    } finally {
      setBusy(false);
    }
  }

  async function revokeKey(id: string) {
    try {
      await new OperatorApi(host).revokeAccessKey(id);
      await load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : 'Failed to revoke the key.');
    }
  }

  async function revokeSession(id: string) {
    try {
      await new OperatorApi(host).revokeSession(id);
      await load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : 'Failed to revoke the session.');
    }
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

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Browser sessions</h2>
        <p className={styles.sectionHint}>
          Signed-in browsers. Revoking a session signs it out immediately.
        </p>
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
            {sessions?.sessions.map((s) => (
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
                    onClick={() => revokeSession(s.id)}
                    disabled={!!s.revoked_at}
                  >
                    Revoke
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Connected devices</h2>
        <p className={styles.sectionHint}>
          Editors and other clients authorized through the device flow.
        </p>
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
            {sessions?.devices.map((d) => (
              <tr key={d.id} className={d.revoked_at ? styles.revoked : undefined}>
                <td>{d.client_id}</td>
                <td>{d.scopes.join(', ')}</td>
                <td>{formatDate(d.created_at)}</td>
                <td>{formatDate(d.last_used_at)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Service access keys</h2>
        <p className={styles.sectionHint}>
          For integrations. Grant only the scopes the integration needs; every key
          expires.
        </p>

        {newSecret && (
          <div className={styles.secretBox}>
            <strong>Copy this now — it is shown once and cannot be retrieved.</strong>
            <code className={styles.secret}>{newSecret}</code>
          </div>
        )}

        <form className={styles.form} onSubmit={createKey}>
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
              max={3650}
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
            disabled={busy || !keyName || keyScopes.length === 0}
          >
            Create key
          </button>
        </form>

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
            {keys?.keys.map((k) => (
              <tr key={k.id} className={k.revoked_at ? styles.revoked : undefined}>
                <td>{k.name}</td>
                <td>{k.scopes.join(', ')}</td>
                <td>{formatDate(k.expires_at)}</td>
                <td>{formatDate(k.last_used_at)}</td>
                <td>
                  <button
                    className={styles.danger}
                    onClick={() => revokeKey(k.id)}
                    disabled={!!k.revoked_at}
                  >
                    Revoke
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </div>
  );
}
