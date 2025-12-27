//! Data Transfer Objects for the REST API.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Note: ToSchema is derived on all DTOs for OpenAPI documentation generation

use crate::issuetypes::schema::IssueTypeSource;
use crate::issuetypes::{IssueType, IssueTypeCollection};
use crate::templates::schema::{
    ExecutionMode, FieldSchema, FieldType, PermissionMode, StepOutput, StepSchema,
};

// =============================================================================
// Issue Type DTOs
// =============================================================================

/// Response for a single issue type
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IssueTypeResponse {
    pub key: String,
    pub name: String,
    pub description: String,
    pub mode: String,
    pub glyph: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub project_required: bool,
    pub source: String,
    pub fields: Vec<FieldResponse>,
    pub steps: Vec<StepResponse>,
}

impl From<&IssueType> for IssueTypeResponse {
    fn from(it: &IssueType) -> Self {
        Self {
            key: it.key.clone(),
            name: it.name.clone(),
            description: it.description.clone(),
            mode: match it.mode {
                ExecutionMode::Autonomous => "autonomous".to_string(),
                ExecutionMode::Paired => "paired".to_string(),
            },
            glyph: it.glyph.clone(),
            color: it.color.clone(),
            project_required: it.project_required,
            source: it.source_display(),
            fields: it.fields.iter().map(FieldResponse::from).collect(),
            steps: it.steps.iter().map(StepResponse::from).collect(),
        }
    }
}

/// Summary response for listing issue types
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IssueTypeSummary {
    pub key: String,
    pub name: String,
    pub description: String,
    pub mode: String,
    pub glyph: String,
    pub source: String,
    pub step_count: usize,
}

impl From<&IssueType> for IssueTypeSummary {
    fn from(it: &IssueType) -> Self {
        Self {
            key: it.key.clone(),
            name: it.name.clone(),
            description: it.description.clone(),
            mode: match it.mode {
                ExecutionMode::Autonomous => "autonomous".to_string(),
                ExecutionMode::Paired => "paired".to_string(),
            },
            glyph: it.glyph.clone(),
            source: it.source_display(),
            step_count: it.steps.len(),
        }
    }
}

/// Request to create a new issue type
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateIssueTypeRequest {
    pub key: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    pub glyph: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default = "default_true")]
    pub project_required: bool,
    #[serde(default)]
    pub fields: Vec<CreateFieldRequest>,
    pub steps: Vec<CreateStepRequest>,
}

fn default_mode() -> String {
    "autonomous".to_string()
}

fn default_true() -> bool {
    true
}

impl CreateIssueTypeRequest {
    /// Convert request to IssueType
    pub fn into_issue_type(self) -> IssueType {
        IssueType {
            key: self.key.to_uppercase(),
            name: self.name,
            description: self.description,
            mode: if self.mode == "paired" {
                ExecutionMode::Paired
            } else {
                ExecutionMode::Autonomous
            },
            glyph: self.glyph,
            color: self.color,
            project_required: self.project_required,
            fields: self.fields.into_iter().map(|f| f.into()).collect(),
            steps: self.steps.into_iter().map(|s| s.into()).collect(),
            agent_prompt: None,
            source: IssueTypeSource::User,
            external_id: None,
        }
    }
}

/// Request to update an issue type
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateIssueTypeRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub glyph: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub project_required: Option<bool>,
    #[serde(default)]
    pub fields: Option<Vec<CreateFieldRequest>>,
    #[serde(default)]
    pub steps: Option<Vec<CreateStepRequest>>,
}

// =============================================================================
// Field DTOs
// =============================================================================

/// Response for a field
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FieldResponse {
    pub name: String,
    pub description: String,
    pub field_type: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    pub user_editable: bool,
}

impl From<&FieldSchema> for FieldResponse {
    fn from(f: &FieldSchema) -> Self {
        Self {
            name: f.name.clone(),
            description: f.description.clone(),
            field_type: match f.field_type {
                FieldType::String => "string".to_string(),
                FieldType::Enum => "enum".to_string(),
                FieldType::Bool => "bool".to_string(),
                FieldType::Date => "date".to_string(),
                FieldType::Text => "text".to_string(),
            },
            required: f.required,
            default: f.default.clone(),
            options: f.options.clone(),
            placeholder: f.placeholder.clone(),
            max_length: f.max_length,
            user_editable: f.user_editable,
        }
    }
}

/// Request to create a field
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateFieldRequest {
    pub name: String,
    pub description: String,
    #[serde(default = "default_string_type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default = "default_true")]
    pub user_editable: bool,
}

fn default_string_type() -> String {
    "string".to_string()
}

impl From<CreateFieldRequest> for FieldSchema {
    fn from(f: CreateFieldRequest) -> Self {
        Self {
            name: f.name,
            description: f.description,
            field_type: match f.field_type.as_str() {
                "enum" => FieldType::Enum,
                "bool" => FieldType::Bool,
                "date" => FieldType::Date,
                "text" => FieldType::Text,
                _ => FieldType::String,
            },
            required: f.required,
            default: f.default,
            auto: None,
            options: f.options,
            placeholder: f.placeholder,
            max_length: f.max_length,
            display_order: None,
            user_editable: f.user_editable,
        }
    }
}

