use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::issuetypes::schema::IssueTypeSource;
use crate::issuetypes::{IssueType, IssueTypeCollection};
use crate::templates::schema::{
    ExecutionMode, FieldSchema, FieldType, PermissionMode, StepOutput, StepSchema,
};

// =============================================================================
// Issue Type DTOs
// =============================================================================

/// Response for a single issue type
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
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
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct IssueTypeSummary {
    pub key: String,
    pub name: String,
    pub description: String,
    pub mode: String,
    pub glyph: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub color: Option<String>,
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
            color: it.color.clone(),
            source: it.source_display(),
            step_count: it.steps.len(),
        }
    }
}

/// Request to create a new issue type
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
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
    /// Convert request to `IssueType`
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
            fields: self
                .fields
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            steps: self
                .steps
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            agent_prompt: None,
            agent: None,
            source: IssueTypeSource::User,
            external_id: None,
        }
    }
}

/// Request to update an issue type
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
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
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
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
                FieldType::Integer => "integer".to_string(),
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
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
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
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct StepResponse {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub prompt: String,
    pub outputs: Vec<String>,
    pub allowed_tools: Vec<String>,
    /// Type of review required: "none", "plan", "visual", "pr"
    pub review_type: String,
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
            review_type: match s.review_type {
                crate::templates::schema::ReviewType::None => "none".to_string(),
                crate::templates::schema::ReviewType::Plan => "plan".to_string(),
                crate::templates::schema::ReviewType::Visual => "visual".to_string(),
                crate::templates::schema::ReviewType::Pr => "pr".to_string(),
            },
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
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateStepRequest {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default = "default_all_tools")]
    pub allowed_tools: Vec<String>,
    /// Type of review required: "none", "plan", "visual", "pr"
    #[serde(default = "default_review_type")]
    pub review_type: String,
    #[serde(default)]
    pub next_step: Option<String>,
    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,
}

fn default_all_tools() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_review_type() -> String {
    "none".to_string()
}

fn default_permission_mode() -> String {
    "default".to_string()
}

impl From<CreateStepRequest> for StepSchema {
    fn from(s: CreateStepRequest) -> Self {
        Self {
            name: s.name,
            display_name: s.display_name,
            step_type: crate::templates::schema::StepTypeTag::Task,
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
            review_type: match s.review_type.as_str() {
                "plan" => crate::templates::schema::ReviewType::Plan,
                "visual" => crate::templates::schema::ReviewType::Visual,
                "pr" => crate::templates::schema::ReviewType::Pr,
                _ => crate::templates::schema::ReviewType::None,
            },
            visual_config: None,
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
            artifact_patterns: vec![],
            agent: None,
            classifier_config: None,
            rag_config: None,
            delegator_config: None,
            mcp_config: None,
            multi_model_config: None,
            multi_prompt_config: None,
            matrixed_config: None,
            pipeline_config: None,
        }
    }
}

/// Request to update a step
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct UpdateStepRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub outputs: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,
    /// Type of review required: "none", "plan", "visual", "pr"
    #[serde(default)]
    pub review_type: Option<String>,
    #[serde(default)]
    pub next_step: Option<String>,
    #[serde(default)]
    pub permission_mode: Option<String>,
}

// =============================================================================
// Collection DTOs
// =============================================================================

/// Descriptive workflow hints for a collection (v1: metadata only).
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WorkflowHintsDto {
    #[serde(default)]
    pub loop_kind: Option<String>,
    #[serde(default)]
    pub memory_surfaces: Vec<String>,
    #[serde(default)]
    pub review_gates: Vec<String>,
    #[serde(default)]
    pub external_tools: Vec<String>,
    #[serde(default)]
    pub stop_conditions: Vec<String>,
    pub runner_semantics: String,
}

impl From<&crate::collections::manifest::WorkflowHints> for WorkflowHintsDto {
    fn from(h: &crate::collections::manifest::WorkflowHints) -> Self {
        Self {
            loop_kind: h.loop_kind.clone(),
            memory_surfaces: h.memory_surfaces.clone(),
            review_gates: h.review_gates.clone(),
            external_tools: h.external_tools.clone(),
            stop_conditions: h.stop_conditions.clone(),
            runner_semantics: h.runner_semantics.clone(),
        }
    }
}

/// Response for a collection
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CollectionResponse {
    pub name: String,
    pub description: String,
    pub types: Vec<String>,
    pub is_active: bool,
    /// Collection semver (present for hosted collections).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Publisher identifier (present for hosted collections).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    /// Descriptive workflow hints (present for hosted collections).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_hints: Option<WorkflowHintsDto>,
}

