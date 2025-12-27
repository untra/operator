//! Schema definitions for dynamic issue types

use serde::{Deserialize, Serialize};

use crate::templates::schema::{
    AutoGenStrategy, ExecutionMode, FieldSchema, FieldType, PermissionMode, StepSchema,
};

/// Source of an issue type definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum IssueTypeSource {
    /// Built-in to operator binary
    #[default]
    Builtin,
    /// User-defined in .tickets/operator/issuetypes/
    User,
    /// Imported from external kanban provider
    #[serde(rename = "import")]
    Import {
        /// Provider name (e.g., "jira", "linear")
        provider: String,
        /// Project/team identifier
        project: String,
    },
}

/// An issue type definition (dynamic version of TemplateSchema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueType {
    /// Unique issuetype key (e.g., FEAT, FIX, STORY, BUG)
    pub key: String,
    /// Display name of the issue type
    pub name: String,
    /// Brief description of when to use this issue type
    pub description: String,
    /// Whether this issue type runs autonomously or requires human pairing
    pub mode: ExecutionMode,
    /// Glyph character displayed in UI for this issue type
    pub glyph: String,
    /// Optional color for glyph display in TUI
    #[serde(default)]
    pub color: Option<String>,
    /// Whether a project must be specified for this issue type
    #[serde(default = "default_true")]
    pub project_required: bool,
    /// Field definitions for this issue type
    pub fields: Vec<FieldSchema>,
    /// Lifecycle steps for completing this ticket type
    pub steps: Vec<StepSchema>,
    /// Prompt for generating this issue type's operator agent via `claude -p`
    #[serde(default)]
    pub agent_prompt: Option<String>,
    /// Source of this issue type (builtin, user, import)
    #[serde(default)]
    pub source: IssueTypeSource,
    /// Original external ID (for imported types)
    #[serde(default)]
    pub external_id: Option<String>,
}

fn default_true() -> bool {
    true
}

impl IssueType {
    /// Create an issue type from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Convert to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Check if this issue type requires paired mode
    pub fn is_paired(&self) -> bool {
        self.mode == ExecutionMode::Paired
    }

    /// Check if this issue type runs autonomously
    pub fn is_autonomous(&self) -> bool {
        self.mode == ExecutionMode::Autonomous
    }

