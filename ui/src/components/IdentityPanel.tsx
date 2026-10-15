import { useCallback, useMemo, useState } from "react";
import type { OidcProvider } from "@operator/bindings/OidcProvider";
import type { OperatorUser } from "@operator/bindings/OperatorUser";
import type { PermissionGrant } from "@operator/bindings/PermissionGrant";
import type { UpdateOperatorUser } from "@operator/bindings/UpdateOperatorUser";
import { PermissionSelector } from "@operator/webcomponents";
import { useApiMutation, useApiQuery } from "../api";
import {
  createOidcConnectionMutation,
  disableOidcConnectionMutation,
  identityCapabilitiesQuery,
  oidcConnectionsQuery,
  operatorUsersQuery,
  updateOperatorUserMutation,
} from "../api/definitions";
import { useHost } from "../host";

const PAGE_SIZE = 50;

export function IdentityPanel() {
  const host = useHost();
  const permissions = useApiQuery(identityCapabilitiesQuery());
  return permissions.data?.administrator ? (
    <IdentityAdministration
      key={`${host.baseUrl()}:${host.profileId ?? "default"}`}
      available={permissions.data.administration_grants ?? permissions.data.grants}
    />
  ) : null;
}

function IdentityAdministration({ available }: { available: PermissionGrant }) {
  const connections = useApiQuery(oidcConnectionsQuery());

  const create = useApiMutation(createOidcConnectionMutation);
  const disable = useApiMutation(disableOidcConnectionMutation);
  const [provider, setProvider] = useState<OidcProvider>("keycloak");
  const [issuer, setIssuer] = useState("");
  const [clientId, setClientId] = useState("");
  const [redirect, setRedirect] = useState("");
  const [secretFile, setSecretFile] = useState("");
  const [page, setPage] = useState(0);
  const users = useApiQuery(operatorUsersQuery(page * PAGE_SIZE));
  const [connectionPage, setConnectionPage] = useState(0);
  const rows = users.data ?? [];
  const providers = connections.data ?? [];
  const error = connections.error ?? users.error ?? create.error ?? disable.error;
  return (
    <section>
      <h2>Operator identities</h2>
      <p>
        Keycloak and Authentik connections require Premium. New users await approval and start
        without permissions. Disabling a connection removes access for its users and their keys.
      </p>
      {error && <p role="alert">{error.message}</p>}
      <button
        type="button"
        onClick={() => {
          void connections.refetch().catch(() => undefined);
          void users.refetch().catch(() => undefined);
        }}
      >
        Refresh identities
      </button>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          create.mutate({
            provider,
            issuer,
            client_id: clientId,
            redirect_uri: redirect,
            client_secret_file: secretFile || null,
          });
        }}
      >
        <label>
          Provider
          <select
            value={provider}
            onChange={(event) =>
              setProvider(event.target.value === "authentik" ? "authentik" : "keycloak")
            }
          >
            <option value="keycloak">Keycloak</option>
            <option value="authentik">Authentik</option>
          </select>
        </label>
        <label>
          Issuer HTTPS URL
          <input
            required
            type="url"
            value={issuer}
            onChange={(event) => setIssuer(event.target.value)}
          />
        </label>
        <label>
          Client ID
          <input
            required
            maxLength={128}
            value={clientId}
            onChange={(event) => setClientId(event.target.value)}
          />
        </label>
        <label>
          Redirect URI
          <input
            required
            type="url"
            value={redirect}
            onChange={(event) => setRedirect(event.target.value)}
          />
        </label>
        <label>
          Client secret file on the Operator server (optional)
          <input value={secretFile} onChange={(event) => setSecretFile(event.target.value)} />
        </label>
        <button disabled={create.isPending} type="submit">
          Add connection
        </button>
      </form>
      {connections.isLoading && <p>Loading connections...</p>}
      {!connections.isLoading && !connections.error && providers.length === 0 && (
        <p>No OIDC connections configured.</p>
      )}
      <ul>
        {providers
          .slice(connectionPage * PAGE_SIZE, (connectionPage + 1) * PAGE_SIZE)
          .map((connection) => (
            <li key={connection.id}>
              {connection.provider}: {connection.issuer}{" "}
              {connection.disabled ? (
                "Disabled"
              ) : (
                <button
                  type="button"
                  disabled={disable.isPending}
                  onClick={() => disable.mutate({ id: connection.id })}
                >
                  Disable connection
                </button>
              )}
            </li>
          ))}
      </ul>
      <button
        type="button"
        disabled={connectionPage === 0}
        onClick={() => setConnectionPage((value) => value - 1)}
      >
        Previous connections
      </button>
      <button
        type="button"
        disabled={(connectionPage + 1) * PAGE_SIZE >= providers.length}
        onClick={() => setConnectionPage((value) => value + 1)}
      >
        Next connections
      </button>
      <h3>User approval and permissions</h3>
      {users.isLoading && <p>Loading users...</p>}
      {!users.isLoading && !users.error && rows.length === 0 && (
        <p>Users appear here after their first OIDC sign-in.</p>
      )}
      {rows.map((user) => (
        <UserPermissions key={user.id} user={user} available={available} />
      ))}
      <button type="button" disabled={page === 0} onClick={() => setPage((value) => value - 1)}>
        Previous users
      </button>
      <button
        type="button"
        disabled={rows.length < PAGE_SIZE}
        onClick={() => setPage((value) => value + 1)}
      >
        Next users
      </button>
    </section>
  );
}

function UserPermissions({ user, available }: { user: OperatorUser; available: PermissionGrant }) {
  const update = useApiMutation(updateOperatorUserMutation);
  const [draft, setDraft] = useState<UpdateOperatorUser | null>(null);
  const value = useMemo(
    () =>
      draft ?? {
        enabled: user.enabled,
        administrator: user.administrator,
        grants: user.grants.find((grant) => grant.profile_id === available.profile_id) ?? {
          ...available,
          instance_wide: false,
          capabilities: [],
          projects: [],
        },
      },
    [draft, user, available],
  );
  const changeGrants = useCallback(
    (grants: PermissionGrant) => setDraft({ ...value, grants }),
    [value],
  );
  return (
    <details>
      <summary>
        {user.subject} ({user.issuer}) {user.enabled ? "Enabled" : "Pending or disabled"}
      </summary>
      {update.error && <p role="alert">{update.error.message}</p>}
      <form
        onSubmit={(event) => {
          event.preventDefault();
          update.mutate({ id: user.id, body: value }, { onSuccess: () => setDraft(null) });
        }}
      >
        <label>
          <input
            type="checkbox"
            checked={value.enabled}
            disabled={update.isPending}
            onChange={(event) => setDraft({ ...value, enabled: event.target.checked })}
          />
          Enabled
        </label>
        <label>
          <input
            type="checkbox"
            checked={value.administrator}
            disabled={update.isPending}
            onChange={(event) => setDraft({ ...value, administrator: event.target.checked })}
          />
          Operator administrator
        </label>
        <PermissionSelector
          available={available}
          value={value.grants}
          onChange={changeGrants}
          disabled={update.isPending}
        />
        <button type="submit" disabled={update.isPending}>
          Save permissions
        </button>
      </form>
    </details>
  );
}
