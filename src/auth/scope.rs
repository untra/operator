//! The route → required-scope table, and the authenticated principal.
//!
//! Authorization is decided by matching a request's [`MatchedPath`] against
//! [`ROUTE_RULES`] rather than by per-handler extractors. The tradeoff is
//! deliberate: an extractor that someone forgets to add leaves a route
//! **unprotected**, and nothing fails. A missing table entry is caught by
//! `tests/route_scope_parity.rs`, which walks the generated OpenAPI spec and
//! fails the build. The failure mode of a mistake should be a red test, not a
//! silent hole.
//!
//! [`MatchedPath`]: axum::extract::MatchedPath

use crate::rest::dto::auth::{PrincipalKind, Scope};

/// What a route requires of its caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// Reachable without any credential. The allowlist is deliberately tiny:
    /// probes, the bootstrap/login endpoints needed to *obtain* a credential,
    /// and the OAuth endpoints a device-flow client polls before it has one.
    Public,
    /// Requires an authenticated principal holding this scope.
    Scoped(Scope),
}

/// One route's authorization requirement.
#[derive(Debug, Clone, Copy)]
pub struct RouteRule {
    /// HTTP method, uppercase.
    pub method: &'static str,
    /// The axum route pattern, exactly as mounted (e.g. `/api/v1/tickets/{id}`).
    pub path: &'static str,
    /// What the route requires.
    pub access: Access,
}

const fn rule(method: &'static str, path: &'static str, access: Access) -> RouteRule {
    RouteRule {
        method,
        path,
        access,
    }
}

const fn public(method: &'static str, path: &'static str) -> RouteRule {
    rule(method, path, Access::Public)
}

const fn read(method: &'static str, path: &'static str) -> RouteRule {
    rule(method, path, Access::Scoped(Scope::Read))
}

const fn write(method: &'static str, path: &'static str) -> RouteRule {
    rule(method, path, Access::Scoped(Scope::Write))
}

const fn execute(method: &'static str, path: &'static str) -> RouteRule {
    rule(method, path, Access::Scoped(Scope::Execute))
}

const fn admin(method: &'static str, path: &'static str) -> RouteRule {
    rule(method, path, Access::Scoped(Scope::Admin))
}

