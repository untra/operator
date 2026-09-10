//! OpenAPI specification builder using utoipa.

use std::collections::BTreeSet;

use utoipa::openapi::{
    content::Content,
    extensions::Extensions,
    header::Header,
    path::{Operation, Parameter, ParameterIn},
    response::Response,
    schema::{Object, Type},
    security::{
        ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityRequirement, SecurityScheme,
    },
    Ref, RefOr, Required,
};
use utoipa::{Modify, OpenApi};

use crate::auth::scope::{required_access, Access};
use crate::mcp::descriptor::McpDescriptorResponse;
use crate::rest::dto::{
    AccessKeyListResponse, AccessKeySummary, ActiveAgentResponse, ActiveAgentsResponse,
    AgentDetailResponse, AssessTicketResponse, BootstrapState, BootstrapStatusResponse,
    BootstrapSubmitRequest, BootstrapSubmitResponse, CollectionResponse, CreateAccessKeyRequest,
    CreateAccessKeyResponse, CreateAlertRequest, CreateAlertResponse,
    CreateDelegatorFromToolRequest, CreateDelegatorRequest, CreateFieldRequest,
    CreateIssueTypeRequest, CreateModelServerRequest, CreateStepRequest, CreateTicketRequest,
    CreateTicketResponse, CsrfTokenResponse, CurrentSessionResponse, DefaultLlmResponse,
    DelegatorLaunchConfigDto, DelegatorResponse, DelegatorsResponse, DeviceApprovalRequest,
    DeviceApprovalResponse, DeviceAuthorizationRequest, DeviceAuthorizationResponse, DeviceSummary,
    ExternalIssueTypeSummary, FieldResponse, HealthResponse, IntegrationCatalogEntryDto,
    IssueTypeResponse, IssueTypeSummary, KanbanBoardResponse, KanbanIssueTypeResponse,
    KanbanProviderCatalogEntry, KanbanSyncResponse, KanbanTicketCard, LaunchTicketRequest,
    LaunchTicketResponse, ListKanbanProjectsRequest, ListKanbanProjectsResponse,
    ListKanbanStatusesRequest, ListKanbanStatusesResponse, LoginRequest, LoginResponse,
    LogoutResponse, ModelEntry, ModelServerKindEntry, ModelServerModelsResponse,
    ModelServerResponse, ModelServersResponse, NextStepInfo, OAuthErrorCode, OAuthErrorResponse,
    OperatorOutput, PrincipalKind, ProjectSummary, QueueByType, QueueControlResponse,
    QueueStatusResponse, RejectReviewRequest, ReviewResponse, RevokeAccessKeyResponse, Scope,
    SectionDto, SectionRowDto, SessionListResponse, SessionSummary, SetDefaultLlmRequest,
    SetKanbanSessionEnvRequest, SetKanbanSessionEnvResponse, SkillEntry, SkillsResponse,
    StatusResponse, StepCompleteRequest, StepCompleteResponse, StepResponse,
    SyncKanbanIssueTypesResponse, TicketDetailResponse, TokenRequest, TokenResponse,
    UpdateIssueTypeRequest, UpdateModelServerRequest, UpdateStepRequest, UpdateTicketStatusRequest,
    UpdateTicketStatusResponse, ValidateKanbanCredentialsRequest,
    ValidateKanbanCredentialsResponse, WorkflowExportResponse, WorkflowFormatDto, WorkflowHintsDto,
    WorkflowPreviewResponse, WriteKanbanConfigRequest, WriteKanbanConfigResponse,
};
// AgentProfile interchange types live in `crate::config`, not `rest::dto`.
use crate::config::{AgentProfile, DelegatorLaunchConfig, RemoteAgentRef, XOperator};
use crate::rest::error::ErrorResponse;

