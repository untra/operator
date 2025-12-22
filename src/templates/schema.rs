//! Schema definitions for issuetype templates

use serde::{Deserialize, Serialize};

/// Schema definition for an issuetype template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSchema {
    /// Unique issuetype key (e.g., FEAT, FIX, SPIKE, INV, TASK)
    pub key: String,
    /// Display name of the template type
    pub name: String,
    /// Brief description of when to use this template
    pub description: String,
    /// Whether this issuetype runs autonomously or requires human pairing
    pub mode: ExecutionMode,
    /// Glyph character displayed in UI for this issuetype
    pub glyph: String,
    /// Optional color for glyph display in TUI
    #[serde(default)]
    pub color: Option<String>,
    /// Git branch prefix for this issuetype
    #[serde(default = "default_branch_prefix")]
    pub branch_prefix: String,
    /// Whether a project must be specified for this issuetype
    #[serde(default = "default_true")]
    pub project_required: bool,
    /// Field definitions for this template
    pub fields: Vec<FieldSchema>,
    /// Lifecycle steps for completing this ticket type
    pub steps: Vec<StepSchema>,
    /// Prompt for generating this issue type's operator agent via `claude -p`
    #[serde(default)]
    pub agent_prompt: Option<String>,
}

fn default_branch_prefix() -> String {
    "task".to_string()
}

fn default_true() -> bool {
    true
}

/// Execution mode for an issuetype
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    /// Runs without human interaction
    Autonomous,
    /// Requires human pairing/interaction
    Paired,
}

/// Schema definition for a single field in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Field identifier (matches handlebar variable name)
    pub name: String,
    /// Help text for the field
    pub description: String,
    /// Type of the field
    #[serde(rename = "type")]
    pub field_type: FieldType,
    /// Whether this field must be filled
    #[serde(default)]
    pub required: bool,
    /// Default value if any
    #[serde(default)]
    pub default: Option<String>,
    /// Auto-generation strategy for this field
    #[serde(default)]
    pub auto: Option<AutoGenStrategy>,
    /// Options for enum fields
    #[serde(default)]
    pub options: Vec<String>,
    /// Placeholder text shown in template
    #[serde(default)]
    pub placeholder: Option<String>,
    /// Maximum length for string fields
    #[serde(default)]
    pub max_length: Option<usize>,
    /// Display order in form (lower = first)
    #[serde(default)]
    pub display_order: Option<i32>,
    /// Whether the user can edit this field (false for auto-generated)
    #[serde(default = "default_true")]
    pub user_editable: bool,
}

/// Auto-generation strategies for fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AutoGenStrategy {
    /// Generate ID from timestamp (e.g., FEAT-1234)
    Id,
    /// Generate current date (YYYY-MM-DD)
    Date,
    /// Generate branch name from type and summary
    Branch,
    /// Set initial status
    Status,
}

/// Types of fields supported in template schemas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    /// Single-line text input
    String,
    /// Selection from predefined options
    Enum,
    /// True/false checkbox
    Bool,
    /// Date field (YYYY-MM-DD format)
    Date,
    /// Multi-line text input
    Text,
}

/// Schema definition for a lifecycle step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSchema {
    /// Step identifier (lowercase)
    pub name: String,
    /// Human-readable step name
    #[serde(default)]
    pub display_name: Option<String>,
    /// Types of outputs this step produces
    pub outputs: Vec<StepOutput>,
    /// Initial prompt template for the Claude agent
    pub prompt: String,
    /// Claude Code tools allowed in this step
    pub allowed_tools: Vec<String>,
    /// Whether this step requires human review before proceeding
    #[serde(default)]
    pub requires_review: bool,
    /// What to do if step output is rejected
    #[serde(default)]
    pub on_reject: Option<OnReject>,
    /// Name of the next step (None for final step)
    #[serde(default)]
    pub next_step: Option<String>,
}

/// Status category for a step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    TODO,
    DOING,
    AWAIT,
    DONE,
}

/// Types of outputs a step can produce
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StepOutput {
    /// Implementation plan
    Plan,
    /// Source code changes
    Code,
    /// Test code/results
    Test,
    /// Pull request
    Pr,
    /// New ticket(s)
    Ticket,
    /// Review output
    Review,
    /// Investigation/research report
    Report,
    /// Documentation
    Documentation,
}

/// Action to take when a step is rejected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnReject {
    /// Step name to return to on rejection
    pub goto_step: String,
    /// Prompt to use when restarting after rejection
    pub prompt: String,
}

impl TemplateSchema {
    /// Parse a template schema from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Validate the schema for consistency
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check key format
        if !self.key.chars().all(|c| c.is_ascii_uppercase()) {
            errors.push(format!("Key '{}' must be uppercase letters only", self.key));
        }