/// Every mounted route and what it requires.
///
/// * **Configuration is `Admin` in both directions.** Even the narrowed public
///   projection controls process launch and resource limits.
/// * **Health and status are `Read`, not public.** They report the workspace
///   directory name and id. `/livez` and `/readyz` exist precisely so probes
///   never need that.
pub static ROUTE_RULES: &[RouteRule] = &[
    read("GET", "/api/v1/license"),
    admin("PUT", "/api/v1/license"),
    admin("DELETE", "/api/v1/license"),
    read("GET", "/api/v1/targets"),
    admin("POST", "/api/v1/targets"),
    admin("PUT", "/api/v1/targets/{name}"),
    admin("DELETE", "/api/v1/targets/{name}"),
    admin("POST", "/api/v1/targets/{name}/probe"),
    read("GET", "/api/v1/profiles"),
    admin("POST", "/api/v1/profiles"),
    read("GET", "/api/v1/profiles/{profile_id}"),
    admin("PATCH", "/api/v1/profiles/{profile_id}"),
    // --- Public: probes -----------------------------------------------------
    public("GET", "/livez"),
    public("GET", "/readyz"),
    // --- Public: obtaining a credential -------------------------------------
    public("GET", "/api/v1/auth/bootstrap"),
    public("POST", "/api/v1/auth/bootstrap"),
    public("POST", "/api/v1/auth/login"),
    public("POST", "/api/v1/auth/forgot-password"),
    public("POST", "/api/v1/auth/reset-password"),
    public("POST", "/api/v1/auth/device/code"),
    public("POST", "/api/v1/auth/token"),
    // --- Auth: authenticated session management -----------------------------
    read("GET", "/api/v1/auth/session"),
    read("GET", "/api/v1/auth/csrf"),
    write("POST", "/api/v1/auth/logout"),
    // Approving a device grants a credential, so it is administration.
    admin("POST", "/api/v1/auth/device/approve"),
    admin("GET", "/api/v1/auth/sessions"),
    admin("DELETE", "/api/v1/auth/sessions/{id}"),
    admin("GET", "/api/v1/auth/keys"),
    admin("POST", "/api/v1/auth/keys"),
    admin("DELETE", "/api/v1/auth/keys/{id}"),
    // --- Health / status ----------------------------------------------------
    read("GET", "/api/v1/health"),
    read("GET", "/api/v1/status"),
    read("GET", "/api/v1/sections"),
    read("GET", "/api/v1/integrations"),
    // --- Setup --------------------------------------------------------------
    read("GET", "/api/v1/setup/status"),
    read("GET", "/api/v1/setup/steps"),
    read("GET", "/api/v1/setup/collections"),
    admin("POST", "/api/v1/setup/initialize"),
    // --- Issue types --------------------------------------------------------
    read("GET", "/api/v1/issuetypes"),
    write("POST", "/api/v1/issuetypes"),
    read("GET", "/api/v1/issuetypes/{key}"),
    write("PUT", "/api/v1/issuetypes/{key}"),
    write("DELETE", "/api/v1/issuetypes/{key}"),
    read("GET", "/api/v1/issuetypes/{key}/document"),
    read("GET", "/api/v1/issuetypes/{key}/steps"),
    read("GET", "/api/v1/issuetypes/{key}/steps/{step_name}"),
    write("PUT", "/api/v1/issuetypes/{key}/steps/{step_name}"),
    read("GET", "/api/v1/issuetypes/{key}/workflow-preview"),
    // --- Collections --------------------------------------------------------
    read("GET", "/api/v1/collections"),
    read("GET", "/api/v1/collections/active"),
    read("GET", "/api/v1/collections/{name}"),
    write("PUT", "/api/v1/collections/{name}/activate"),
    // --- Queue --------------------------------------------------------------
    read("GET", "/api/v1/queue/kanban"),
    read("GET", "/api/v1/queue/status"),
    write("POST", "/api/v1/queue/pause"),
    write("POST", "/api/v1/queue/resume"),
    // Sync reaches out to a third-party provider with stored credentials and
    // writes tickets, so it is more than a queue mutation.
    execute("POST", "/api/v1/queue/sync"),
    execute("POST", "/api/v1/queue/sync/{provider}/{project_key}"),
    // --- Agents -------------------------------------------------------------
    read("GET", "/api/v1/agents/active"),
    read("GET", "/api/v1/agents/{agent_id}"),
    write("POST", "/api/v1/agents/{agent_id}/approve"),
    write("POST", "/api/v1/agents/{agent_id}/reject"),
    // Focus shells out to the session multiplexer binary.
    execute("POST", "/api/v1/agents/{agent_id}/focus"),
    // --- Projects -----------------------------------------------------------
    read("GET", "/api/v1/projects"),
    write("POST", "/api/v1/projects/{name}/assess"),
    // --- Tickets ------------------------------------------------------------
    read("GET", "/api/v1/tickets/{id}"),
    write("POST", "/api/v1/tickets"),
    write("PUT", "/api/v1/tickets/{id}/status"),
    write("POST", "/api/v1/alerts"),
    // --- Launch -------------------------------------------------------------
    execute("POST", "/api/v1/tickets/{id}/launch"),
    execute("POST", "/api/v1/tickets/{id}/steps/{step}/complete"),
    // --- Workflow export ----------------------------------------------------
    read("POST", "/api/v1/tickets/{id}/workflow-export"),
    read("GET", "/api/v1/workflow-formats"),
    // --- Kanban -------------------------------------------------------------
    read("GET", "/api/v1/kanban/providers"),
    read("GET", "/api/v1/kanban/{provider}/{project_key}/issuetypes"),
    read("GET", "/api/v1/kanban/{provider}/{project_key}/statuses"),
    write(
        "POST",
        "/api/v1/kanban/{provider}/{project_key}/issuetypes/sync",
    ),
    // Onboarding takes live credentials and calls the provider with them.
    execute("POST", "/api/v1/kanban/validate"),
    execute("POST", "/api/v1/kanban/projects"),
    execute("POST", "/api/v1/kanban/statuses"),
    // Writing provider config and setting process env are administration.
    admin("PUT", "/api/v1/kanban/config"),
    admin("POST", "/api/v1/kanban/session-env"),
    // --- Git onboarding -----------------------------------------------------
    read("GET", "/api/v1/git/providers"),
    execute("POST", "/api/v1/git/validate"),
    admin("PUT", "/api/v1/git/config"),
    admin("POST", "/api/v1/git/session-env"),
    // --- Skills / LLM tools -------------------------------------------------
    read("GET", "/api/v1/skills"),
    read("GET", "/api/v1/llm-tools"),
    read("GET", "/api/v1/llm-tools/default"),
    admin("PUT", "/api/v1/llm-tools/default"),
    // --- Delegators (command templates => administration) -------------------
    read("GET", "/api/v1/delegators"),
    admin("POST", "/api/v1/delegators"),
    admin("POST", "/api/v1/delegators/from-tool"),
    admin("POST", "/api/v1/delegators/import-profile"),
    read("GET", "/api/v1/delegators/{name}/profile"),
    read("GET", "/api/v1/delegators/{name}"),
    admin("PUT", "/api/v1/delegators/{name}"),
    admin("DELETE", "/api/v1/delegators/{name}"),
    // --- Model servers ------------------------------------------------------
    read("GET", "/api/v1/model-servers"),
    admin("POST", "/api/v1/model-servers"),
    read("GET", "/api/v1/model-servers/kinds"),
    read("GET", "/api/v1/model-servers/kinds/{slug}/models"),
    read("GET", "/api/v1/model-servers/{name}"),
    admin("PUT", "/api/v1/model-servers/{name}"),
    admin("DELETE", "/api/v1/model-servers/{name}"),
    // Probing makes an outbound request carrying the provider API key.
    execute("GET", "/api/v1/model-servers/{name}/models"),
    // --- Configuration ------------------------------------------------------
    admin("GET", "/api/v1/configuration"),
    admin("PATCH", "/api/v1/configuration"),
    read("GET", "/api/v1/execution-targets"),
    // --- MCP ----------------------------------------------------------------
    read("GET", "/api/v1/mcp/descriptor"),
    execute("GET", "/api/v1/mcp/sse"),
    execute("POST", "/api/v1/mcp/message"),
];

