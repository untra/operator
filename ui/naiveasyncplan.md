# Adopt NaiveAsync for All `ui/` API State

## Summary

- Add `@untra/naiveasync@^2.0.0` and Redux 5 to `ui/`, using NaiveAsync lifecycles as the sole owner of API response, loading, error, polling, and mutation state.
- Build an Operator-specific hook layer because NaiveAsync exposes `call`, `sync`, `onData`, and `onError`, while components need the requested query/mutation interface. Queries will use `.sync()` to preserve cached data; mutations will use `.call()`. See the [NaiveAsync documentation](https://github.com/untra/naiveasync).
- Migrate every API flow, including authentication, onboarding, polling, settings, panels, and mutations. `webcomponents/` remains presentation-only and receives no NaiveAsync, Redux, API-client, or lifecycle exposure.
- Keep `OperatorApi` as the transport layer. Only modules under `ui/src/api/` may call it; components must use typed query and mutation hooks.

## Implementation Changes

- Add direct runtime dependencies to `ui/package.json` and the canonical Bun lockfile:
  - `@untra/naiveasync@^2.0.0`
  - `redux@^5.0.1`
  - Required runtime peers: `react-redux@^9.3.0`, `rxjs@^7.8.2`, and `lodash@^4.18.1`
  - Extend the UI dependency allowlist in `tests/ui_packaging.rs`; do not change the `webcomponents/` allowlist.
- Mount one Redux/NaiveAsync provider above the router. Scope lifecycle IDs by API base URL, profile ID, query key, and safe query parameters so cached data cannot leak between configurations or parameterized resources.
- Export all query-key constants from `api/queries/`; components, tests, and stories must import them rather than reproduce string keys.
- Provide typed UI-only interfaces:
  - `useApiQuery(definition, { enabled?, pollIntervalMs? })` returning `data`, `error`, `isLoading`, `isFetching`, and `refetch`.
  - `useApiMutation(definition)` returning `data`, `error`, `isPending`, `mutate(variables, { onSuccess?, onError? })`, `mutateAsync(variables)`, and `reset`.
  - `isLoading` is true only when enabled and no cached data exists. Background polling or invalidation sets `isFetching` while retaining cached data.
  - Use `mutate()` callbacks by default; use `mutateAsync()` only where returned data controls a subsequent operation, navigation, or wizard decision.
- Keep passwords, tokens, license keys, and credentials out of Redux action parameters. Mutation/query coordinators will pass opaque request IDs through NaiveAsync and retain sensitive inputs only in a transient in-memory registry that is cleared in `finally`.
- Define query lifecycles for every current read, including auth/session state, profiles, setup catalogs, status/sections, queue/board/agents, configuration, issue types/workflows, licensing/targets, security, providers/models, and delegators. Preserve existing 3-second dashboard/section/queue polling and 5-second agent polling through lifecycle synchronization.
- Define each write as a mutation with an explicit affected-query list. Invalidation runs after both success and failure so partial server-side writes cannot leave cached data stale. Multi-step onboarding/auth flows remain separate mutations so each completed or failed step performs its own invalidation.
- Reset all session-scoped lifecycle state on login/logout boundaries. Broad setup initialization invalidates all queries for the affected profile; narrower mutations invalidate their related families, such as:
  - Ticket/queue actions -> board, queue status, agents, sections.
  - Profile/setup/configuration changes -> profiles, setup status, configuration, status, sections.
  - License/target changes -> license, targets, execution targets.
  - Issue-type/collection changes -> issue types, workflow documents, collections, status.
  - Session/access-key changes -> current session, sessions, access keys.
  - Model-server/delegator changes -> provider models, model servers, delegators, configuration.
- Remove component-owned server-data `useState`/`useEffect`, manual cancellation flags, empty promise catches, and direct `OperatorApi` construction. Retain React state only for local form drafts, selection, visibility, navigation, and other presentation state.

## Test Plan

- Write failing adapter tests first, then implement:
  - Initial query without cached data reports loading.
  - `.sync()` preserves cached data during refresh and reports fetching rather than loading.
  - Query errors retain cached data.
  - Parameterized and profile-scoped queries cannot overwrite one another.
  - Mutation success and failure callbacks fire correctly.
  - Every declared affected query is invalidated on success and error, including failure of the second operation in a chain.
  - Sensitive variables never appear in Redux state or dispatched lifecycle parameters.
  - Poll cleanup prevents updates after unmount and does not expose stale parameter results.
- Add an architectural test that rejects direct `fetch` or `OperatorApi` usage in UI components/routes/contexts and verifies NaiveAsync remains confined to `ui/`.
- Update affected component tests to import query keys and seed lifecycle states with NaiveAsync's mock initial, pending, error, and success shapes.
- Run `bun run typecheck`, UI tests, UI lint/format checks, the packaging tests, and finally the required full `make check`.

## Assumptions

- The migration intentionally targets the currently published NaiveAsync 2.0 API and its `"" | "pending" | "error" | "success"` statuses.
- The archived upstream repository is accepted as an intentional dependency choice; Operator's adapter isolates components from that API and provides one replacement boundary if the store changes later.
- `ui/bun.lock` is the maintained UI lockfile used by CI; the already-stale npm lockfile is left untouched.
- Existing user-visible behavior, polling cadence, authentication redirects, and local form state remain unchanged unless lifecycle correctness requires eliminating stale server data.