    /// Get display name for source
    pub fn source_display(&self) -> String {
        match &self.source {
            IssueTypeSource::Builtin => "builtin".to_string(),
            IssueTypeSource::User => "user".to_string(),
            IssueTypeSource::Import { provider, project } => {
                format!("{}/{}", provider, project)
            }
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

    /// Create a minimal imported issue type with a single "execute" step
    pub fn new_imported(
        key: String,
        name: String,
        description: String,
        provider: String,
        project: String,
        external_id: Option<String>,
    ) -> Self {
        Self {
            key: key.clone(),
            name,
            description,
            mode: ExecutionMode::Autonomous,
            glyph: key.chars().next().unwrap_or('?').to_string(),
            color: None,
            project_required: true,
            fields: vec![
                FieldSchema {
                    name: "id".to_string(),
                    description: "Unique identifier".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: None,
                    auto: Some(AutoGenStrategy::Id),
                    options: vec![],
                    placeholder: None,
                    max_length: Some(50),
                    display_order: Some(0),
                    user_editable: false,
                },
                FieldSchema {
                    name: "summary".to_string(),
                    description: "Brief summary".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    default: Some(String::new()),
                    auto: None,
                    options: vec![],
                    placeholder: Some("What needs to be done?".to_string()),
                    max_length: Some(200),
                    display_order: Some(1),
                    user_editable: true,
                },
            ],
            steps: vec![StepSchema {
                name: "execute".to_string(),
                display_name: Some("Execute".to_string()),
                outputs: vec![],
                prompt: "Execute this task according to the description.".to_string(),
                allowed_tools: vec!["*".to_string()],
                requires_review: false,
                on_reject: None,
                next_step: None,
                permissions: None,
                cli_args: None,
                permission_mode: PermissionMode::Default,
                json_schema: None,
                json_schema_file: None,
            }],
            agent_prompt: None,
            source: IssueTypeSource::Import { provider, project },
            external_id,
        }
    }
}

/// Validation errors for issue types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Key format is invalid (must be uppercase letters only)
    InvalidKey(String),
    /// Key length is invalid (must be 2-10 characters)
    KeyLength(String),
    /// Glyph is invalid (must be 1-4 characters)
    InvalidGlyph(String),
    /// Required field missing default value
    MissingDefault(String),
    /// Enum field has no options
    MissingEnumOptions(String),
    /// Step references non-existent step
    InvalidStepRef(String),
    /// No steps defined
    NoSteps,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidKey(key) => {
                write!(f, "Key '{}' must be uppercase letters only", key)
            }
            ValidationError::KeyLength(key) => {
                write!(f, "Key '{}' must be 2-10 characters", key)
            }
            ValidationError::InvalidGlyph(glyph) => {
                write!(f, "Glyph '{}' must be 1-4 characters", glyph)
            }
            ValidationError::MissingDefault(field) => {
                write!(f, "Required field '{}' must have a default value", field)
            }
            ValidationError::MissingEnumOptions(field) => {
                write!(f, "Enum field '{}' must have options", field)
            }
            ValidationError::InvalidStepRef(step) => {
                write!(f, "References unknown step '{}'", step)
            }
            ValidationError::NoSteps => {
                write!(f, "Issue type must have at least one step")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl IssueType {
    /// Validate the issue type for consistency
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check key format: uppercase letters only
        if !self.key.chars().all(|c| c.is_ascii_uppercase()) {
            errors.push(ValidationError::InvalidKey(self.key.clone()));
        }

        // Check key length: 2-10 characters
        if self.key.len() < 2 || self.key.len() > 10 {
            errors.push(ValidationError::KeyLength(self.key.clone()));
        }

        // Check glyph: 1-4 characters
        let glyph_chars = self.glyph.chars().count();
        if !(1..=4).contains(&glyph_chars) {
            errors.push(ValidationError::InvalidGlyph(self.glyph.clone()));
        }

        // Check that steps exist
        if self.steps.is_empty() {
            errors.push(ValidationError::NoSteps);
        }

        // Check that all required fields (except 'id' with auto=id) have defaults
        for field in &self.fields {
            if field.required
                && field.auto.is_none()
                && field.name != "id"
                && field.default.is_none()
            {
                errors.push(ValidationError::MissingDefault(field.name.clone()));
            }

            // Check enum fields have options
            if field.field_type == FieldType::Enum && field.options.is_empty() {
                errors.push(ValidationError::MissingEnumOptions(field.name.clone()));
            }
        }

        // Check step transitions reference valid steps
        let step_names: std::collections::HashSet<&str> =
            self.steps.iter().map(|s| s.name.as_str()).collect();

        for step in &self.steps {
            if let Some(ref next) = step.next_step {
                if !step_names.contains(next.as_str()) {
                    errors.push(ValidationError::InvalidStepRef(next.clone()));
                }
            }

            if let Some(ref on_reject) = step.on_reject {
                if !step_names.contains(on_reject.goto_step.as_str()) {
                    errors.push(ValidationError::InvalidStepRef(on_reject.goto_step.clone()));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_issuetype() -> IssueType {
        IssueType {
            key: "TEST".to_string(),
            name: "Test".to_string(),
            description: "A test issue type".to_string(),
            mode: ExecutionMode::Autonomous,
            glyph: "T".to_string(),
            color: Some("cyan".to_string()),
            project_required: true,
            fields: vec![FieldSchema {
                name: "id".to_string(),
                description: "ID".to_string(),
                field_type: FieldType::String,
                required: true,
                default: None,
                auto: Some(AutoGenStrategy::Id),
                options: vec![],
                placeholder: None,
                max_length: Some(50),
                display_order: Some(0),
                user_editable: false,
            }],
            steps: vec![StepSchema {
                name: "execute".to_string(),
                display_name: Some("Execute".to_string()),
                outputs: vec![],
                prompt: "Do the task".to_string(),
                allowed_tools: vec!["*".to_string()],
                requires_review: false,
                on_reject: None,
                next_step: None,
                permissions: None,
                cli_args: None,
                permission_mode: PermissionMode::Default,
                json_schema: None,
                json_schema_file: None,
            }],
            agent_prompt: None,
            source: IssueTypeSource::User,
            external_id: None,
        }
    }

    #[test]
    fn test_valid_issuetype() {
        let issue_type = create_valid_issuetype();
        assert!(issue_type.validate().is_ok());
    }

    #[test]
    fn test_invalid_key_lowercase() {
        let mut issue_type = create_valid_issuetype();
        issue_type.key = "test".to_string();
        let result = issue_type.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidKey(_))));
    }

    #[test]
    fn test_invalid_key_too_short() {
        let mut issue_type = create_valid_issuetype();
        issue_type.key = "T".to_string();
        let result = issue_type.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::KeyLength(_))));
    }