/// Look up the requirement for a matched route, if the table knows it.
///
/// Returns `None` for an unknown route. Callers must treat that as **deny**:
/// an unclassified route is a bug, and failing closed keeps it from being an
/// exploitable one.
pub fn required_access(method: &str, matched_path: &str) -> Option<Access> {
    ROUTE_RULES
        .iter()
        .find(|r| r.method == method && r.path == matched_path)
        .map(|r| r.access)
}

/// An authenticated caller.
#[derive(Debug, Clone)]
pub struct Principal {
    /// Account name; always `admin` today.
    pub subject: String,
    /// Scopes this credential carries.
    pub scopes: Vec<Scope>,
    /// How the caller authenticated.
    pub kind: PrincipalKind,
    /// When the presented credential expires, if it has a fixed deadline.
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Session id, when the caller presented a browser session cookie.
    pub session_id: Option<String>,
    /// For an agent callback token: the ticket it may report on.
    pub ticket_id: Option<String>,
    /// For an agent callback token: the step it may report on.
    pub step: Option<String>,
}

impl Principal {
    /// Whether this principal holds `scope`.
    ///
    /// Membership is explicit: scopes do not imply one another, so `Admin` does
    /// not satisfy a `Read` requirement unless it was also granted.
    pub fn has_scope(&self, scope: Scope) -> bool {
        self.scopes.contains(&scope)
    }