impl CollectionResponse {
    pub fn from_collection(c: &IssueTypeCollection, is_active: bool) -> Self {
        Self {
            name: c.name.clone(),
            description: c.description.clone(),
            types: c.types.clone(),
            is_active,
            version: c.version.clone(),
            publisher: c.publisher.clone(),
            workflow_hints: c.workflow_hints.as_ref().map(WorkflowHintsDto::from),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_type_summary_serializes_step_count_as_camel_case() {
        // IssueTypeSummary is the only DTO here using rename_all = "camelCase";
        // snake_case `step_count` must appear on the wire as `stepCount`.
        let summary = IssueTypeSummary {
            key: "FEAT".to_string(),
            name: "Feature".to_string(),
            description: "A feature".to_string(),
            mode: "autonomous".to_string(),
            glyph: "F".to_string(),
            color: Some("cyan".to_string()),
            source: "user".to_string(),
            step_count: 3,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"stepCount\":3"));
        assert!(!json.contains("step_count"));
    }

    #[test]
    fn test_issue_type_summary_color_absent_when_none() {
        let summary = IssueTypeSummary {
            key: "FEAT".to_string(),
            name: "Feature".to_string(),
            description: "A feature".to_string(),
            mode: "autonomous".to_string(),
            glyph: "F".to_string(),
            color: None,
            source: "user".to_string(),
            step_count: 0,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(!json.contains("color"));
    }

    #[test]
    fn test_issue_type_summary_deserializes_from_camel_case() {
        let json = r#"{
            "key": "FIX",
            "name": "Fix",
            "description": "A fix",
            "mode": "autonomous",
            "glyph": "X",
            "source": "user",
            "stepCount": 5
        }"#;
        let summary: IssueTypeSummary = serde_json::from_str(json).unwrap();
        assert_eq!(summary.step_count, 5);
        assert!(summary.color.is_none());
    }

    #[test]
    fn test_create_issue_type_request_applies_defaults_when_absent() {
        // mode -> default_mode(), project_required -> default_true(), fields -> empty.
        let json = r#"{
            "key": "feat",
            "name": "Feature",
            "description": "A feature",
            "glyph": "F",
            "steps": []
        }"#;
        let req: CreateIssueTypeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.mode, "autonomous");
        assert!(req.project_required);
        assert!(req.fields.is_empty());
        assert!(req.color.is_none());
    }

    #[test]
    fn test_create_field_request_applies_typed_defaults_when_absent() {
        // field_type -> default_string_type(), user_editable -> default_true(),
        // required defaults to false, options to empty.
        let json = r#"{ "name": "title", "description": "Title field" }"#;
        let req: CreateFieldRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.field_type, "string");
        assert!(req.user_editable);
        assert!(!req.required);
        assert!(req.options.is_empty());
    }

    #[test]
    fn test_create_step_request_applies_defaults_when_absent() {
        // allowed_tools -> ["*"], review_type -> "none", permission_mode -> "default".
        let json = r#"{ "name": "execute", "prompt": "Do the thing" }"#;
        let req: CreateStepRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.allowed_tools, vec!["*".to_string()]);
        assert_eq!(req.review_type, "none");
        assert_eq!(req.permission_mode, "default");
        assert!(req.next_step.is_none());
        assert!(req.outputs.is_empty());
    }

    #[test]
    fn test_field_response_skips_empty_options_and_none_default() {
        let resp = FieldResponse {
            name: "title".to_string(),
            description: "Title".to_string(),
            field_type: "string".to_string(),
            required: true,
            default: None,
            options: vec![],
            placeholder: None,
            max_length: None,
            user_editable: true,
        };
        let json = serde_json::to_string(&resp).unwrap();
        // skip_serializing_if = "Vec::is_empty" and "Option::is_none"
        assert!(!json.contains("options"));
        assert!(!json.contains("default"));
        assert!(!json.contains("placeholder"));
        assert!(json.contains("\"required\":true"));
    }

    #[test]
    fn test_field_response_includes_options_when_present() {
        let resp = FieldResponse {
            name: "priority".to_string(),
            description: "Priority".to_string(),
            field_type: "enum".to_string(),
            required: false,
            default: Some("P2".to_string()),
            options: vec!["P0".to_string(), "P2".to_string()],
            placeholder: None,
            max_length: None,
            user_editable: true,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"options\":[\"P0\",\"P2\"]"));
        assert!(json.contains("\"default\":\"P2\""));
    }

    #[test]
    fn test_update_issue_type_request_all_fields_optional() {
        // Every field is #[serde(default)] Option/None; empty object parses to all-None.
        let req: UpdateIssueTypeRequest = serde_json::from_str("{}").unwrap();
        assert!(req.name.is_none());
        assert!(req.mode.is_none());
        assert!(req.project_required.is_none());
        assert!(req.fields.is_none());
        assert!(req.steps.is_none());
    }

    #[test]
    fn test_collection_response_roundtrip() {
        let resp = CollectionResponse {
            name: "default".to_string(),
            description: "Default collection".to_string(),
            types: vec!["FEAT".to_string(), "FIX".to_string()],
            is_active: true,
            version: None,
            publisher: None,
            workflow_hints: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: CollectionResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.types, vec!["FEAT".to_string(), "FIX".to_string()]);
        assert!(parsed.is_active);
        // Optional hosted-only fields are omitted from the wire when absent.
        assert!(!json.contains("workflow_hints"));
    }

    #[test]
    fn test_collection_response_includes_workflow_hints() {
        let collection = IssueTypeCollection::new("dev_kanban", "Dev")
            .with_types(["TASK"])
            .with_manifest_metadata(
                Some(crate::collections::manifest::WorkflowHints {
                    loop_kind: Some("single_pass".to_string()),
                    review_gates: vec!["test_suite".to_string()],
                    ..Default::default()
                }),
                Some("1.0.0".to_string()),
                Some("untra".to_string()),
            );
        let resp = CollectionResponse::from_collection(&collection, true);
        assert_eq!(resp.version.as_deref(), Some("1.0.0"));
        assert_eq!(resp.publisher.as_deref(), Some("untra"));
        let hints = resp.workflow_hints.expect("hints surfaced");
        assert_eq!(hints.loop_kind.as_deref(), Some("single_pass"));
        assert_eq!(hints.runner_semantics, "prompt_driven");
    }
}