    #[test]
    fn test_invalid_key_too_long() {
        let mut issue_type = create_valid_issuetype();
        issue_type.key = "VERYLONGKEY".to_string();
        let result = issue_type.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::KeyLength(_))));
    }

    #[test]
    fn test_invalid_glyph_empty() {
        let mut issue_type = create_valid_issuetype();
        issue_type.glyph = "".to_string();
        let result = issue_type.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidGlyph(_))));
    }

    #[test]
    fn test_invalid_glyph_too_long() {
        let mut issue_type = create_valid_issuetype();
        issue_type.glyph = "TOOLONG".to_string();
        let result = issue_type.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidGlyph(_))));
    }

    #[test]
    fn test_no_steps() {
        let mut issue_type = create_valid_issuetype();
        issue_type.steps = vec![];
        let result = issue_type.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ValidationError::NoSteps)));
    }

    #[test]
    fn test_invalid_step_reference() {
        let mut issue_type = create_valid_issuetype();
        issue_type.steps[0].next_step = Some("nonexistent".to_string());
        let result = issue_type.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidStepRef(_))));
    }

    #[test]
    fn test_source_builtin() {
        let mut issue_type = create_valid_issuetype();
        issue_type.source = IssueTypeSource::Builtin;
        assert_eq!(issue_type.source_display(), "builtin");
    }

    #[test]
    fn test_source_user() {
        let issue_type = create_valid_issuetype();
        assert_eq!(issue_type.source_display(), "user");
    }

    #[test]
    fn test_source_import() {
        let mut issue_type = create_valid_issuetype();
        issue_type.source = IssueTypeSource::Import {
            provider: "jira".to_string(),
            project: "MYPROJ".to_string(),
        };
        assert_eq!(issue_type.source_display(), "jira/MYPROJ");
    }

    #[test]
    fn test_new_imported() {
        let issue_type = IssueType::new_imported(
            "BUG".to_string(),
            "Bug".to_string(),
            "A bug report".to_string(),
            "jira".to_string(),
            "MYPROJ".to_string(),
            Some("10001".to_string()),
        );

        assert_eq!(issue_type.key, "BUG");
        assert_eq!(issue_type.glyph, "B");
        assert!(issue_type.is_autonomous());
        assert_eq!(issue_type.steps.len(), 1);
        assert_eq!(issue_type.steps[0].name, "execute");
        assert!(matches!(
            &issue_type.source,
            IssueTypeSource::Import { provider, project }
            if provider == "jira" && project == "MYPROJ"
        ));
        assert_eq!(issue_type.external_id, Some("10001".to_string()));
        assert!(issue_type.validate().is_ok());
    }

    #[test]
    fn test_is_paired() {
        let mut issue_type = create_valid_issuetype();
        issue_type.mode = ExecutionMode::Paired;
        assert!(issue_type.is_paired());
        assert!(!issue_type.is_autonomous());
    }

    #[test]
    fn test_is_autonomous() {
        let issue_type = create_valid_issuetype();
        assert!(issue_type.is_autonomous());
        assert!(!issue_type.is_paired());
    }

    #[test]
    fn test_from_json() {
        let json = r#"{
            "key": "STORY",
            "name": "User Story",
            "description": "A user story",
            "mode": "autonomous",
            "glyph": "S",
            "color": "cyan",
            "project_required": true,
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
                    "name": "execute",
                    "outputs": [],
                    "prompt": "Execute",
                    "allowed_tools": ["*"]
                }
            ],
            "source": "user"
        }"#;

        let issue_type = IssueType::from_json(json).unwrap();
        assert_eq!(issue_type.key, "STORY");
        assert_eq!(issue_type.name, "User Story");
        assert_eq!(issue_type.source, IssueTypeSource::User);
        assert!(issue_type.validate().is_ok());
    }

    #[test]
    fn test_from_json_import_source() {
        let json = r#"{
            "key": "BUG",
            "name": "Bug",
            "description": "A bug",
            "mode": "autonomous",
            "glyph": "B",
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
                    "name": "execute",
                    "outputs": [],
                    "prompt": "Fix it",
                    "allowed_tools": ["*"]
                }
            ],
            "source": {
                "import": {
                    "provider": "jira",
                    "project": "MYPROJ"
                }
            },
            "external_id": "10001"
        }"#;

        let issue_type = IssueType::from_json(json).unwrap();
        assert_eq!(issue_type.key, "BUG");
        assert!(matches!(
            &issue_type.source,
            IssueTypeSource::Import { provider, project }
            if provider == "jira" && project == "MYPROJ"
        ));
        assert_eq!(issue_type.external_id, Some("10001".to_string()));
    }
}
