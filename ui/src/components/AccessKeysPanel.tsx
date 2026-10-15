import { useEffect, useMemo, useRef, useState } from "react";
import type { PermissionGrant } from "@operator/bindings/PermissionGrant";
import type { UserKey } from "@operator/bindings/UserKey";
import { PermissionSelector } from "@operator/webcomponents";
import { useHost } from "../host";
import { useApiMutation, useApiQuery } from "../api";
import {
  accessKeysQuery,
  createAccessKeyMutation,
  revokeAccessKeyMutation,
  identityCapabilitiesQuery,
} from "../api/definitions";
import styles from "../routes/SecurityPage.module.css";

const PAGE_SIZE = 50;
const DEFAULT_EXPIRY_DAYS = 90;

export function AccessKeysPanel() {
  const host = useHost();
  return <ProfileKeys key={`${host.baseUrl()}:${host.profileId ?? "default"}`} />;
}

function ProfileKeys() {
  const host = useHost();
  const capabilities = useApiQuery(identityCapabilitiesQuery());
  const [grants, setGrants] = useState<PermissionGrant | null>(null);
  const [name, setName] = useState("");
  const [days, setDays] = useState(DEFAULT_EXPIRY_DAYS);
  const [offset, setOffset] = useState(0);
  const [secret, setSecret] = useState<string | null>(null);
  const createKey = useApiMutation(createAccessKeyMutation);
  const revokeKey = useApiMutation(revokeAccessKeyMutation);
  const busy = createKey.isPending || revokeKey.isPending;
  const [error, setError] = useState<string | null>(null);
  const generation = useRef(0);
  const available = capabilities.data;
  const keyQuery = useApiQuery(accessKeysQuery(offset), { enabled: available?.oidc_user ?? false });
  const keys = keyQuery.data?.keys ?? [];
  const selected = useMemo(
    () =>
      grants ??
      (available
        ? { ...available.grants, capabilities: [], projects: [], instance_wide: false }
        : null),
    [grants, available],
  );

  useEffect(() => {
    generation.current += 1;
    return () => {
      generation.current += 1;
    };
  }, []);

  function issue(key?: UserKey) {
    if (!selected || busy) {
      return;
    }
    const current = generation.current;
    setError(null);
    setSecret(null);
    createKey.mutate({
      request: { name: key?.name ?? name, expires_in_days: days, grants: key?.grants ?? selected },
      id: key?.id,
      onIssued: (value) => {
        if (generation.current === current) {
          setSecret(value);
        }
      },
    });
  }

  function revoke(id: string) {
    revokeKey.mutate({ id });
  }

  return (
    <section className={styles.section}>
      <h2>Operator API keys</h2>
      <p>
        Keys act on your behalf in this profile. Revocation and permission reductions apply on the
        next authorization check.
      </p>
      {(error ??
        capabilities.error?.message ??
        keyQuery.error?.message ??
        createKey.error?.message ??
        revokeKey.error?.message) && (
        <p role="alert">
          {error ??
            capabilities.error?.message ??
            keyQuery.error?.message ??
            createKey.error?.message ??
            revokeKey.error?.message}
        </p>
      )}
      {!available && <p>Loading permissions...</p>}
      {available && !available.oidc_user && (
        <p>Sign in with an approved OIDC account to create and manage your API keys.</p>
      )}
      {available?.oidc_user && !available.can_issue && (
        <p>
          Issuing or rotating keys requires Premium and an OIDC sign-in within the last five
          minutes.
        </p>
      )}
      {available?.connection_id && (
        <a
          href={`${host.baseUrl()}/api/v1/auth/oidc/login/${encodeURIComponent(available.connection_id)}`}
        >
          Sign in again
        </a>
      )}
      {secret && (
        <div className={styles.secretBox}>
          <strong>Copy this secret now. It cannot be retrieved later.</strong>
          <code className={styles.secret}>{secret}</code>
          <button
            type="button"
            onClick={() => {
              void navigator.clipboard
                .writeText(secret)
                .catch(() => setError("Clipboard unavailable. Select and copy the secret."));
            }}
          >
            Copy secret
          </button>
          <button type="button" onClick={() => setSecret(null)}>
            Dismiss secret
          </button>
        </div>
      )}
      {available?.oidc_user && selected && (
        <form
          className={styles.form}
          onSubmit={(event) => {
            event.preventDefault();
            issue();
          }}
        >
          <label>
            Name
            <input
              required
              maxLength={128}
              value={name}
              onChange={(event) => setName(event.target.value)}
            />
          </label>
          <label>
            Expires in days
            <input
              required
              type="number"
              min={1}
              max={365}
              value={days}
              onChange={(event) => setDays(Number(event.target.value))}
            />
          </label>
          <PermissionSelector
            available={available.grants}
            value={selected}
            onChange={setGrants}
            disabled={busy || !available.can_issue}
          />
          <button
            type="submit"
            disabled={busy || !available.can_issue || selected.capabilities.length === 0}
          >
            Create key
          </button>
        </form>
      )}
      {available?.oidc_user && (
        <>
          <button
            type="button"
            onClick={() => {
              void keyQuery.refetch().catch(() => undefined);
            }}
          >
            Refresh keys
          </button>
          <p>
            Last used is approximate and normally updates within one minute. Rotation immediately
            revokes the old key. For an overlap, create a replacement and revoke the old key after
            switching clients.
          </p>
          {keyQuery.isLoading && <p>Loading keys...</p>}
          {!keyQuery.isLoading && !keyQuery.error && keys.length === 0 && (
            <p>No keys on this page.</p>
          )}
          <table className={styles.table}>
            <thead>
              <tr>
                <th>Name and prefix</th>
                <th>Permissions</th>
                <th>Expires</th>
                <th>Last used</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {keys.map((key) => (
                <tr key={key.id}>
                  <td>
                    {key.name}
                    <br />
                    <code>{key.display_prefix}</code>
                    {key.revoked_at ? " (revoked)" : ""}
                  </td>
                  <td>
                    {key.grants.capabilities.join(", ")}
                    <br />
                    {key.grants.projects.join(", ") || "No projects"}
                  </td>
                  <td>{new Date(key.expires_at).toLocaleString()}</td>
                  <td>
                    {key.last_used_at
                      ? new Date(key.last_used_at).toLocaleString()
                      : "Never observed"}
                  </td>
                  <td>
                    <button
                      type="button"
                      disabled={busy || !!key.revoked_at || !available.can_issue}
                      onClick={() => {
                        issue(key);
                      }}
                    >
                      Rotate
                    </button>
                    <button
                      type="button"
                      disabled={busy || !!key.revoked_at}
                      onClick={() => {
                        revoke(key.id);
                      }}
                    >
                      Revoke
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <button
            type="button"
            disabled={offset === 0}
            onClick={() => setOffset((value) => Math.max(0, value - PAGE_SIZE))}
          >
            Previous
          </button>
          <button
            type="button"
            disabled={keys.length < PAGE_SIZE}
            onClick={() => setOffset((value) => value + PAGE_SIZE)}
          >
            Next
          </button>
        </>
      )}
    </section>
  );
}