/// OpenAPI documentation for the Operator REST API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Operator API",
        // NOTE: no `version` here on purpose. The version is stamped at runtime
        // from `CARGO_PKG_VERSION` in `crate::rest::openapi_spec`, the single
        // source of truth, so it always matches the published release and
        // `/api/v1/health`. A hardcoded literal here would silently go stale.
        description = "REST API for managing and running Operator server system.",
        license(name = "MIT"),
        contact(
            name = "untra",
            url = "https://github.com/untra/operator"
        )
    ),
    // NOTE: `paths(...)` is intentionally omitted. Routes self-register in the
    // OpenAPI spec when mounted via `utoipa_axum::routes!` in
    // `crate::rest::build_router` — mounting a route *is* documenting it, so the
    // two can no longer drift. See `crate::rest::openapi_spec`.
    components(
        schemas(
            // Response types
            HealthResponse,
            StatusResponse,
            SectionDto,
            SectionRowDto,
            IntegrationCatalogEntryDto,
            crate::integrations::SupportStatus,
            IssueTypeResponse,
            IssueTypeSummary,
            FieldResponse,
            StepResponse,
            CollectionResponse,
            WorkflowHintsDto,
            LaunchTicketResponse,
            ErrorResponse,
            // Request types
            CreateIssueTypeRequest,
            UpdateIssueTypeRequest,
            CreateFieldRequest,
            CreateStepRequest,
            UpdateStepRequest,
            LaunchTicketRequest,
            // Skills types
            SkillEntry,
            SkillsResponse,
            // Delegator types
            DelegatorResponse,
            DelegatorsResponse,
            CreateDelegatorRequest,
            CreateDelegatorFromToolRequest,
            DelegatorLaunchConfigDto,
            // AgentProfile interchange types
            AgentProfile,
            XOperator,
            RemoteAgentRef,
            DelegatorLaunchConfig,
            // Model server types
            ModelServerResponse,
            ModelServersResponse,
            CreateModelServerRequest,
            UpdateModelServerRequest,
            ModelServerKindEntry,
            ModelEntry,
            ModelServerModelsResponse,
            // LLM tools types
            SetDefaultLlmRequest,
            DefaultLlmResponse,
            // Ticket types
            TicketDetailResponse,
            UpdateTicketStatusRequest,
            UpdateTicketStatusResponse,
            CreateTicketRequest,
            CreateTicketResponse,
            CreateAlertRequest,
            CreateAlertResponse,
            // Workflow export types
            WorkflowExportResponse,
            WorkflowPreviewResponse,
            WorkflowFormatDto,
            crate::workflow_gen::WorkflowFormat,
            // MCP types
            McpDescriptorResponse,
            // Queue types
            KanbanBoardResponse,
            KanbanTicketCard,
            QueueStatusResponse,
            QueueByType,
            QueueControlResponse,
            KanbanSyncResponse,
            // Agent types
            ActiveAgentsResponse,
            ActiveAgentResponse,
            AgentDetailResponse,
            ReviewResponse,
            RejectReviewRequest,
            OperatorOutput,
            // Project types
            ProjectSummary,
            AssessTicketResponse,
            // Launch step-completion types
            StepCompleteRequest,
            StepCompleteResponse,
            NextStepInfo,
            // Kanban provider types
            ExternalIssueTypeSummary,
            KanbanIssueTypeResponse,
            SyncKanbanIssueTypesResponse,
            KanbanProviderCatalogEntry,
            // Kanban onboarding types
            ValidateKanbanCredentialsRequest,
            ValidateKanbanCredentialsResponse,
            ListKanbanProjectsRequest,
            ListKanbanProjectsResponse,
            ListKanbanStatusesRequest,
            ListKanbanStatusesResponse,
            crate::config::kanban::KanbanStatusMapping,
            WriteKanbanConfigRequest,
            WriteKanbanConfigResponse,
            SetKanbanSessionEnvRequest,
            SetKanbanSessionEnvResponse,
            // Authentication
            Scope,
            PrincipalKind,
            BootstrapState,
            BootstrapStatusResponse,
            BootstrapSubmitRequest,
            BootstrapSubmitResponse,
            LoginRequest,
            LoginResponse,
            LogoutResponse,
            CurrentSessionResponse,
            CsrfTokenResponse,
            SessionSummary,
            DeviceSummary,
            SessionListResponse,
            DeviceAuthorizationRequest,
            DeviceAuthorizationResponse,
            DeviceApprovalRequest,
            DeviceApprovalResponse,
            TokenRequest,
            TokenResponse,
            OAuthErrorCode,
            OAuthErrorResponse,
            CreateAccessKeyRequest,
            CreateAccessKeyResponse,
            AccessKeySummary,
            AccessKeyListResponse,
            RevokeAccessKeyResponse,
        )
    ),
    modifiers(&SecurityAddon),
    paths(
        crate::mcp::transport::sse_handler,
        crate::mcp::transport::message_handler,
    ),
    tags(
        (name = "Health", description = "Health check and status endpoints"),
        (name = "Status", description = "Canonical status sections (TUI / VS Code parity)"),
        (name = "Issue Types", description = "Issue type CRUD operations"),
        (name = "Steps", description = "Step management within issue types"),
        (name = "Collections", description = "Issue type collection management"),
        (name = "Tickets", description = "Ticket CRUD and status management"),
        (name = "Launch", description = "Ticket launch operations"),
        (name = "Workflow", description = "Export tickets to Claude dynamic workflows"),
        (name = "Skills", description = "Skill discovery across LLM tools"),
        (name = "Delegators", description = "Agent delegator CRUD operations"),
        (name = "ModelServers", description = "Model server (ollama, openai-compat, etc.) CRUD operations"),
        (name = "MCP", description = "Model Context Protocol integration"),
        (name = "Queue", description = "Ticket queue board, status, and control"),
        (name = "Agents", description = "Active agent tracking and review actions"),
        (name = "Projects", description = "Project discovery and ticket assessment"),
        (name = "Configuration", description = "Operator configuration read/write"),
        (name = "Kanban", description = "Kanban provider issue types and onboarding"),
        (name = "Auth", description = "Bootstrap, sessions, OAuth device flow, and access keys"),
    )
)]
pub struct ApiDoc;

