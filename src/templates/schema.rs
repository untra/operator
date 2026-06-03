#![allow(dead_code)]

//! Schema definitions for issuetype templates

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::permissions::{ProviderCliArgs, StepPermissions};

/// Schema definition for an issuetype template
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    /// Whether a project must be specified for this issuetype
    #[serde(default = "default_true")]
    pub project_required: bool,
    /// Field definitions for this template
    pub fields: Vec<FieldSchema>,
    /// Lifecycle steps for completing this ticket type
    pub steps: Vec<StepSchema>,
    /// Optional prompt for work launching (interpolated with handlebars)
    #[serde(default)]
    pub prompt: Option<String>,
    /// Prompt for generating this issue type's operator agent via `claude -p`
    #[serde(default)]
    pub agent_prompt: Option<String>,
    /// Default delegator name for this issuetype (overridden by step.agent)
    #[serde(default)]
    pub agent: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Execution mode for an issuetype
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    /// Runs without human interaction
    Autonomous,
    /// Requires human pairing/interaction
    Paired,
}

/// Schema definition for a single field in a template
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
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
    /// Integer number input
    Integer,
}

/// Schema definition for a lifecycle step
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StepSchema {
    // ── Common base (all step types) ────────────────────────────────
    /// Step identifier (lowercase)
    pub name: String,
    /// Human-readable step name
    #[serde(default)]
    pub display_name: Option<String>,
    /// Step type discriminator (defaults to "task" for backward compatibility)
    #[serde(default = "default_step_type", rename = "type")]
    pub step_type: StepTypeTag,
    /// Types of outputs this step produces
    pub outputs: Vec<StepOutput>,
    /// Initial prompt template for the Claude agent
    pub prompt: String,
    /// Type of review required for this step (none, plan, visual, pr)
    #[serde(default)]
    pub review_type: ReviewType,
    /// Configuration for visual review (required when `review_type` is "visual")
    #[serde(default)]
    pub visual_config: Option<VisualReviewConfig>,
    /// What to do if step output is rejected
    #[serde(default)]
    pub on_reject: Option<OnReject>,
    /// Name of the next step (None for final step)
    #[serde(default)]
    pub next_step: Option<String>,

    // ── Task fields (backward-compat, used when type=task) ──────────
    /// Claude Code tools allowed in this step
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Optional agent (delegator) name for this step (overrides ticket's default agent)
    #[serde(default)]
    pub agent: Option<String>,
    /// Provider-agnostic permissions for this step
    #[serde(default)]
    pub permissions: Option<StepPermissions>,
    /// Arbitrary CLI arguments per provider
    #[serde(default)]
    pub cli_args: Option<ProviderCliArgs>,
    /// Preferred LLM permission mode for this step
    #[serde(default)]
    pub permission_mode: PermissionMode,
    /// Inline JSON schema for structured output (Claude-specific)
    #[serde(default, rename = "jsonSchema")]
    pub json_schema: Option<serde_json::Value>,
    /// Path to JSON schema file for structured output (Claude-specific)
    #[serde(default, rename = "jsonSchemaFile")]
    pub json_schema_file: Option<String>,
    /// File glob patterns in the worktree that signal this step is complete
    #[serde(default)]
    pub artifact_patterns: Vec<String>,

    // ── Type-specific configs ───────────────────────────────────────
    /// Configuration for classifier steps (required when type=classifier)
    #[serde(default)]
    pub classifier_config: Option<ClassifierConfig>,
    /// Configuration for RAG steps (required when type=rag)
    #[serde(default)]
    pub rag_config: Option<RagConfig>,
    /// Configuration for delegator steps (required when type=delegator)
    #[serde(default)]
    pub delegator_config: Option<DelegatorStepConfig>,
    /// Configuration for MCP steps (required when type=mcp)
    #[serde(default)]
    pub mcp_config: Option<McpStepConfig>,
    /// Configuration for multi-model steps (required when `type=multi_model`)
    #[serde(default)]
    pub multi_model_config: Option<MultiModelConfig>,
    /// Configuration for multi-prompt steps (required when `type=multi_prompt`)
    #[serde(default)]
    pub multi_prompt_config: Option<MultiPromptConfig>,
    /// Configuration for matrixed steps (required when type=matrixed)
    #[serde(default)]
    pub matrixed_config: Option<MatrixedConfig>,
    /// Configuration for pipeline steps (required when type=pipeline)
    #[serde(default)]
    pub pipeline_config: Option<PipelineConfig>,
}

/// Status category for a step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum StepStatus {
    Todo,
    Doing,
    Await,
    Done,
}

