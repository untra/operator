import { useEffect, useRef, useState } from "react";
import type { Scope } from "@operator/bindings/Scope";
import { useApiMutation, useApiQuery } from "../api";
import {
  accessKeysQuery,
  createAccessKeyMutation,
  revokeAccessKeyMutation,
} from "../api/definitions";
import styles from "../routes/SecurityPage.module.css";

const ALL_SCOPES: Scope[] = ["read", "write", "execute", "admin"];
const PAGE_SIZE = 10;
const DEFAULT_EXPIRY_DAYS = 90;
const DEFAULT_SCOPES: Scope[] = ["read"];

interface AccessKeysPanelProps {
  defaultScopes?: Scope[];
}

function formatDate(iso: string | null | undefined): string {
  return iso ? new Date(iso).toLocaleString() : "-";
}

export function AccessKeysPanel({ defaultScopes = DEFAULT_SCOPES }: AccessKeysPanelProps) {
  const keys = useApiQuery(accessKeysQuery());
  const createKey = useApiMutation(createAccessKeyMutation);
  const revokeKey = useApiMutation(revokeAccessKeyMutation);
  const [keyName, setKeyName] = useState("");
  const [keyScopes, setKeyScopes] = useState<Scope[]>(defaultScopes);
  const [keyDays, setKeyDays] = useState(DEFAULT_EXPIRY_DAYS);
  const [keyPage, setKeyPage] = useState(0);
  const [copyError, setCopyError] = useState<string | null>(null);
  const keyRows = keys.data?.keys ?? [];
  const visibleKeys = keyRows.slice(keyPage * PAGE_SIZE, (keyPage + 1) * PAGE_SIZE);
  const resetKey = createKey.reset;
  const mounted = useRef(false);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      resetKey();
    };
  }, [resetKey]);

  function onCreateKey(event: React.SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    createKey.mutate(
      { name: keyName, scopes: keyScopes, expires_in_days: BigInt(keyDays) },
      {
        onSuccess: () => {
          if (!mounted.current) {
            resetKey();
          } else {
            setKeyPage(0);
          }
        },
        onError: () => {
          if (!mounted.current) {
            resetKey();
          }
        },
      },
    );
  }

  function onRevokeKey(id: string) {
    revokeKey.mutate({ id });
  }

  function toggleScope(scope: Scope) {
    setKeyScopes((current) =>
      current.includes(scope) ? current.filter((item) => item !== scope) : [...current, scope],
    );
  }

  async function copySecret(secret: string) {
    try {
      await navigator.clipboard.writeText(secret);
      setCopyError(null);
    } catch {
      setCopyError("Clipboard unavailable. Select and copy the secret above.");
    }
  }

  return (
    <section className={styles.section}>
      <h2 className={styles.sectionTitle}>Service access keys</h2>
      {(keys.error ?? createKey.error ?? revokeKey.error) && (
        <p role="alert" className={styles.error}>
          {(keys.error ?? createKey.error ?? revokeKey.error)?.message}
        </p>
      )}
      {copyError && <p role="alert">{copyError}</p>}
      <button
        type="button"
        onClick={() => {
          void keys.refetch().catch(() => undefined);
        }}
      >
        Refresh keys
      </button>
      <p className={styles.sectionHint}>
        To rotate a key, create a replacement, update the client, then revoke the old key. Existing
        bearer tokens remain valid until they expire.
      </p>
      <p className={styles.sectionHint}>
        For integrations. Grant only the scopes the integration needs; every key expires.
      </p>

      {createKey.data?.secret && (
        <div className={styles.secretBox}>
          <strong>Copy this now - it is shown once and cannot be retrieved.</strong>
          <code className={styles.secret}>{createKey.data.secret}</code>
          <button
            type="button"
            onClick={() => {
              if (createKey.data) {
                void copySecret(createKey.data.secret);
              }
            }}
          >
            Copy secret
          </button>
          <button type="button" onClick={createKey.reset}>
            Dismiss secret
          </button>
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
                <td>
                  {k.name}
                  {k.revoked_at ? " (revoked)" : ""}
                </td>
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
  );
}
