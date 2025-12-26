//! OpenAPI specification builder using utoipa.

use utoipa::OpenApi;

use crate::rest::dto::{
    CollectionResponse, CreateFieldRequest, CreateIssueTypeRequest, CreateStepRequest,
    FieldResponse, HealthResponse, IssueTypeResponse, IssueTypeSummary, StatusResponse,
    StepResponse, UpdateIssueTypeRequest, UpdateStepRequest,
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
            ErrorResponse,
            // Request types
            CreateIssueTypeRequest,
            UpdateIssueTypeRequest,
            CreateFieldRequest,
            CreateStepRequest,
            UpdateStepRequest,
        )
    ),
    tags(
        (name = "Health", description = "Health check and status endpoints"),
        (name = "Issue Types", description = "Issue type CRUD operations"),
        (name = "Steps", description = "Step management within issue types"),
        (name = "Collections", description = "Issue type collection management"),
    )
)]
pub struct ApiDoc;

impl ApiDoc {
    /// Generate the OpenAPI specification as a JSON string
    pub fn json() -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&Self::openapi())
    }

    /// Generate the OpenAPI specification as a YAML string
    #[allow(dead_code)]
    pub fn yaml() -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&Self::openapi())
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
}