        // Check that all required fields (except 'id' with auto=id) have defaults
        for field in &self.fields {
            if field.required && field.auto.is_none() && field.name != "id" {
                if field.default.is_none() {
                    errors.push(format!(
                        "Required field '{}' must have a default value",
                        field.name
                    ));
                }
            }

            // Check enum fields have options
            if field.field_type == FieldType::Enum && field.options.is_empty() {
                errors.push(format!("Enum field '{}' must have options", field.name));
            }
        }

        // Check step transitions
        let step_names: Vec<&str> = self.steps.iter().map(|s| s.name.as_str()).collect();
        for step in &self.steps {
            if let Some(ref next) = step.next_step {
                if !step_names.contains(&next.as_str()) {
                    errors.push(format!(
                        "Step '{}' references unknown next_step '{}'",
                        step.name, next
                    ));
                }
            }

            if let Some(ref on_reject) = step.on_reject {
                if !step_names.contains(&on_reject.goto_step.as_str()) {
                    errors.push(format!(
                        "Step '{}' on_reject references unknown step '{}'",
                        step.name, on_reject.goto_step
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get step by name
    pub fn get_step(&self, name: &str) -> Option<&StepSchema> {
        self.steps.iter().find(|s| s.name == name)
    }

    /// Get the first step (entry point)
    pub fn first_step(&self) -> Option<&StepSchema> {
        self.steps.first()
    }
}

impl StepSchema {
    /// Get the display name, falling back to name if not set
    pub fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }

    /// Derive status from step properties and position
    pub fn derived_status(&self, is_first: bool, is_last: bool) -> StepStatus {
        if is_last {
            StepStatus::DONE
        } else if self.requires_review {
            StepStatus::AWAIT
        } else if is_first {
            StepStatus::TODO
        } else {
            StepStatus::DOING
        }
    }

    /// Check if this step outputs a plan
    pub fn outputs_plan(&self) -> bool {
        self.outputs.contains(&StepOutput::Plan)
    }

    /// Check if this step outputs a review
    pub fn outputs_review(&self) -> bool {
        self.outputs.contains(&StepOutput::Review)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_schema() {
        let json = r#"{
            "key": "TEST",
            "name": "Test",
            "description": "Test template",
            "mode": "autonomous",
            "glyph": "*",
            "branch_prefix": "test",
            "project_required": true,
            "fields": [
                {
                    "name": "id",
                    "description": "Unique ID",
                    "type": "string",
                    "required": true,
                    "auto": "id",
                    "max_length": 50,
                    "display_order": 0,
                    "user_editable": false
                },
                {
                    "name": "priority",
                    "description": "Priority level",
                    "type": "enum",
                    "required": true,
                    "default": "P2-medium",
                    "options": ["P0-critical", "P1-high", "P2-medium", "P3-low"],
                    "display_order": 1
                }
            ],
            "steps": [
                {
                    "name": "plan",
                    "display_name": "Planning",
                    "outputs": ["plan"],
                    "prompt": "Create a plan for implementing this feature",
                    "allowed_tools": ["Read", "Glob", "Grep"],
                    "next_step": "build"
                },
                {
                    "name": "build",
                    "display_name": "Building",
                    "outputs": ["code"],
                    "prompt": "Implement the plan",
                    "allowed_tools": ["Read", "Write", "Edit", "Bash"],
                    "requires_review": true,
                    "on_reject": {
                        "goto_step": "plan",
                        "prompt": "Review rejected. Please revise the plan."
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.key, "TEST");
        assert_eq!(schema.name, "Test");
        assert_eq!(schema.mode, ExecutionMode::Autonomous);
        assert_eq!(schema.fields.len(), 2);
        assert_eq!(schema.fields[0].auto, Some(AutoGenStrategy::Id));
        assert!(!schema.fields[0].user_editable);
        assert_eq!(schema.steps.len(), 2);
        assert!(schema.steps[0].outputs_plan());
        assert!(schema.steps[1].requires_review);

        // Validate
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_validation_catches_missing_default() {
        let json = r#"{
            "key": "TEST",
            "name": "Test",
            "description": "Test template",
            "mode": "autonomous",
            "glyph": "*",
            "fields": [
                {
                    "name": "summary",
                    "description": "Summary",
                    "type": "string",
                    "required": true
                }
            ],
            "steps": [
                {
                    "name": "do",
                    "outputs": ["code"],
                    "prompt": "Do the thing",
                    "allowed_tools": ["Read"]
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        let result = schema.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("must have a default value"));
    }

    #[test]
    fn test_validation_catches_invalid_step_reference() {
        let json = r#"{
            "key": "TEST",
            "name": "Test",
            "description": "Test template",
            "mode": "autonomous",
            "glyph": "*",
            "fields": [
                {
                    "name": "id",
                    "description": "ID",
                    "type": "string",
                    "required": true,
                    "auto": "id"
                }
            ],
            "steps": [
                {
                    "name": "plan",
                    "outputs": ["plan"],
                    "prompt": "Plan it",
                    "allowed_tools": ["Read"],
                    "next_step": "nonexistent"
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        let result = schema.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("unknown next_step"));
    }
}
