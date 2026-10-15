import { useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import type { PermissionGrant } from "@operator/bindings/PermissionGrant";
import type { CapabilityResponse } from "@operator/bindings/CapabilityResponse";
import { getSessionVersion, subscribeSession } from "../api/adapter";
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

const PERMISSIONS_POLL_MS = 30_000;
const CLOCK_TICK_MS = 1000;
const systemTime = () => Date.now();

interface AccessKeysPanelProps {
  clock?: () => number;
}

export function AccessKeysPanel({ clock = systemTime }: AccessKeysPanelProps) {
  const host = useHost();
  const sessionVersion = useSyncExternalStore(subscribeSession, getSessionVersion);
  const capabilities = useApiQuery(identityCapabilitiesQuery(), { pollIntervalMs: PERMISSIONS_POLL_MS });
  const available = capabilities.data;
  if (!available || capabilities.error) {
    return <section id="operator-api-keys" className={styles.section}>
      <h2>Operator API keys</h2>
      {capabilities.error ? <><p role="alert">{capabilities.error.message}</p><button type="button" onClick={() => { void capabilities.refetch().catch(() => undefined); }}>Retry permissions</button></> : <p>Loading permissions...</p>}
    </section>;
  }
  return <ProfileKeys key={`${host.baseUrl()}:${host.profileId ?? "default"}:${available.owner_id ?? "anonymous"}:${sessionVersion}`} available={available} clock={clock} />;
}

interface ProfileKeysProps {
  available: CapabilityResponse;
  clock: () => number;
}

function ProfileKeys({ available, clock }: ProfileKeysProps) {
  const host = useHost();
  const [now, setNow] = useState(clock);
  useEffect(() => {
    const timer = setInterval(() => setNow(clock()), CLOCK_TICK_MS);
    return () => clearInterval(timer);
  }, [clock]);
  const canIssue = available.can_issue && (!available.issuance_expires_at || now < Date.parse(available.issuance_expires_at));
  const [confirmation, setConfirmation] = useState<{ action: "rotate" | "revoke"; key: UserKey } | null>(null);
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
  const keyQuery = useApiQuery(accessKeysQuery(offset), { enabled: available.oidc_user ?? false, pollIntervalMs: PERMISSIONS_POLL_MS });
  const keys = keyQuery.data?.keys ?? [];
  const selected = useMemo(
    () =>
      grants ??
      { ...available.grants, capabilities: [], projects: [], instance_wide: false },
    [grants, available],
  );

  useEffect(() => {
    generation.current += 1;
    return () => {
      generation.current += 1;
    };
  }, []);

  function issue(key?: UserKey) {
    if (!selected || busy || !canIssue || (key && (key.revoked_at || Date.parse(key.expires_at) <= clock()))) {
      return;
    }
    const current = generation.current;
    setError(null);
    setSecret(null);
    setConfirmation(null);
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
    setSecret(null);
    setConfirmation(null);
    revokeKey.mutate({ id });
  }

  return (
    <section id="operator-api-keys" className={styles.section}>
      <h2>Operator API keys</h2>
      <p>
        Keys act on your behalf in this profile. Revocation and permission reductions apply on the
        next authorization check.
      </p>
      {(error ??
        keyQuery.error?.message ??
        createKey.error?.message ??
        revokeKey.error?.message) && (
        <p role="alert">
          {error ??
                keyQuery.error?.message ??
            createKey.error?.message ??
            revokeKey.error?.message}
        </p>
      )}
      {available && !available.oidc_user && (
        <p>Sign in with an approved OIDC account to create and manage your API keys.</p>
      )}
      {available?.oidc_user && !canIssue && (
        <p>
          {available.issuance_block_reason === "premium_required"
            ? "Premium is required to issue or rotate keys. You can still list and revoke keys."
            : available.issuance_block_reason === "permissions_required"
              ? "Your account has no capabilities to grant in this profile. Contact an administrator."
              : "Sign in with OIDC again to issue or rotate keys. Authentication must be within five minutes."}
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
            descriptions={available.capability_descriptions}
            value={selected}
            onChange={setGrants}
            disabled={busy || !canIssue}
          />
          <button
            type="submit"
            disabled={busy || !canIssue || selected.capabilities.length === 0}
          >
            Create key
          </button>
        </form>
      )}
      {available?.oidc_user && (
        <>
          {confirmation && <div role="group" aria-label="Confirm key action">
            <p>{confirmation.action === "rotate" ? "Rotate" : "Revoke"} {confirmation.key.name} ({confirmation.key.display_prefix})? The old key and its access tokens will stop working immediately. Already authorized operations may finish.</p>
            {confirmation.action === "rotate" && <>
              <p>Replacement expires in {days} days. You may remove permissions before rotating.</p>
              <PermissionSelector available={keys.find((key) => key.id === confirmation.key.id)?.grants ?? confirmation.key.grants} descriptions={available.capability_descriptions} value={confirmation.key.grants} disabled={busy} onChange={(value) => setConfirmation({ ...confirmation, key: { ...confirmation.key, grants: value } })} />
            </>}
            <button type="button" disabled={busy || (confirmation.action === "rotate" && (!canIssue || confirmation.key.grants.capabilities.length === 0))} onClick={() => {
              if (confirmation.action === "rotate") { issue(confirmation.key); } else { revoke(confirmation.key.id); }
            }}>Confirm {confirmation.action}</button>
            <button type="button" onClick={() => setConfirmation(null)}>Cancel</button>
          </div>}
          <button
            type="button"
            onClick={() => {
              void keyQuery.refetch().catch(() => undefined);
            }}
          >
            Refresh keys
          </button>
          <p>
            Last used records successful authentication, including denied operations. It is approximate and normally updates within one minute. Rotation immediately
            revokes the old key. For an overlap, create a replacement and revoke the old key after
            switching clients.
          </p>
          <p>If an issuance response is lost, refresh the list and revoke the new key before trying again. Its secret cannot be recovered.</p>
          {keyQuery.isLoading && <p>Loading keys...</p>}
          {!keyQuery.isLoading && !keyQuery.error && keys.length === 0 && (
            <p>No keys on this page.</p>
          )}
          <table className={styles.table}>
            <thead>
              <tr>
                <th>Name and prefix</th>
                <th>Status and profile</th>
                <th>Permissions</th>
                <th>Created</th>
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
                  </td>
                  <td>{key.revoked_at ? "Revoked" : Date.parse(key.expires_at) <= now ? "Expired" : "Active"}<br />{key.profile_id}</td>
                  <td>
                    {key.grants.capabilities.join(", ")}
                    <br />
                    {key.grants.projects.join(", ") || "No projects"}
                    <br />{key.grants.instance_wide ? "Selected instance operations allowed" : "No instance operations"}
                  </td>
                  <td>{new Date(key.created_at).toLocaleString()}</td>
                  <td>{new Date(key.expires_at).toLocaleString()}</td>
                  <td>
                    {key.last_used_at
                      ? new Date(key.last_used_at).toLocaleString()
                      : "Never observed"}
                  </td>
                  <td>
                    <button
                      type="button"
                      disabled={busy || !!key.revoked_at || Date.parse(key.expires_at) <= now || !canIssue}
                      onClick={() => {
                        setConfirmation({ action: "rotate", key: { ...key, grants: {
                          ...key.grants,
                          capabilities: key.grants.capabilities.filter((capability) => available.grants.capabilities.includes(capability)),
                          projects: key.grants.projects.filter((project) => available.grants.projects.includes(project)),
                          instance_wide: key.grants.instance_wide && available.grants.instance_wide,
                        } } });
                      }}
                    >
                      Rotate
                    </button>
                    <button
                      type="button"
                      disabled={busy || !!key.revoked_at}
                      onClick={() => {
                        setConfirmation({ action: "revoke", key });
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