    /// A local loopback process, or the TUI in-process: full authority.
    pub fn local(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            scopes: Scope::ALL.to_vec(),
            kind: PrincipalKind::LocalProcess,
            expires_at: None,
            session_id: None,
            ticket_id: None,
            step: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_route_table_has_no_duplicate_entries() {
        // A duplicate is ambiguous: `required_access` returns the first match,
        // so a later, stricter entry would be silently unreachable.
        let mut seen = HashSet::new();
        for r in ROUTE_RULES {
            assert!(
                seen.insert((r.method, r.path)),
                "duplicate ROUTE_RULES entry for {} {}",
                r.method,
                r.path
            );
        }
    }

    #[test]
    fn test_public_routes_are_exactly_the_documented_allowlist() {
        // Widening this set is the single highest-impact mistake available in
        // this file, so it is pinned literally rather than derived.
        let public: HashSet<(&str, &str)> = ROUTE_RULES
            .iter()
            .filter(|r| r.access == Access::Public)
            .map(|r| (r.method, r.path))
            .collect();

        let expected: HashSet<(&str, &str)> = [
            ("GET", "/livez"),
            ("GET", "/readyz"),
            ("GET", "/api/v1/auth/bootstrap"),
            ("POST", "/api/v1/auth/bootstrap"),
            ("POST", "/api/v1/auth/login"),
            ("POST", "/api/v1/auth/forgot-password"),
            ("POST", "/api/v1/auth/reset-password"),
            ("POST", "/api/v1/auth/device/code"),
            ("POST", "/api/v1/auth/token"),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            public, expected,
            "the public route allowlist changed - this is a security boundary, \
             not a routing detail. Update the threat model and docs/security/ too."
        );
    }

    #[test]
    fn test_health_and_status_are_not_public() {
        // They disclose the workspace directory name and id; /livez and /readyz
        // exist so probes never need them.
        for path in ["/api/v1/health", "/api/v1/status"] {
            assert_eq!(
                required_access("GET", path),
                Some(Access::Scoped(Scope::Read)),
                "{path} must require a credential"
            );
        }
    }

    #[test]
    fn test_configuration_requires_admin_in_both_directions() {
        // Reading returns every model-server URL and delegator command template.
        assert_eq!(
            required_access("GET", "/api/v1/configuration"),
            Some(Access::Scoped(Scope::Admin))
        );
        assert_eq!(
            required_access("PATCH", "/api/v1/configuration"),
            Some(Access::Scoped(Scope::Admin))
        );
    }

    #[test]
    fn test_process_launching_and_outbound_probes_require_execute() {
        for (method, path) in [
            ("POST", "/api/v1/tickets/{id}/launch"),
            ("POST", "/api/v1/tickets/{id}/steps/{step}/complete"),
            ("POST", "/api/v1/agents/{agent_id}/focus"),
            ("GET", "/api/v1/model-servers/{name}/models"),
            ("GET", "/api/v1/mcp/sse"),
            ("POST", "/api/v1/mcp/message"),
        ] {
            assert_eq!(
                required_access(method, path),
                Some(Access::Scoped(Scope::Execute)),
                "{method} {path} launches a process or makes an outbound request"
            );
        }
    }

    #[test]
    fn test_unknown_route_is_unclassified_so_callers_fail_closed() {
        assert_eq!(required_access("GET", "/api/v1/not-a-route"), None);
        assert_eq!(required_access("DELETE", "/api/v1/health"), None);
    }

    #[test]
    fn test_scopes_do_not_imply_one_another() {
        let p = Principal {
            subject: "admin".to_string(),
            scopes: vec![Scope::Admin],
            kind: PrincipalKind::AccessToken,
            expires_at: None,
            session_id: None,
            ticket_id: None,
            step: None,
        };
        assert!(p.has_scope(Scope::Admin));
        assert!(
            !p.has_scope(Scope::Read),
            "Admin must not implicitly satisfy Read; grants are explicit"
        );
    }

    #[test]
    fn test_local_principal_holds_every_scope() {
        let p = Principal::local("admin");
        for scope in Scope::ALL {
            assert!(p.has_scope(scope));
        }
        assert_eq!(p.kind, PrincipalKind::LocalProcess);
    }
}