/// Registers the two accepted authentication schemes on the generated spec.
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // `components` is always present: the derive registers schemas above.
        let components = openapi
            .components
            .as_mut()
            .expect("ApiDoc registers component schemas, so components exists");

        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some(
                        "Short-lived signed access token. Obtain one at the token \
                         endpoint with a refresh token or a service access key.",
                    ))
                    .build(),
            ),
        );

        components.add_security_scheme(
            "sessionCookie",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
                "__Host-operator_session",
                "Opaque server-side browser session. Cookie-authenticated \
                mutations additionally require a CSRF token and a matching Origin.",
            ))),
        );

        let mut unauthorized = error_response("Unauthorized");
        unauthorized.headers.insert(
            "WWW-Authenticate".to_string(),
            response_header(
                "Authentication challenge naming the accepted schemes.",
                Type::String,
            ),
        );
        components
            .responses
            .insert(UNAUTHORIZED_RESPONSE_NAME.to_string(), unauthorized.into());
        components.responses.insert(
            FORBIDDEN_RESPONSE_NAME.to_string(),
            error_response("Forbidden").into(),
        );
    }
}

const ERROR_SCHEMA_NAME: &str = "ErrorResponse";
const FORBIDDEN_RESPONSE_NAME: &str = "Forbidden";
const JSON_MEDIA_TYPE: &str = "application/json";
const UNAUTHORIZED_RESPONSE_NAME: &str = "Unauthorized";

fn error_content() -> Content {
    Content::new(Some(Ref::from_schema_name(ERROR_SCHEMA_NAME)))
}

fn error_response(description: &str) -> Response {
    let mut response = Response::new(description);
    response
        .content
        .insert(JSON_MEDIA_TYPE.to_string(), error_content());
    response
}

fn response_header(description: &str, schema_type: Type) -> Header {
    let mut header = Header::new(Object::with_type(schema_type));
    header.description = Some(description.to_string());
    header
}

fn document_error_responses(operation: &mut Operation) {
    for (status, response) in &mut operation.responses.responses {
        if !(status.starts_with('4') || status.starts_with('5')) {
            continue;
        }

        let RefOr::T(response) = response else {
            continue;
        };
        response
            .content
            .entry(JSON_MEDIA_TYPE.to_string())
            .or_insert_with(error_content);

        if status == "401" {
            response.headers.insert(
                "WWW-Authenticate".to_string(),
                response_header(
                    "Authentication challenge naming the accepted schemes.",
                    Type::String,
                ),
            );
        }
        if status == "429" {
            response.headers.insert(
                "Retry-After".to_string(),
                response_header(
                    "Seconds the client must wait before retrying.",
                    Type::Integer,
                ),
            );
        }
    }
}