/// Types of outputs a step can produce
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OnReject {
    /// Step name to return to on rejection
    pub goto_step: String,
    /// Prompt to use when restarting after rejection
    pub prompt: String,
}

/// Permission mode for LLM interaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    /// Default permission mode - standard interactive behavior
    #[default]
    Default,
    /// Plan mode - read-only exploration before implementation
    Plan,
    /// Accept edits mode - auto-approve file edits
    AcceptEdits,
    /// Delegate mode - task delegation with DAG management
    Delegate,
}

/// Type of review required for a step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReviewType {
    /// No review required - proceed automatically
    #[default]
    None,
    /// Review the plan/output before proceeding
    Plan,
    /// Visual confirmation via browser
    Visual,
    /// Git interface PR review workflow
    Pr,
}

/// Configuration for visual review steps
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VisualReviewConfig {
    /// URL to open for visual check (supports handlebars templates)
    pub url: String,
    /// Optional startup command (e.g., dev server) to run before opening browser
    #[serde(default)]
    pub startup_command: Option<String>,
    /// Timeout in seconds for server startup (default: 30)
    #[serde(default)]
    pub startup_timeout_secs: Option<u32>,
}

/// Discriminator tag for step types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StepTypeTag {
    /// Default pass-through task step
    #[default]
    Task,
    /// Structured typed output (boolean, number, string, enum)
    Classifier,
    /// Context-augmented prompting with retrieved sources
    Rag,
    /// Runs with a specific delegator and prompt flavor
    Delegator,
    /// Ensures specific MCP tools are available
    Mcp,
    /// Fan-out to N delegators, then aggregate via voting
    MultiModel,
    /// N prompt variations with one model, then select best
    MultiPrompt,
    /// N x M delegators x prompt variations
    Matrixed,
    /// Iterate a list of items through ordered stages with no barrier
    Pipeline,
}

fn default_step_type() -> StepTypeTag {
    StepTypeTag::Task
}

// ── Classifier ──────────────────────────────────────────────────────────

/// Configuration for classifier steps that return structured typed output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClassifierConfig {
    /// What type of answer the classifier returns
    pub output_type: ClassifierOutputType,
    /// For enum type: the allowed options
    #[serde(default)]
    pub options: Option<Vec<String>>,
    /// For `short_string`: max character length (default 255)
    #[serde(default)]
    pub max_length: Option<usize>,
    /// Agent/delegator to use (overrides issuetype default)
    #[serde(default)]
    pub agent: Option<String>,
}

/// Output types for classifier steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClassifierOutputType {
    /// true/false answer
    Boolean,
    /// Numeric answer (integer or float)
    Number,
    /// Short string < 255 chars
    ShortString,
    /// Longer arbitrary-length text
    BigText,
    /// One of a fixed set of options
    Enum,
}

// ── RAG ─────────────────────────────────────────────────────────────────

/// Configuration for RAG (retrieval-augmented generation) steps
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RagConfig {
    /// Context sources to retrieve before running the prompt
    pub sources: Vec<RagSource>,
    /// Maximum tokens of context to inject (default: 50000)
    #[serde(default)]
    pub max_context_tokens: Option<usize>,
    /// Agent/delegator to use
    #[serde(default)]
    pub agent: Option<String>,
    /// Tools allowed for the agent
    #[serde(default)]
    pub allowed_tools: Vec<String>,
}

/// A source of context for RAG steps
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RagSource {
    /// Match files by glob pattern
    Glob {
        /// Glob pattern relative to project root
        pattern: String,
    },
    /// Single file path
    File {
        /// File path relative to project root
        path: String,
    },
    /// Retrieve via MCP server tool
    Mcp {
        /// MCP server name
        server: String,
        /// Tool name on the MCP server
        tool: String,
        /// Optional query template (Handlebars)
        #[serde(default)]
        query: Option<String>,
    },
}

// ── Delegator Step ──────────────────────────────────────────────────────

/// Configuration for delegator steps that run with a specific model+flavor
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DelegatorStepConfig {
    /// Named delegator reference (from config.delegators)
    pub delegator: String,
    /// Additional prompt flavor text prepended to the step prompt
    #[serde(default)]
    pub prompt_flavor: Option<String>,
    /// Tools allowed
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Permissions
    #[serde(default)]
    pub permissions: Option<StepPermissions>,
}

// ── MCP Step ────────────────────────────────────────────────────────────

