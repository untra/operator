//! OpenAPI specification builder using utoipa.

use utoipa::OpenApi;

use crate::rest::dto::{
    CollectionResponse, CreateDelegatorRequest, CreateFieldRequest, CreateIssueTypeRequest,
    CreateStepRequest, DelegatorLaunchConfigDto, DelegatorResponse, DelegatorsResponse,
    FieldResponse, HealthResponse, IssueTypeResponse, IssueTypeSummary, LaunchTicketRequest,
    LaunchTicketResponse, SkillEntry, SkillsResponse, StatusResponse, StepResponse,
    UpdateIssueTypeRequest, UpdateStepRequest,
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
    paths(
        // Health endpoints
        crate::rest::routes::health::health,
        crate::rest::routes::health::status,
        // Issue type endpoints
        crate::rest::routes::issuetypes::list,
        crate::rest::routes::issuetypes::get_one,
        crate::rest::routes::issuetypes::create,
        crate::rest::routes::issuetypes::update,
        crate::rest::routes::issuetypes::delete,
        // Step endpoints
        crate::rest::routes::steps::list,
        crate::rest::routes::steps::get_one,
        crate::rest::routes::steps::update,
        // Collection endpoints
        crate::rest::routes::collections::list,
        crate::rest::routes::collections::get_active,
        crate::rest::routes::collections::get_one,
        crate::rest::routes::collections::activate,
        // Launch endpoints
        crate::rest::routes::launch::launch_ticket,
        // Skills endpoints
        crate::rest::routes::skills::list,
        // Delegator endpoints
        crate::rest::routes::delegators::list,
        crate::rest::routes::delegators::get_one,
        crate::rest::routes::delegators::create,
        crate::rest::routes::delegators::delete,
    ),
    components(
        schemas(
            // Response types
            HealthResponse,
            StatusResponse,
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
            DelegatorLaunchConfigDto,
        )
    ),
    tags(
        (name = "Health", description = "Health check and status endpoints"),
        (name = "Issue Types", description = "Issue type CRUD operations"),
        (name = "Steps", description = "Step management within issue types"),
        (name = "Collections", description = "Issue type collection management"),
        (name = "Launch", description = "Ticket launch operations"),
        (name = "Skills", description = "Skill discovery across LLM tools"),
        (name = "Delegators", description = "Agent delegator CRUD operations"),
    )
)]
pub struct ApiDoc;

impl ApiDoc {
    /// Generate the OpenAPI specification as a JSON string
    ///
    /// The version is automatically derived from Cargo.toml to stay in sync.
    pub fn json() -> Result<String, serde_json::Error> {
        let mut spec = Self::openapi();
        spec.info.version = env!("CARGO_PKG_VERSION").to_string();
        serde_json::to_string_pretty(&spec)
    }

    /// Generate the OpenAPI specification as a YAML string
    ///
    /// The version is automatically derived from Cargo.toml to stay in sync.
    #[allow(dead_code)]
    pub fn yaml() -> Result<String, serde_yaml::Error> {
        let mut spec = Self::openapi();
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
    fn test_openapi_version_matches_cargo() {
        let spec = ApiDoc::json().expect("Failed to generate OpenAPI spec");
        let cargo_version = env!("CARGO_PKG_VERSION");
        assert!(
            spec.contains(&format!("\"version\": \"{}\"", cargo_version)),
            "OpenAPI version should match Cargo.toml version ({}), but spec contains different version",
            cargo_version
        );
    }
}