fn document_set_cookie(operation: &mut Operation, description: &str) {
    let Some(RefOr::T(response)) = operation.responses.responses.get_mut("200") else {
        return;
    };
    response.headers.insert(
        "Set-Cookie".to_string(),
        response_header(description, Type::String),
    );
}

fn collect_schema_refs(value: &serde_json::Value, refs: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(name) = object
                .get("$ref")
                .and_then(serde_json::Value::as_str)
                .and_then(|reference| reference.strip_prefix("#/components/schemas/"))
            {
                refs.insert(name.to_string());
            }
            for nested in object.values() {
                collect_schema_refs(nested, refs);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_schema_refs(item, refs);
            }
        }
        _ => {}
    }
}

fn prune_unreachable_schemas(spec: &mut utoipa::openapi::OpenApi) {
    let Some(components) = spec.components.as_mut() else {
        return;
    };
    let schemas = components.schemas.clone();
    let mut reachable = BTreeSet::new();
    collect_schema_refs(
        &serde_json::to_value(&spec.paths).expect("serialize OpenAPI paths"),
        &mut reachable,
    );
    collect_schema_refs(
        &serde_json::to_value(&components.responses).expect("serialize OpenAPI responses"),
        &mut reachable,
    );

    let mut pending: Vec<String> = reachable.iter().cloned().collect();
    while let Some(name) = pending.pop() {
        let Some(schema) = schemas.get(&name) else {
            continue;
        };
        let mut nested = BTreeSet::new();
        collect_schema_refs(
            &serde_json::to_value(schema).expect("serialize component schema"),
            &mut nested,
        );
        for reference in nested {
            if reachable.insert(reference.clone()) {
                pending.push(reference);
            }
        }
    }

    components
        .schemas
        .retain(|name, _| reachable.contains(name));
}

fn csrf_parameter() -> Parameter {
    let mut parameter = Parameter::new(crate::rest::middleware::auth::CSRF_HEADER);
    parameter.parameter_in = ParameterIn::Header;
    parameter.required = Required::False;
    parameter.description = Some(
        "Required for cookie-authenticated mutations; omit when using bearer authentication."
            .to_string(),
    );
    parameter.schema = Some(Object::with_type(Type::String).into());
    parameter
}

fn enrich_operation(method: &str, path: &str, operation: Option<&mut Operation>) {
    let Some(operation) = operation else {
        return;
    };
    let access = required_access(method, path)
        .unwrap_or_else(|| panic!("documented route {method} {path} is missing from ROUTE_RULES"));

    match access {
        Access::Public => operation.security = Some(Vec::new()),
        Access::Scoped(scope) => {
            operation.security = Some(vec![
                SecurityRequirement::new("bearerAuth", Vec::<String>::new()),
                SecurityRequirement::new("sessionCookie", Vec::<String>::new()),
            ]);
            operation
                .extensions
                .get_or_insert_with(Extensions::default)
                .insert(
                    "x-operator-scope".to_string(),
                    serde_json::json!(scope.as_str()),
                );
            operation
                .responses
                .responses
                .entry("401".to_string())
                .or_insert_with(|| Ref::from_response_name(UNAUTHORIZED_RESPONSE_NAME).into());
            operation
                .responses
                .responses
                .entry("403".to_string())
                .or_insert_with(|| Ref::from_response_name(FORBIDDEN_RESPONSE_NAME).into());

            if !matches!(method, "GET" | "HEAD" | "OPTIONS") {
                operation
                    .parameters
                    .get_or_insert_with(Vec::new)
                    .push(csrf_parameter());
            }
        }
    }

    document_error_responses(operation);

    match (method, path) {
        ("POST", "/api/v1/auth/login") => {
            document_set_cookie(operation, "Sets the opaque HttpOnly browser session cookie");
        }
        ("POST", "/api/v1/auth/logout") => {
            document_set_cookie(operation, "Expires the browser session cookie");
        }
        _ => {}
    }
}

/// Apply the external contract metadata after `utoipa_axum` has merged paths.
pub fn apply_contract_metadata(mut openapi: utoipa::openapi::OpenApi) -> utoipa::openapi::OpenApi {
    for (path, item) in &mut openapi.paths.paths {
        enrich_operation("GET", path, item.get.as_mut());
        enrich_operation("PUT", path, item.put.as_mut());
        enrich_operation("POST", path, item.post.as_mut());
        enrich_operation("DELETE", path, item.delete.as_mut());
        enrich_operation("PATCH", path, item.patch.as_mut());
        enrich_operation("OPTIONS", path, item.options.as_mut());
        enrich_operation("HEAD", path, item.head.as_mut());
        enrich_operation("TRACE", path, item.trace.as_mut());
    }

    prune_unreachable_schemas(&mut openapi);
    openapi
}