// =============================================================================
// Step DTOs
// =============================================================================

/// Response for a step
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StepResponse {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub prompt: String,
    pub outputs: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub requires_review: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step: Option<String>,
    pub permission_mode: String,
}

impl From<&StepSchema> for StepResponse {
    fn from(s: &StepSchema) -> Self {
        Self {
            name: s.name.clone(),
            display_name: s.display_name.clone(),
            prompt: s.prompt.clone(),
            outputs: s
                .outputs
                .iter()
                .map(|o| match o {
                    StepOutput::Plan => "plan".to_string(),
                    StepOutput::Code => "code".to_string(),
                    StepOutput::Test => "test".to_string(),
                    StepOutput::Pr => "pr".to_string(),
                    StepOutput::Ticket => "ticket".to_string(),
                    StepOutput::Review => "review".to_string(),
                    StepOutput::Report => "report".to_string(),
                    StepOutput::Documentation => "documentation".to_string(),
                })
                .collect(),
            allowed_tools: s.allowed_tools.clone(),
            requires_review: s.requires_review,
            next_step: s.next_step.clone(),
            permission_mode: match s.permission_mode {
                PermissionMode::Default => "default".to_string(),
                PermissionMode::Plan => "plan".to_string(),
                PermissionMode::AcceptEdits => "acceptEdits".to_string(),
                PermissionMode::Delegate => "delegate".to_string(),
            },
        }
    }
}

/// Request to create a step
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateStepRequest {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default = "default_all_tools")]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub requires_review: bool,
    #[serde(default)]
    pub next_step: Option<String>,
    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,
}

fn default_all_tools() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_permission_mode() -> String {
    "default".to_string()
}

impl From<CreateStepRequest> for StepSchema {
    fn from(s: CreateStepRequest) -> Self {
        Self {
            name: s.name,
            display_name: s.display_name,
            prompt: s.prompt,
            outputs: s
                .outputs
                .iter()
                .filter_map(|o| match o.as_str() {
                    "plan" => Some(StepOutput::Plan),
                    "code" => Some(StepOutput::Code),
                    "test" => Some(StepOutput::Test),
                    "pr" => Some(StepOutput::Pr),
                    "ticket" => Some(StepOutput::Ticket),
                    "review" => Some(StepOutput::Review),
                    "report" => Some(StepOutput::Report),
                    "documentation" => Some(StepOutput::Documentation),
                    _ => None,
                })
                .collect(),
            allowed_tools: s.allowed_tools,
            requires_review: s.requires_review,
            on_reject: None,
            next_step: s.next_step,
            permissions: None,
            cli_args: None,
            permission_mode: match s.permission_mode.as_str() {
                "plan" => PermissionMode::Plan,
                "acceptEdits" => PermissionMode::AcceptEdits,
                "delegate" => PermissionMode::Delegate,
                _ => PermissionMode::Default,
            },
            json_schema: None,
            json_schema_file: None,
        }
    }
}

/// Request to update a step
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateStepRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub outputs: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,
    #[serde(default)]
    pub requires_review: Option<bool>,
    #[serde(default)]
    pub next_step: Option<String>,
    #[serde(default)]
    pub permission_mode: Option<String>,
}

// =============================================================================
// Collection DTOs
// =============================================================================

/// Response for a collection
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CollectionResponse {
    pub name: String,
    pub description: String,
    pub types: Vec<String>,
    pub is_active: bool,
}

impl CollectionResponse {
    pub fn from_collection(c: &IssueTypeCollection, is_active: bool) -> Self {
        Self {
            name: c.name.clone(),
            description: c.description.clone(),
            types: c.types.clone(),
            is_active,
        }
    }
}

// =============================================================================
// Health/Status DTOs
// =============================================================================

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Status response with registry info
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub issuetype_count: usize,
    pub collection_count: usize,
    pub active_collection: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_issue_type_request_into() {
        let req = CreateIssueTypeRequest {
            key: "test".to_string(),
            name: "Test".to_string(),
            description: "A test type".to_string(),
            mode: "autonomous".to_string(),
            glyph: "T".to_string(),
            color: None,
            project_required: true,
            fields: vec![],
            steps: vec![CreateStepRequest {
                name: "execute".to_string(),
                display_name: None,
                prompt: "Do the thing".to_string(),
                outputs: vec![],
                allowed_tools: vec!["*".to_string()],
                requires_review: false,
                next_step: None,
                permission_mode: "default".to_string(),
            }],
        };

        let it = req.into_issue_type();
        assert_eq!(it.key, "TEST"); // Uppercased
        assert_eq!(it.name, "Test");
        assert!(matches!(it.mode, ExecutionMode::Autonomous));
        assert!(matches!(it.source, IssueTypeSource::User));
        assert_eq!(it.steps.len(), 1);
    }

    #[test]
    fn test_issue_type_response_from() {
        let it = IssueType::new_imported(
            "TEST".to_string(),
            "Test".to_string(),
            "A test".to_string(),
            "jira".to_string(),
            "PROJ".to_string(),
            None,
        );

        let resp = IssueTypeResponse::from(&it);
        assert_eq!(resp.key, "TEST");
        assert_eq!(resp.mode, "autonomous");
        assert_eq!(resp.source, "jira/PROJ");
    }
}