/// Configuration for MCP steps that require specific MCP tools
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpStepConfig {
    /// MCP tools that MUST be available (step fails if missing)
    pub required_tools: Vec<McpToolRef>,
    /// MCP tools that SHOULD be available (warning if missing)
    #[serde(default)]
    pub optional_tools: Vec<McpToolRef>,
    /// Agent/delegator to use
    #[serde(default)]
    pub agent: Option<String>,
    /// Tools allowed (in addition to MCP tools)
    #[serde(default)]
    pub allowed_tools: Vec<String>,
}

/// Reference to a specific MCP server tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpToolRef {
    /// MCP server name
    pub server: String,
    /// Specific tool name (None = all tools from this server)
    #[serde(default)]
    pub tool: Option<String>,
}

// ── Multi-Model ─────────────────────────────────────────────────────────

/// Configuration for multi-model delegation steps (fan-out + vote)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MultiModelConfig {
    /// Named delegator references (from config.delegators), minimum 2
    pub delegators: Vec<String>,
    /// How to aggregate/select the final answer
    pub voting_strategy: VotingStrategy,
    /// Whether to share all answers with all models in the voting round
    #[serde(default = "default_true")]
    pub share_answers: bool,
    /// Prompt for the voting round (Handlebars, receives {{ answers }} array)
    #[serde(default)]
    pub voting_prompt: Option<String>,
    /// How the voting round executes
    #[serde(default)]
    pub voting_mode: VotingMode,
}

/// Voting strategy for multi-model steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VotingStrategy {
    /// Simple majority vote
    Majority,
    /// Ranked choice voting
    Ranked,
    /// Unanimous required (falls back to longest answer if no consensus)
    Unanimous,
}

/// How the voting round is executed in multi-model steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VotingMode {
    /// One agent reviews all answers and picks winner (uses 1 slot)
    #[default]
    SingleJudge,
    /// All original delegators re-run with shared answers, each votes (uses N slots)
    MultiVoter,
}

// ── Multi-Prompt ────────────────────────────────────────────────────────

/// Configuration for multi-prompt interrogation steps (N variations, select best)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MultiPromptConfig {
    /// Prompt variations (Handlebars templates), minimum 2
    pub prompt_variations: Vec<String>,
    /// How to select the best result
    pub selection_strategy: SelectionStrategy,
    /// Agent/delegator to use for all variations
    #[serde(default)]
    pub agent: Option<String>,
    /// Prompt for the selection/review round
    #[serde(default)]
    pub selection_prompt: Option<String>,
}

/// Selection strategy for multi-prompt steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SelectionStrategy {
    /// Model reviews all outputs and picks the best
    ModelChoice,
    /// Model scores each and highest wins
    Scored,
}

// ── Matrixed ────────────────────────────────────────────────────────────

/// Configuration for matrixed work output steps (N x M delegators x prompts)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MatrixedConfig {
    /// Named delegator references (N), minimum 2
    pub delegators: Vec<String>,
    /// Prompt variations (M) — Handlebars templates, minimum 2
    pub prompt_variations: Vec<String>,
    /// How to organize/present the N x M output
    pub output_format: MatrixedOutputFormat,
    /// Optional aggregation prompt (receives the full matrix of results)
    #[serde(default)]
    pub aggregation_prompt: Option<String>,
}

/// Output format for matrixed steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatrixedOutputFormat {
    /// Each cell's output in `temp_dir/{delegator}/{prompt_index}/`
    Directory,
    /// Structured N x M JSON matrix in step output artifact
    Structured,
}

// ── Pipeline ────────────────────────────────────────────────────────────

/// Configuration for pipeline steps: iterate a list of items through ordered
/// stages with no barrier (each item flows through all stages independently).
///
/// The step graph stays linear — a pipeline step still has exactly one
/// `next_step`. The fan-out (N items x M stages) lives entirely inside this one
/// step; iteration is an intra-step concern, never a step-to-step edge.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineConfig {
    /// Where the iterated items come from.
    pub item_source: ItemSource,
    /// Ordered mini-steps each item flows through. Must be non-empty.
    pub stages: Vec<PipelineStage>,
}

/// A single stage in a pipeline — deliberately flat (not a recursive
/// `StepSchema`): "prompt + optional agent/model/schema" only. It has no
/// `next_step`/`review_type`/`on_reject`, so a stage cannot reopen the
/// step-graph linearity question.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineStage {
    /// Handlebars prompt. The per-item value is appended as a JS binding at
    /// export time (see `workflow_gen::export`), not via a Handlebars variable.
    pub prompt: String,
    /// Optional agent/delegator name (falls back to the step/issuetype agent).
    #[serde(default)]
    pub agent: Option<String>,
    /// Optional model pin (emitted as `{ model: … }`).
    #[serde(default)]
    pub model: Option<String>,
    /// Optional structured-output JSON schema (emitted as `{ schema: … }`).
    #[serde(default, rename = "jsonSchema")]
    pub json_schema: Option<serde_json::Value>,
    /// Optional display label override (defaults to `<step>:<stage-index>`).
    #[serde(default)]
    pub label: Option<String>,
}