impl ApiDoc {
    /// Generate the OpenAPI specification as a JSON string.
    ///
    /// Sourced from the fully-mounted router via [`crate::rest::openapi_spec`]
    /// so every live route appears in the spec (the bare `ApiDoc` derive carries
    /// only info/components/tags — paths self-register on mount). `openapi_spec`
    /// also stamps `info.version` from `CARGO_PKG_VERSION`, so it stays in sync
    /// with the release version and `/api/v1/health`.
    pub fn json() -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&crate::rest::openapi_spec())
    }

    /// Generate the OpenAPI specification as a YAML string.
    ///
    /// The version is stamped from `CARGO_PKG_VERSION` by [`crate::rest::openapi_spec`].
    #[allow(dead_code)]
    pub fn yaml() -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&crate::rest::openapi_spec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed_spec() -> serde_json::Value {
        serde_json::from_str(&ApiDoc::json().expect("generate spec")).expect("spec is JSON")
    }

    #[test]
    fn test_openapi_spec_generates() {
        let spec = ApiDoc::json().expect("Failed to generate OpenAPI spec");
        assert!(spec.contains("Operator API"));
        assert!(spec.contains("/api/v1/health"));
        assert!(spec.contains("/api/v1/issuetypes"));
    }

    #[test]
    fn test_openapi_declares_both_security_schemes() {
        // The schemes are added by a `Modify` addon, which is easy to drop from
        // the derive without noticing — the spec still builds, just without any
        // way for a client to learn how to authenticate.
        let spec = ApiDoc::json().expect("generate spec");
        let parsed: serde_json::Value = serde_json::from_str(&spec).expect("spec is JSON");
        let schemes = parsed
            .get("components")
            .and_then(|c| c.get("securitySchemes"))
            .expect("spec must declare securitySchemes");

        let bearer = schemes.get("bearerAuth").expect("bearerAuth scheme");
        assert_eq!(
            bearer.get("scheme").and_then(|v| v.as_str()),
            Some("bearer")
        );
        assert_eq!(
            bearer.get("bearerFormat").and_then(|v| v.as_str()),
            Some("JWT")
        );

        let cookie = schemes.get("sessionCookie").expect("sessionCookie scheme");
        assert_eq!(cookie.get("in").and_then(|v| v.as_str()), Some("cookie"));
        assert_eq!(
            cookie.get("name").and_then(|v| v.as_str()),
            Some("__Host-operator_session"),
            "the cookie name must keep its __Host- prefix, which the browser enforces"
        );
    }

    #[test]
    fn test_openapi_security_matches_route_rules() {
        let spec = parsed_spec();
        for rule in crate::auth::scope::ROUTE_RULES {
            let operation = &spec["paths"][rule.path][rule.method.to_ascii_lowercase()];
            assert!(
                operation.is_object(),
                "{} {} must be documented",
                rule.method,
                rule.path
            );
            match rule.access {
                Access::Public => assert_eq!(operation["security"], serde_json::json!([])),
                Access::Scoped(scope) => {
                    assert_eq!(
                        operation["security"],
                        serde_json::json!([
                            { "bearerAuth": [] },
                            { "sessionCookie": [] }
                        ])
                    );
                    assert_eq!(operation["x-operator-scope"], scope.as_str());
                }
            }
        }
    }

    #[test]
    fn test_protected_mutations_document_csrf_header() {
        let spec = parsed_spec();
        for rule in crate::auth::scope::ROUTE_RULES {
            if matches!(rule.access, Access::Public)
                || matches!(rule.method, "GET" | "HEAD" | "OPTIONS")
            {
                continue;
            }
            let parameters = spec["paths"][rule.path][rule.method.to_ascii_lowercase()]
                ["parameters"]
                .as_array()
                .expect("protected mutation parameters");
            assert!(parameters.iter().any(|parameter| {
                parameter["name"] == crate::rest::middleware::auth::CSRF_HEADER
                    && parameter["in"] == "header"
                    && parameter["required"] == false
            }));
        }
    }

    #[test]
    fn test_error_responses_have_standard_bodies_and_headers() {
        let spec = parsed_spec();
        for item in spec["paths"].as_object().expect("paths").values() {
            for operation in item.as_object().expect("path item").values() {
                let Some(responses) = operation
                    .get("responses")
                    .and_then(|value| value.as_object())
                else {
                    continue;
                };
                for (status, response) in responses {
                    let response = response
                        .get("$ref")
                        .and_then(serde_json::Value::as_str)
                        .and_then(|reference| reference.rsplit('/').next())
                        .map_or(response, |name| &spec["components"]["responses"][name]);
                    if status.starts_with('4') || status.starts_with('5') {
                        assert!(
                            response["content"][JSON_MEDIA_TYPE]["schema"].is_object(),
                            "error response {status} must declare a JSON schema"
                        );
                    }
                    if status == "401" {
                        assert!(response["headers"]["WWW-Authenticate"].is_object());
                    }
                    if status == "429" {
                        assert!(response["headers"]["Retry-After"].is_object());
                    }
                }
            }
        }
    }

    #[test]
    fn test_only_operation_reachable_schemas_are_published() {
        let spec = parsed_spec();
        let schemas = spec["components"]["schemas"]
            .as_object()
            .expect("component schemas");
        let mut reachable = BTreeSet::new();
        collect_schema_refs(&spec["paths"], &mut reachable);
        collect_schema_refs(&spec["components"]["responses"], &mut reachable);
        let mut pending: Vec<_> = reachable.iter().cloned().collect();
        while let Some(name) = pending.pop() {
            let schema = schemas
                .get(&name)
                .unwrap_or_else(|| panic!("missing referenced schema {name}"));
            let mut nested = BTreeSet::new();
            collect_schema_refs(schema, &mut nested);
            for reference in nested {
                if reachable.insert(reference.clone()) {
                    pending.push(reference);
                }
            }
        }
        assert_eq!(schemas.keys().cloned().collect::<BTreeSet<_>>(), reachable);
    }

    #[test]
    fn test_configuration_and_token_contracts_are_explicit() {
        let spec = parsed_spec();
        let schemas = &spec["components"]["schemas"];
        assert!(schemas.get("Config").is_none());
        assert!(schemas["ConfigurationResponse"]["properties"]["agents"].is_object());
        assert!(schemas["UpdateConfigurationRequest"]["properties"]["launch"].is_object());
        assert_eq!(
            schemas["TokenRequest"]["oneOf"].as_array().map(Vec::len),
            Some(3)
        );
        for variant in schemas["TokenRequest"]["oneOf"]
            .as_array()
            .expect("token variants")
        {
            let credential = variant["properties"]
                .as_object()
                .expect("token properties")
                .iter()
                .find(|(name, _)| {
                    name.ends_with("token") || *name == "device_code" || *name == "access_key"
                })
                .map(|(_, schema)| schema)
                .expect("credential property");
            assert_eq!(credential["writeOnly"], true);
            assert_eq!(credential["format"], "password");
        }
    }

    #[test]
    fn test_openapi_registers_auth_schemas() {
        // Phase 2 ships the contract before any handler exists, so these types
        // are reachable only through `components(schemas(...))`. Dropping one
        // would silently remove it from the spec and from every generated client.
        let spec = ApiDoc::json().expect("generate spec");
        for schema in [
            "Scope",
            "BootstrapState",
            "BootstrapStatusResponse",
            "BootstrapSubmitRequest",
            "LoginRequest",
            "LoginResponse",
            "CurrentSessionResponse",
            "DeviceAuthorizationResponse",
            "TokenRequest",
            "TokenResponse",
            "OAuthErrorResponse",
            "CreateAccessKeyResponse",
            "AccessKeySummary",
            "SessionListResponse",
        ] {
            assert!(
                spec.contains(&format!("\"{schema}\"")),
                "spec should register the {schema} component schema"
            );
        }
    }

    #[test]
    fn test_openapi_has_all_tags() {
        let spec = ApiDoc::json().expect("Failed to generate OpenAPI spec");
        assert!(spec.contains("\"Health\""));
        assert!(spec.contains("\"Issue Types\""));
        assert!(spec.contains("\"Steps\""));
        assert!(spec.contains("\"Collections\""));
    }

    #[test]
    fn test_openapi_operation_ids_are_unique() {
        // Structural guard: utoipa derives operationId from the bare fn name, so
        // collisions (multiple `list` / `get_one` / `create`) silently produce
        // an invalid spec that breaks downstream codegen. Every `#[utoipa::path]`
        // sets an explicit `module_fn` operation_id; this asserts they stay
        // globally unique as routes are added.
        let spec: serde_json::Value =
            serde_json::from_str(&ApiDoc::json().expect("generate spec")).expect("parse spec");

        let mut ids: Vec<String> = Vec::new();
        let paths = spec["paths"].as_object().expect("paths object");
        for (path, item) in paths {
            let methods = item.as_object().expect("path item object");
            for (method, op) in methods {
                let oid = op
                    .get("operationId")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        panic!("{} {path} is missing an operationId", method.to_uppercase())
                    });
                ids.push(oid.to_string());
            }
        }

        let mut seen = std::collections::HashSet::new();
        let mut dups: Vec<&String> = ids.iter().filter(|id| !seen.insert(*id)).collect();
        dups.sort();
        dups.dedup();
        assert!(
            dups.is_empty(),
            "duplicate operationId(s) in OpenAPI spec: {dups:?}"
        );
        assert!(
            !ids.is_empty(),
            "expected at least one documented operation"
        );
    }

    #[test]
    fn test_openapi_includes_previously_undocumented_routes() {
        // Regression guard for the drift this migration fixed: these routes are
        // mounted by `build_router` and must appear in the generated spec.
        let spec = ApiDoc::json().expect("generate spec");
        for path in [
            "/api/v1/queue/kanban",
            "/api/v1/agents/active",
            "/api/v1/projects",
            "/api/v1/configuration",
            "/api/v1/kanban/validate",
            "/api/v1/tickets/{id}/steps/{step}/complete",
        ] {
            assert!(
                spec.contains(path),
                "spec should document the mounted route {path}"
            );
        }
    }

    #[test]
    fn test_openapi_documents_kanban_status_discovery_routes() {
        // Both status-discovery surfaces must be mounted + documented: the
        // ephemeral-creds onboarding POST and the stored-config GET.
        let spec = ApiDoc::json().expect("generate spec");
        assert!(spec.contains("/api/v1/kanban/statuses"));
        assert!(spec.contains("/api/v1/kanban/{provider}/{project_key}/statuses"));
        assert!(spec.contains("KanbanStatusMapping"));
    }

    #[test]
    fn test_openapi_version_matches_cargo() {
        let spec = ApiDoc::json().expect("Failed to generate OpenAPI spec");
        let cargo_version = env!("CARGO_PKG_VERSION");
        assert!(
            spec.contains(&format!("\"version\": \"{cargo_version}\"")),
            "OpenAPI version should match Cargo.toml version ({cargo_version}), but spec contains different version"
        );
    }

    #[test]
    fn test_served_openapi_spec_version_matches_cargo() {
        // The version served by swagger-ui (`/api-docs/openapi.json`) comes from
        // `openapi_spec()`, NOT `ApiDoc::json()`. Guard the served path directly
        // so a stale hardcoded literal in the derive can never resurface and make
        // swagger-ui disagree with `/api/v1/health`.
        let spec = crate::rest::openapi_spec();
        assert_eq!(
            spec.info.version,
            env!("CARGO_PKG_VERSION"),
            "served OpenAPI spec version must match the compiled release version"
        );
    }

    #[test]
    fn test_version_file_matches_cargo_version() {
        // CI computes the release version, then writes it to BOTH `VERSION` and
        // `Cargo.toml` before building (see .github/workflows/build.yaml). This
        // asserts they never drift, so the version every API surface reports
        // (`CARGO_PKG_VERSION`) is exactly the published release version.
        let version_file = concat!(env!("CARGO_MANIFEST_DIR"), "/VERSION");
        let file_version = std::fs::read_to_string(version_file)
            .expect("VERSION file should exist at the crate root");
        assert_eq!(
            file_version.trim(),
            env!("CARGO_PKG_VERSION"),
            "VERSION file must match Cargo.toml version so the reported API version matches the published release"
        );
    }
}
