//! OpenAPI specification builder using utoipa.

use utoipa::OpenApi;

use crate::mcp::descriptor::McpDescriptorResponse;
use crate::rest::dto::{
    ActiveAgentResponse, ActiveAgentsResponse, AgentDetailResponse, AssessTicketResponse,
    CollectionResponse, CreateDelegatorFromToolRequest, CreateDelegatorRequest, CreateFieldRequest,
    CreateIssueTypeRequest, CreateModelServerRequest, CreateStepRequest, DefaultLlmResponse,
    DelegatorLaunchConfigDto, DelegatorResponse, DelegatorsResponse, ExternalIssueTypeSummary,
    FieldResponse, HealthResponse, IssueTypeResponse, IssueTypeSummary, KanbanBoardResponse,
    KanbanIssueTypeResponse, KanbanProviderCatalogEntry, KanbanSyncResponse, KanbanTicketCard,
    LaunchTicketRequest, LaunchTicketResponse, ListKanbanProjectsRequest,
    ListKanbanProjectsResponse, ModelServerResponse, ModelServersResponse, NextStepInfo,
    OperatorOutput, ProjectSummary, QueueByType, QueueControlResponse, QueueStatusResponse,
    RejectReviewRequest, ReviewResponse, SectionDto, SectionRowDto, SetDefaultLlmRequest,
    SetKanbanSessionEnvRequest, SetKanbanSessionEnvResponse, SkillEntry, SkillsResponse,
    StatusResponse, StepCompleteRequest, StepCompleteResponse, StepResponse,
    SyncKanbanIssueTypesResponse, TicketDetailResponse, UpdateIssueTypeRequest, UpdateStepRequest,
    UpdateTicketStatusRequest, UpdateTicketStatusResponse, ValidateKanbanCredentialsRequest,
    ValidateKanbanCredentialsResponse, WorkflowExportResponse, WriteKanbanConfigRequest,
    WriteKanbanConfigResponse,
};
use crate::rest::error::ErrorResponse;

/// OpenAPI documentation for the Operator REST API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Operator API",
        version = "0.1.6",
        description = "REST API for managing issue types and collections in the Operator system.",
        license(name = "MIT"),
        contact(
            name = "gbqr.us",
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
            IssueTypeResponse,
            IssueTypeSummary,
            FieldResponse,
            StepResponse,
            CollectionResponse,
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
            // Model server types
            ModelServerResponse,
            ModelServersResponse,
            CreateModelServerRequest,
            // LLM tools types
            SetDefaultLlmRequest,
            DefaultLlmResponse,
            // Ticket types
            TicketDetailResponse,
            UpdateTicketStatusRequest,
            UpdateTicketStatusResponse,
            // Workflow export types
            WorkflowExportResponse,
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
            WriteKanbanConfigRequest,
            WriteKanbanConfigResponse,
            SetKanbanSessionEnvRequest,
            SetKanbanSessionEnvResponse,
        )
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
    )
)]
pub struct ApiDoc;

impl ApiDoc {
    /// Generate the OpenAPI specification as a JSON string.
    ///
    /// Sourced from the fully-mounted router via [`crate::rest::openapi_spec`]
    /// so every live route appears in the spec (the bare `ApiDoc` derive carries
    /// only info/components/tags — paths self-register on mount). The version is
    /// automatically derived from Cargo.toml to stay in sync.
    pub fn json() -> Result<String, serde_json::Error> {
        let mut spec = crate::rest::openapi_spec();
        spec.info.version = env!("CARGO_PKG_VERSION").to_string();
        serde_json::to_string_pretty(&spec)
    }

    /// Generate the OpenAPI specification as a YAML string.
    ///
    /// The version is automatically derived from Cargo.toml to stay in sync.
    #[allow(dead_code)]
    pub fn yaml() -> Result<String, serde_yaml::Error> {
        let mut spec = crate::rest::openapi_spec();
        spec.info.version = env!("CARGO_PKG_VERSION").to_string();
        serde_yaml::to_string(&spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generates() {
        let spec = ApiDoc::json().expect("Failed to generate OpenAPI spec");
        assert!(spec.contains("Operator API"));
        assert!(spec.contains("/api/v1/health"));
        assert!(spec.contains("/api/v1/issuetypes"));
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
    fn test_openapi_version_matches_cargo() {
        let spec = ApiDoc::json().expect("Failed to generate OpenAPI spec");
        let cargo_version = env!("CARGO_PKG_VERSION");
        assert!(
            spec.contains(&format!("\"version\": \"{cargo_version}\"")),
            "OpenAPI version should match Cargo.toml version ({cargo_version}), but spec contains different version"
        );
    }
}
