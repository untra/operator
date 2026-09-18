//! Every mounted route must declare what it requires.
//!
//! Authorization is decided by matching a request against
//! `operator::auth::scope::ROUTE_RULES`. A route that is mounted but missing
//! from that table is denied at runtime - which fails safe, but as a 401 on a
//! working endpoint rather than as anything a developer would notice locally.
//! This suite turns that into a build failure instead, and pins the public
//! allowlist so widening it cannot happen quietly.

use std::collections::BTreeSet;

use operator::auth::scope::{Access, ROUTE_RULES};

/// Routes mounted outside the OpenAPI-documented router.
///
/// The MCP SSE/message pair is config-gated on `[mcp].http_enabled` and carries no `#[utoipa::path]`, so it never appears in the spec
const UNDOCUMENTED_MOUNTED_ROUTES: &[(&str, &str)] =
    &[("GET", "/api/v1/mcp/sse"), ("POST", "/api/v1/mcp/message")];

// `/api/v1/profiles/{profile_id}/{*path}` is deliberately absent from both
// lists. It is a dispatcher, not an endpoint: it rewrites the URI and
// re-dispatches into the same router these rules already cover, so the request
// is authorized against the rule for the route it actually resolves to. A
// `ROUTE_RULES` entry for the wildcard would never be consulted, and an
// unconsulted rule is a lie about the surface.
//
// What that indirection *could* do is stop enforcing, so it is covered
// behaviourally instead - see `tests/premium_http_contract.rs`:
// `a_configuration_qualified_route_still_requires_authentication` and
// `a_configuration_qualified_route_enforces_entitlement`.

/// The complete set of routes reachable without a credential.
const EXPECTED_PUBLIC: &[(&str, &str)] = &[
    // Kubernetes probes - no workspace metadata.
    ("GET", "/livez"),
    ("GET", "/readyz"),
    // The endpoints needed to *obtain* a credential.
    ("GET", "/api/v1/auth/bootstrap"),
    ("POST", "/api/v1/auth/bootstrap"),
    ("POST", "/api/v1/auth/login"),
    ("POST", "/api/v1/auth/forgot-password"),
    ("POST", "/api/v1/auth/reset-password"),
    ("POST", "/api/v1/auth/device/code"),
    ("POST", "/api/v1/auth/token"),
];

/// Every `(METHOD, path)` the generated OpenAPI spec documents.
fn documented_routes() -> BTreeSet<(String, String)> {
    let spec = operator::rest::ApiDoc::json().expect("generate OpenAPI spec");
    let parsed: serde_json::Value = serde_json::from_str(&spec).expect("spec is JSON");
    let paths = parsed
        .get("paths")
        .and_then(|p| p.as_object())
        .expect("spec has paths");

    let mut out = BTreeSet::new();
    for (path, item) in paths {
        let Some(methods) = item.as_object() else {
            continue;
        };
        for method in methods.keys() {
            // Skip OpenAPI path-level keys that are not operations.
            if matches!(method.as_str(), "parameters" | "summary" | "description") {
                continue;
            }
            out.insert((method.to_uppercase(), path.clone()));
        }
    }
    out
}

fn table_routes() -> BTreeSet<(String, String)> {
    ROUTE_RULES
        .iter()
        .map(|r| (r.method.to_string(), r.path.to_string()))
        .collect()
}

#[test]
fn test_every_documented_route_declares_a_scope() {
    let table = table_routes();
    let missing: Vec<_> = documented_routes()
        .into_iter()
        .filter(|r| !table.contains(r))
        .collect();

    assert!(
        missing.is_empty(),
        "these routes are mounted but absent from ROUTE_RULES, so they would be \
         denied at runtime. Classify each one in src/auth/scope.rs:\n{missing:#?}"
    );
}

#[test]
fn test_undocumented_mounted_routes_declare_a_scope() {
    // The MCP transport pair never reaches the OpenAPI spec, so the check above
    // cannot see it. It executes tools, which makes it the last thing that
    // should slip through unclassified.
    let table = table_routes();
    for (method, path) in UNDOCUMENTED_MOUNTED_ROUTES {
        assert!(
            table.contains(&(method.to_string(), path.to_string())),
            "{method} {path} is mounted by build_router but has no ROUTE_RULES entry"
        );
    }
}

#[test]
fn test_route_table_has_no_entries_for_routes_that_do_not_exist() {
    // A stale entry is not a security hole, but it is a lie about the surface
    // and it hides a genuine miss behind noise.
    let documented = documented_routes();
    let undocumented: BTreeSet<(String, String)> = UNDOCUMENTED_MOUNTED_ROUTES
        .iter()
        .map(|(m, p)| ((*m).to_string(), (*p).to_string()))
        .collect();

    let stale: Vec<_> = table_routes()
        .into_iter()
        .filter(|r| !documented.contains(r) && !undocumented.contains(r))
        .collect();

    assert!(
        stale.is_empty(),
        "ROUTE_RULES names routes that are not mounted - remove them:\n{stale:#?}"
    );
}

#[test]
fn test_public_routes_are_exactly_the_expected_allowlist() {
    let actual: BTreeSet<(String, String)> = ROUTE_RULES
        .iter()
        .filter(|r| r.access == Access::Public)
        .map(|r| (r.method.to_string(), r.path.to_string()))
        .collect();
    let expected: BTreeSet<(String, String)> = EXPECTED_PUBLIC
        .iter()
        .map(|(m, p)| ((*m).to_string(), (*p).to_string()))
        .collect();

    let added: Vec<_> = actual.difference(&expected).collect();
    let removed: Vec<_> = expected.difference(&actual).collect();

    assert!(
        added.is_empty(),
        "these routes became public. That is a change to the security boundary, \
         not a routing detail - update docs/security/ and this allowlist \
         deliberately:\n{added:#?}"
    );
    assert!(
        removed.is_empty(),
        "these routes stopped being public; bootstrap or login may now be \
         unreachable:\n{removed:#?}"
    );
}

#[test]
fn test_the_endpoints_that_disclose_workspace_identity_are_not_public() {
    // /api/v1/health and /status report the workspace directory name and id.
    // /livez and /readyz exist so a probe never needs them.
    let public: BTreeSet<(String, String)> = ROUTE_RULES
        .iter()
        .filter(|r| r.access == Access::Public)
        .map(|r| (r.method.to_string(), r.path.to_string()))
        .collect();

    for path in ["/api/v1/health", "/api/v1/status"] {
        assert!(
            !public.contains(&("GET".to_string(), path.to_string())),
            "{path} discloses workspace identity and must not be public"
        );
    }
}