/// Where a pipeline's iterated items come from. The variant determines *when*
/// the list resolves: export-time (a literal array → static fan-out width in
/// the compiled graph) vs runtime (an identifier → symbolic width).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemSource {
    /// The configured/relevant projects (`config.discover_projects()`),
    /// resolved to a literal array at export time. The "plan work across many
    /// projects" mechanism.
    Projects,
    /// An array produced by a prior step. Emits that step's result identifier
    /// (`r_<step>`) — a runtime value, so the graph width is symbolic.
    FromStep {
        /// Name of the prior step whose (array) output is iterated.
        step: String,
    },
    /// A glob pattern, expanded to a literal array at export time against the
    /// project root (`projects_path()/<ticket.project>`).
    Glob {
        /// Glob pattern, relative to the project root.
        pattern: String,
    },
    /// A literal, author-provided list, emitted verbatim as a literal array.
    Static {
        /// The items to iterate.
        items: Vec<String>,
    },
    /// A ticket field value split into a list. Resolution is deferred — there
    /// is no list `FieldType` and ticket field values are not captured at
    /// export time yet — so this currently emits a symbolic placeholder.
    Field {
        /// Name of the ticket field to read.
        name: String,
    },
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
            if field.required
                && field.auto.is_none()
                && field.name != "id"
                && field.default.is_none()
            {
                errors.push(format!(
                    "Required field '{}' must have a default value",
                    field.name
                ));
            }

            // Check enum fields have options
            if field.field_type == FieldType::Enum && field.options.is_empty() {
                errors.push(format!("Enum field '{}' must have options", field.name));
            }
        }

        // Check step transitions and type-specific config
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

            // A pipeline from_step source must name an existing step: the
            // export emits its result identifier (`r_<step>`), which would be
            // undefined at runtime otherwise.
            if let Some(ref cfg) = step.pipeline_config {
                if let ItemSource::FromStep { step: source } = &cfg.item_source {
                    if !source.is_empty() && !step_names.contains(&source.as_str()) {
                        errors.push(format!(
                            "Step '{}': pipeline item_source 'from_step' references unknown step '{}'",
                            step.name, source
                        ));
                    }
                }
            }

            // Validate step type config presence and constraints
            step.validate_type_config(&mut errors);
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
            StepStatus::Done
        } else if self.requires_review() {
            StepStatus::Await
        } else if is_first {
            StepStatus::Todo
        } else {
            StepStatus::Doing
        }
    }

    /// Check if this step requires any form of review
    pub fn requires_review(&self) -> bool {
        !matches!(self.review_type, ReviewType::None)
    }

    /// Check if this step outputs a plan
    pub fn outputs_plan(&self) -> bool {
        self.outputs.contains(&StepOutput::Plan)
    }

    /// Check if this step outputs a review
    pub fn outputs_review(&self) -> bool {
        self.outputs.contains(&StepOutput::Review)
    }

    /// Check if this step has artifact patterns for completion detection
    pub fn has_artifact_patterns(&self) -> bool {
        !self.artifact_patterns.is_empty()
    }

    /// Validate that the step type config is present and internally consistent
    pub fn validate_type_config(&self, errors: &mut Vec<String>) {
        match self.step_type {
            StepTypeTag::Task => {
                // Task steps don't require any specific config
            }
            StepTypeTag::Classifier => {
                if let Some(ref cfg) = self.classifier_config {
                    // Enum classifiers must have options
                    if cfg.output_type == ClassifierOutputType::Enum
                        && cfg.options.as_ref().is_none_or(Vec::is_empty)
                    {
                        errors.push(format!(
                            "Step '{}': classifier with output_type 'enum' must have non-empty options",
                            self.name
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Step '{}': type 'classifier' requires classifier_config",
                        self.name
                    ));
                }
            }
            StepTypeTag::Rag => {
                if let Some(ref cfg) = self.rag_config {
                    if cfg.sources.is_empty() {
                        errors.push(format!(
                            "Step '{}': rag_config must have at least one source",
                            self.name
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Step '{}': type 'rag' requires rag_config",
                        self.name
                    ));
                }
            }
            StepTypeTag::Delegator => {
                if self.delegator_config.is_none() {
                    errors.push(format!(
                        "Step '{}': type 'delegator' requires delegator_config",
                        self.name
                    ));
                }
            }
            StepTypeTag::Mcp => {
                if let Some(ref cfg) = self.mcp_config {
                    if cfg.required_tools.is_empty() && cfg.optional_tools.is_empty() {
                        errors.push(format!(
                            "Step '{}': mcp_config must have at least one required or optional tool",
                            self.name
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Step '{}': type 'mcp' requires mcp_config",
                        self.name
                    ));
                }
            }
            StepTypeTag::MultiModel => {
                if let Some(ref cfg) = self.multi_model_config {
                    if cfg.delegators.len() < 2 {
                        errors.push(format!(
                            "Step '{}': multi_model_config requires at least 2 delegators",
                            self.name
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Step '{}': type 'multi_model' requires multi_model_config",
                        self.name
                    ));
                }
            }
            StepTypeTag::MultiPrompt => {
                if let Some(ref cfg) = self.multi_prompt_config {
                    if cfg.prompt_variations.len() < 2 {
                        errors.push(format!(
                            "Step '{}': multi_prompt_config requires at least 2 prompt_variations",
                            self.name
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Step '{}': type 'multi_prompt' requires multi_prompt_config",
                        self.name
                    ));
                }
            }
            StepTypeTag::Matrixed => {
                if let Some(ref cfg) = self.matrixed_config {
                    if cfg.delegators.len() < 2 {
                        errors.push(format!(
                            "Step '{}': matrixed_config requires at least 2 delegators",
                            self.name
                        ));
                    }
                    if cfg.prompt_variations.len() < 2 {
                        errors.push(format!(
                            "Step '{}': matrixed_config requires at least 2 prompt_variations",
                            self.name
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Step '{}': type 'matrixed' requires matrixed_config",
                        self.name
                    ));
                }
            }
            StepTypeTag::Pipeline => {
                if let Some(ref cfg) = self.pipeline_config {
                    if cfg.stages.is_empty() {
                        errors.push(format!(
                            "Step '{}': pipeline_config.stages must be non-empty",
                            self.name
                        ));
                    }
                    match &cfg.item_source {
                        ItemSource::Static { items } if items.is_empty() => {
                            errors.push(format!(
                                "Step '{}': pipeline item_source 'static' must have a non-empty items list",
                                self.name
                            ));
                        }
                        ItemSource::Glob { pattern } if pattern.is_empty() => {
                            errors.push(format!(
                                "Step '{}': pipeline item_source 'glob' must have a non-empty pattern",
                                self.name
                            ));
                        }
                        ItemSource::FromStep { step } if step.is_empty() => {
                            errors.push(format!(
                                "Step '{}': pipeline item_source 'from_step' must name a step",
                                self.name
                            ));
                        }
                        ItemSource::Field { name } if name.is_empty() => {
                            errors.push(format!(
                                "Step '{}': pipeline item_source 'field' must name a field",
                                self.name
                            ));
                        }
                        _ => {}
                    }
                } else {
                    errors.push(format!(
                        "Step '{}': type 'pipeline' requires pipeline_config",
                        self.name
                    ));
                }
            }
        }
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
                    "review_type": "plan",
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
        assert!(schema.steps[1].requires_review());
        assert_eq!(schema.steps[1].review_type, ReviewType::Plan);

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

    #[test]
    fn test_permission_mode_default() {
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
                    "allowed_tools": ["Read"]
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].permission_mode, PermissionMode::Default);
    }

    #[test]
    fn test_permission_mode_plan() {
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
                    "permission_mode": "plan"
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].permission_mode, PermissionMode::Plan);
    }

    #[test]
    fn test_permission_mode_delegate() {
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
                    "name": "build",
                    "outputs": ["code"],
                    "prompt": "Build it",
                    "allowed_tools": ["Read", "Write"],
                    "permission_mode": "delegate"
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].permission_mode, PermissionMode::Delegate);
    }

    #[test]
    fn test_json_schema_inline() {
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
                    "name": "analyze",
                    "outputs": ["report"],
                    "prompt": "Analyze it",
                    "allowed_tools": ["Read"],
                    "jsonSchema": {
                        "type": "object",
                        "properties": {
                            "summary": { "type": "string" },
                            "score": { "type": "number" }
                        },
                        "required": ["summary"]
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert!(schema.steps[0].json_schema.is_some());
        let json_schema = schema.steps[0].json_schema.as_ref().unwrap();
        assert_eq!(json_schema["type"], "object");
        assert!(json_schema["properties"]["summary"].is_object());
    }

    #[test]
    fn test_json_schema_file() {
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
                    "name": "analyze",
                    "outputs": ["report"],
                    "prompt": "Analyze it",
                    "allowed_tools": ["Read"],
                    "jsonSchemaFile": "schemas/report.schema.json"
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert!(schema.steps[0].json_schema_file.is_some());
        assert_eq!(
            schema.steps[0].json_schema_file.as_ref().unwrap(),
            "schemas/report.schema.json"
        );
    }

    #[test]
    fn test_json_schema_default_none() {
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
                    "name": "build",
                    "outputs": ["code"],
                    "prompt": "Build it",
                    "allowed_tools": ["Read", "Write"]
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert!(schema.steps[0].json_schema.is_none());
        assert!(schema.steps[0].json_schema_file.is_none());
    }

    #[test]
    fn test_artifact_patterns_parsed_from_step() {
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
                    "artifact_patterns": [".tickets/plans/*.md"]
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(
            schema.steps[0].artifact_patterns,
            vec![".tickets/plans/*.md".to_string()]
        );
        assert!(schema.steps[0].has_artifact_patterns());
    }

    #[test]
    fn test_artifact_patterns_defaults_to_empty_vec() {
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
                    "name": "build",
                    "outputs": ["code"],
                    "prompt": "Build it",
                    "allowed_tools": ["Read", "Write"]
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert!(schema.steps[0].artifact_patterns.is_empty());
        assert!(!schema.steps[0].has_artifact_patterns());
    }

    #[test]
    fn test_artifact_patterns_multiple_patterns() {
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
                    "name": "build",
                    "outputs": ["code"],
                    "prompt": "Build it",
                    "allowed_tools": ["Read", "Write"],
                    "artifact_patterns": ["src/**/*.rs", "tests/**/*.rs", "Cargo.toml"]
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].artifact_patterns.len(), 3);
        assert_eq!(schema.steps[0].artifact_patterns[0], "src/**/*.rs");
        assert_eq!(schema.steps[0].artifact_patterns[1], "tests/**/*.rs");
        assert_eq!(schema.steps[0].artifact_patterns[2], "Cargo.toml");
        assert!(schema.steps[0].has_artifact_patterns());
    }

    #[test]
    fn test_step_agent_deserializes() {
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
                    "agent": "claude-opus"
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].agent, Some("claude-opus".to_string()));
    }

    #[test]
    fn test_step_agent_default_none() {
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
                    "name": "build",
                    "outputs": ["code"],
                    "prompt": "Build it",
                    "allowed_tools": ["Read", "Write"]
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert!(schema.steps[0].agent.is_none());
    }

    // ── Step type tests ─────────────────────────────────────────────

    #[test]
    fn test_step_type_defaults_to_task() {
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
                    "name": "execute",
                    "outputs": ["code"],
                    "prompt": "Do the thing",
                    "allowed_tools": ["Read"]
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].step_type, StepTypeTag::Task);
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_classifier_step_enum() {
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
                    "name": "classify",
                    "type": "classifier",
                    "outputs": ["report"],
                    "prompt": "Classify the severity",
                    "allowed_tools": [],
                    "classifier_config": {
                        "output_type": "enum",
                        "options": ["critical", "high", "medium", "low"]
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].step_type, StepTypeTag::Classifier);
        let cfg = schema.steps[0].classifier_config.as_ref().unwrap();
        assert_eq!(cfg.output_type, ClassifierOutputType::Enum);
        assert_eq!(cfg.options.as_ref().unwrap().len(), 4);
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_classifier_step_boolean() {
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
                    "name": "verify",
                    "type": "classifier",
                    "outputs": ["report"],
                    "prompt": "Does this pass the bar?",
                    "allowed_tools": [],
                    "classifier_config": {
                        "output_type": "boolean"
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        let cfg = schema.steps[0].classifier_config.as_ref().unwrap();
        assert_eq!(cfg.output_type, ClassifierOutputType::Boolean);
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_classifier_enum_missing_options_fails_validation() {
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
                    "name": "classify",
                    "type": "classifier",
                    "outputs": ["report"],
                    "prompt": "Classify it",
                    "allowed_tools": [],
                    "classifier_config": {
                        "output_type": "enum"
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        let result = schema.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("non-empty options"));
    }

    #[test]
    fn test_classifier_missing_config_fails_validation() {
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
                    "name": "classify",
                    "type": "classifier",
                    "outputs": ["report"],
                    "prompt": "Classify it",
                    "allowed_tools": []
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        let result = schema.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("requires classifier_config"));
    }

    #[test]
    fn test_rag_step() {
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
                    "name": "review",
                    "type": "rag",
                    "outputs": ["review"],
                    "prompt": "Review with context",
                    "allowed_tools": [],
                    "rag_config": {
                        "sources": [
                            { "type": "glob", "pattern": "docs/**/*.md" },
                            { "type": "file", "path": "ARCHITECTURE.md" },
                            { "type": "mcp", "server": "confluence", "tool": "search", "query": "test query" }
                        ],
                        "max_context_tokens": 50000
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].step_type, StepTypeTag::Rag);
        let cfg = schema.steps[0].rag_config.as_ref().unwrap();
        assert_eq!(cfg.sources.len(), 3);
        assert_eq!(cfg.max_context_tokens, Some(50000));
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_rag_missing_config_fails_validation() {
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
                    "name": "review",
                    "type": "rag",
                    "outputs": ["review"],
                    "prompt": "Review",
                    "allowed_tools": []
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        let result = schema.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("requires rag_config"));
    }

    #[test]
    fn test_multi_model_step() {
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
                    "name": "consensus",
                    "type": "multi_model",
                    "outputs": ["review"],
                    "prompt": "Review this PR",
                    "allowed_tools": [],
                    "multi_model_config": {
                        "delegators": ["claude-opus", "gemini-pro", "codex-high"],
                        "voting_strategy": "majority",
                        "share_answers": true
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].step_type, StepTypeTag::MultiModel);
        let cfg = schema.steps[0].multi_model_config.as_ref().unwrap();
        assert_eq!(cfg.delegators.len(), 3);
        assert_eq!(cfg.voting_strategy, VotingStrategy::Majority);
        assert!(cfg.share_answers);
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_multi_model_too_few_delegators_fails() {
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
                    "name": "consensus",
                    "type": "multi_model",
                    "outputs": ["review"],
                    "prompt": "Review",
                    "allowed_tools": [],
                    "multi_model_config": {
                        "delegators": ["only-one"],
                        "voting_strategy": "majority"
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        let result = schema.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("at least 2 delegators"));
    }

    #[test]
    fn test_multi_prompt_step() {
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
                    "name": "explore",
                    "type": "multi_prompt",
                    "outputs": ["plan"],
                    "prompt": "Base context",
                    "allowed_tools": [],
                    "multi_prompt_config": {
                        "prompt_variations": [
                            "Approach as refactoring",
                            "Approach as greenfield"
                        ],
                        "selection_strategy": "model_choice"
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].step_type, StepTypeTag::MultiPrompt);
        let cfg = schema.steps[0].multi_prompt_config.as_ref().unwrap();
        assert_eq!(cfg.prompt_variations.len(), 2);
        assert_eq!(cfg.selection_strategy, SelectionStrategy::ModelChoice);
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_matrixed_step() {
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
                    "name": "matrix",
                    "type": "matrixed",
                    "outputs": ["report"],
                    "prompt": "Analyze the codebase",
                    "allowed_tools": [],
                    "matrixed_config": {
                        "delegators": ["claude-opus", "gemini-pro"],
                        "prompt_variations": [
                            "Focus on performance",
                            "Focus on security",
                            "Focus on maintainability"
                        ],
                        "output_format": "structured"
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].step_type, StepTypeTag::Matrixed);
        let cfg = schema.steps[0].matrixed_config.as_ref().unwrap();
        assert_eq!(cfg.delegators.len(), 2);
        assert_eq!(cfg.prompt_variations.len(), 3);
        assert_eq!(cfg.output_format, MatrixedOutputFormat::Structured);
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_delegator_step() {
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
                    "name": "security_review",
                    "type": "delegator",
                    "outputs": ["review"],
                    "prompt": "Review for security",
                    "allowed_tools": [],
                    "delegator_config": {
                        "delegator": "claude-opus-security",
                        "prompt_flavor": "You are a security expert."
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].step_type, StepTypeTag::Delegator);
        let cfg = schema.steps[0].delegator_config.as_ref().unwrap();
        assert_eq!(cfg.delegator, "claude-opus-security");
        assert_eq!(
            cfg.prompt_flavor.as_deref(),
            Some("You are a security expert.")
        );
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_mcp_step() {
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
                    "name": "deploy",
                    "type": "mcp",
                    "outputs": ["report"],
                    "prompt": "Deploy infrastructure",
                    "allowed_tools": [],
                    "mcp_config": {
                        "required_tools": [
                            { "server": "terraform", "tool": "plan" },
                            { "server": "terraform", "tool": "apply" }
                        ],
                        "optional_tools": [
                            { "server": "slack" }
                        ]
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        assert_eq!(schema.steps[0].step_type, StepTypeTag::Mcp);
        let cfg = schema.steps[0].mcp_config.as_ref().unwrap();
        assert_eq!(cfg.required_tools.len(), 2);
        assert_eq!(cfg.required_tools[0].server, "terraform");
        assert_eq!(cfg.optional_tools.len(), 1);
        assert!(cfg.optional_tools[0].tool.is_none());
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_mcp_empty_tools_fails_validation() {
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
                    "name": "deploy",
                    "type": "mcp",
                    "outputs": ["report"],
                    "prompt": "Deploy",
                    "allowed_tools": [],
                    "mcp_config": {
                        "required_tools": [],
                        "optional_tools": []
                    }
                }
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        let result = schema.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("at least one required or optional tool"));
    }

    // ── Pipeline step validation ────────────────────────────────────

    /// Wrap a single pipeline step's JSON into a full template and validate it.
    fn validate_pipeline_step(step_json: &str) -> Result<(), Vec<String>> {
        let json = format!(
            r#"{{
                "key": "PIPE",
                "name": "Pipe",
                "description": "Pipeline template",
                "mode": "autonomous",
                "glyph": "*",
                "fields": [
                    {{ "name": "id", "description": "ID", "type": "string", "required": true, "auto": "id" }}
                ],
                "steps": [ {step_json} ]
            }}"#
        );
        TemplateSchema::from_json(&json).unwrap().validate()
    }

    #[test]
    fn test_pipeline_valid_static_config_passes() {
        let result = validate_pipeline_step(
            r#"{
                "name": "triage",
                "type": "pipeline",
                "outputs": ["report"],
                "prompt": "",
                "pipeline_config": {
                    "item_source": { "type": "static", "items": ["a", "b"] },
                    "stages": [ { "prompt": "Look at {{item}}" } ]
                }
            }"#,
        );
        assert!(result.is_ok(), "expected valid pipeline, got {result:?}");
    }

    #[test]
    fn test_pipeline_missing_config_errors() {
        let result = validate_pipeline_step(
            r#"{
                "name": "triage",
                "type": "pipeline",
                "outputs": ["report"],
                "prompt": ""
            }"#,
        );
        let errs = result.unwrap_err();
        assert!(
            errs.iter().any(|e| e.contains("requires pipeline_config")),
            "got {errs:?}"
        );
    }

    #[test]
    fn test_pipeline_empty_stages_errors() {
        let result = validate_pipeline_step(
            r#"{
                "name": "triage",
                "type": "pipeline",
                "outputs": ["report"],
                "prompt": "",
                "pipeline_config": {
                    "item_source": { "type": "projects" },
                    "stages": []
                }
            }"#,
        );
        let errs = result.unwrap_err();
        assert!(
            errs.iter().any(|e| e.contains("stages must be non-empty")),
            "got {errs:?}"
        );
    }

    #[test]
    fn test_pipeline_static_empty_items_errors() {
        let result = validate_pipeline_step(
            r#"{
                "name": "triage",
                "type": "pipeline",
                "outputs": ["report"],
                "prompt": "",
                "pipeline_config": {
                    "item_source": { "type": "static", "items": [] },
                    "stages": [ { "prompt": "x" } ]
                }
            }"#,
        );
        let errs = result.unwrap_err();
        assert!(
            errs.iter()
                .any(|e| e.contains("static") && e.contains("non-empty")),
            "got {errs:?}"
        );
    }

    #[test]
    fn test_pipeline_from_step_unknown_step_errors() {
        // Cross-referenced alongside next_step/on_reject target checks: the
        // emitted r_<step> identifier would be undefined at runtime otherwise.
        let result = validate_pipeline_step(
            r#"{
                "name": "fix",
                "type": "pipeline",
                "outputs": ["code"],
                "prompt": "",
                "pipeline_config": {
                    "item_source": { "type": "from_step", "step": "nonexistent" },
                    "stages": [ { "prompt": "x" } ]
                }
            }"#,
        );
        let errs = result.unwrap_err();
        assert!(
            errs.iter()
                .any(|e| e.contains("from_step") && e.contains("nonexistent")),
            "got {errs:?}"
        );
    }

    #[test]
    fn test_pipeline_glob_empty_pattern_errors() {
        let result = validate_pipeline_step(
            r#"{
                "name": "triage",
                "type": "pipeline",
                "outputs": ["report"],
                "prompt": "",
                "pipeline_config": {
                    "item_source": { "type": "glob", "pattern": "" },
                    "stages": [ { "prompt": "x" } ]
                }
            }"#,
        );
        let errs = result.unwrap_err();
        assert!(
            errs.iter()
                .any(|e| e.contains("glob") && e.contains("pattern")),
            "got {errs:?}"
        );
    }
}
