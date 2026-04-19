//! Step-type-specific prompt augmentation and config derivation
//!
//! Handles the differences between step types (task, classifier, rag, delegator, mcp)
//! by augmenting prompts and deriving configuration from type-specific configs.

use crate::templates::schema::{
    ClassifierConfig, ClassifierOutputType, DelegatorStepConfig, McpStepConfig, RagConfig,
    RagSource, StepSchema, StepTypeTag,
};

/// Generate a JSON schema from a classifier config's output type
pub fn classifier_json_schema(config: &ClassifierConfig) -> serde_json::Value {
    let value_schema = match config.output_type {
        ClassifierOutputType::Boolean => serde_json::json!({ "type": "boolean" }),
        ClassifierOutputType::Number => serde_json::json!({ "type": "number" }),
        ClassifierOutputType::ShortString => {
            let max_len = config.max_length.unwrap_or(255);
            serde_json::json!({ "type": "string", "maxLength": max_len })
        }
        ClassifierOutputType::BigText => serde_json::json!({ "type": "string" }),
        ClassifierOutputType::Enum => {
            if let Some(ref options) = config.options {
                serde_json::json!({ "type": "string", "enum": options })
            } else {
                serde_json::json!({ "type": "string" })
            }
        }
    };

    serde_json::json!({
        "type": "object",
        "required": ["value"],
        "additionalProperties": false,
        "properties": {
            "value": value_schema,
            "reasoning": {
                "type": "string",
                "description": "Brief explanation of why this value was chosen"
            }
        }
    })
}

/// Generate prompt augmentation text for a step based on its type.
/// Returns additional text to append to the step prompt, or empty string for task steps.
pub fn prompt_augmentation(step: &StepSchema) -> String {
    match step.step_type {
        StepTypeTag::Task => String::new(),

        StepTypeTag::Classifier => {
            if let Some(ref cfg) = step.classifier_config {
                classifier_prompt_augmentation(cfg)
            } else {
                String::new()
            }
        }

        StepTypeTag::Rag => {
            if let Some(ref cfg) = step.rag_config {
                rag_prompt_augmentation(cfg)
            } else {
                String::new()
            }
        }

        StepTypeTag::Delegator => {
            if let Some(ref cfg) = step.delegator_config {
                delegator_prompt_augmentation(cfg)
            } else {
                String::new()
            }
        }

        StepTypeTag::Mcp => {
            if let Some(ref cfg) = step.mcp_config {
                mcp_prompt_augmentation(cfg)
            } else {
                String::new()
            }
        }

        // Multi-agent types handled in Phase 4
        StepTypeTag::MultiModel | StepTypeTag::MultiPrompt | StepTypeTag::Matrixed => String::new(),
    }
}

/// Derive the effective `allowed_tools` for a step, accounting for type-specific configs
pub fn effective_allowed_tools(step: &StepSchema) -> &[String] {
    match step.step_type {
        StepTypeTag::Rag => {
            if let Some(ref cfg) = step.rag_config {
                if !cfg.allowed_tools.is_empty() {
                    return &cfg.allowed_tools;
                }
            }
            &step.allowed_tools
        }
        StepTypeTag::Delegator => {
            if let Some(ref cfg) = step.delegator_config {
                if !cfg.allowed_tools.is_empty() {
                    return &cfg.allowed_tools;
                }
            }
            &step.allowed_tools
        }
        StepTypeTag::Mcp => {
            if let Some(ref cfg) = step.mcp_config {
                if !cfg.allowed_tools.is_empty() {
                    return &cfg.allowed_tools;
                }
            }
            &step.allowed_tools
        }
        _ => &step.allowed_tools,
    }
}

/// Derive the effective agent (delegator) name for a step
pub fn effective_agent(step: &StepSchema) -> Option<&str> {
    match step.step_type {
        StepTypeTag::Classifier => step
            .classifier_config
            .as_ref()
            .and_then(|c| c.agent.as_deref())
            .or(step.agent.as_deref()),
        StepTypeTag::Rag => step
            .rag_config
            .as_ref()
            .and_then(|c| c.agent.as_deref())
            .or(step.agent.as_deref()),
        StepTypeTag::Delegator => step.delegator_config.as_ref().map(|c| c.delegator.as_str()),
        StepTypeTag::Mcp => step
            .mcp_config
            .as_ref()
            .and_then(|c| c.agent.as_deref())
            .or(step.agent.as_deref()),
        _ => step.agent.as_deref(),
    }
}

// ── Per-type prompt augmentation ────────────────────────────────────────

fn classifier_prompt_augmentation(cfg: &ClassifierConfig) -> String {
    let type_desc = match cfg.output_type {
        ClassifierOutputType::Boolean => "a boolean (true or false)".to_string(),
        ClassifierOutputType::Number => "a number".to_string(),
        ClassifierOutputType::ShortString => {
            let max = cfg.max_length.unwrap_or(255);
            format!("a short string (max {max} characters)")
        }
        ClassifierOutputType::BigText => "a text string of any length".to_string(),
        ClassifierOutputType::Enum => {
            if let Some(ref opts) = cfg.options {
                format!("one of: {}", opts.join(", "))
            } else {
                "one of the given options".to_string()
            }
        }
    };

    format!(
        "\n\n## Structured Output Required\n\
         Your response MUST be valid JSON matching the provided schema.\n\
         The `value` field must be {type_desc}.\n\
         Include a brief `reasoning` field explaining your choice."
    )
}

fn rag_prompt_augmentation(cfg: &RagConfig) -> String {
    let mut parts = vec!["\n\n## Retrieved Context".to_string()];
    parts.push("The following context sources have been loaded for this step:".to_string());

    for source in &cfg.sources {
        match source {
            RagSource::Glob { pattern } => {
                parts.push(format!("- Files matching `{pattern}`"));
            }
            RagSource::File { path } => {
                parts.push(format!("- File: `{path}`"));
            }
            RagSource::Mcp { server, tool, .. } => {
                parts.push(format!("- MCP retrieval: {server}/{tool}"));
            }
        }
    }

    if let Some(max_tokens) = cfg.max_context_tokens {
        parts.push(format!(
            "\nContext is limited to approximately {max_tokens} tokens."
        ));
    }

    parts.join("\n")
}

fn delegator_prompt_augmentation(cfg: &DelegatorStepConfig) -> String {
    match &cfg.prompt_flavor {
        Some(flavor) => format!("\n\n## Role\n{flavor}"),
        None => String::new(),
    }
}

fn mcp_prompt_augmentation(cfg: &McpStepConfig) -> String {
    let mut parts = vec!["\n\n## Available MCP Tools".to_string()];

    if !cfg.required_tools.is_empty() {
        parts.push("Required tools (use these to complete the step):".to_string());
        for tool_ref in &cfg.required_tools {
            if let Some(ref tool_name) = tool_ref.tool {
                parts.push(format!("- `{}/{tool_name}`", tool_ref.server));
            } else {
                parts.push(format!("- All tools from `{}`", tool_ref.server));
            }
        }
    }

    if !cfg.optional_tools.is_empty() {
        parts.push("Optional tools (available if needed):".to_string());
        for tool_ref in &cfg.optional_tools {
            if let Some(ref tool_name) = tool_ref.tool {
                parts.push(format!("- `{}/{tool_name}`", tool_ref.server));
            } else {
                parts.push(format!("- All tools from `{}`", tool_ref.server));
            }
        }
    }

    parts.join("\n")
}

// ── Multi-agent aggregation ─────────────────────────────────────────

use crate::templates::schema::{
    MatrixedConfig, MatrixedOutputFormat, MultiModelConfig, MultiPromptConfig, VotingStrategy,
};

/// Aggregate multi-model outputs into the final step output artifact.
///
/// `outputs` maps delegator name -> response value (typically a string or structured JSON).
/// Returns the structured output artifact with responses, votes, and winner.
pub fn aggregate_multi_model(
    outputs: &HashMap<String, serde_json::Value>,
    config: &MultiModelConfig,
) -> serde_json::Value {
    let responses: Vec<serde_json::Value> = config
        .delegators
        .iter()
        .map(|d| {
            let response = outputs.get(d).cloned().unwrap_or(serde_json::Value::Null);
            serde_json::json!({
                "delegator": d,
                "response": response,
            })
        })
        .collect();

    // For now, voting is represented as a placeholder structure.
    // Actual voting requires a Phase 2 agent round — the votes will be
    // filled in by the sync loop after the voting phase completes.
    // Here we select the winner based on strategy from the raw outputs.
    let (winner_index, winner_delegator) = select_winner_by_strategy(outputs, config);

    let winner_response = outputs
        .get(&winner_delegator)
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    serde_json::json!({
        "type": "multi_model",
        "responses": responses,
        "winner_index": winner_index,
        "winner_delegator": winner_delegator,
        "winner_response": winner_response,
        "value": winner_response,
    })
}

/// Select winner from raw outputs using the voting strategy (pre-voting fallback).
fn select_winner_by_strategy(
    outputs: &HashMap<String, serde_json::Value>,
    config: &MultiModelConfig,
) -> (usize, String) {
    match config.voting_strategy {
        VotingStrategy::Majority | VotingStrategy::Ranked => {
            // Without actual votes, fall back to first delegator with output
            for (i, d) in config.delegators.iter().enumerate() {
                if outputs.contains_key(d) {
                    return (i, d.clone());
                }
            }
            (0, config.delegators.first().cloned().unwrap_or_default())
        }
        VotingStrategy::Unanimous => {
            // Fall back to the longest response
            let mut best = (0, config.delegators.first().cloned().unwrap_or_default());
            let mut best_len = 0;
            for (i, d) in config.delegators.iter().enumerate() {
                if let Some(val) = outputs.get(d) {
                    let len = val.as_str().map_or(0, str::len);
                    if len > best_len {
                        best_len = len;
                        best = (i, d.clone());
                    }
                }
            }
            best
        }
    }
}

/// Apply vote results to a multi-model aggregation.
///
/// `votes` maps voter (delegator name) -> chosen index.
/// Updates the artifact with vote data and recalculates the winner.
pub fn apply_votes(
    base: &mut serde_json::Value,
    votes: &HashMap<String, usize>,
    config: &MultiModelConfig,
) {
    let vote_array: Vec<serde_json::Value> = votes
        .iter()
        .map(|(voter, choice)| {
            serde_json::json!({
                "voter": voter,
                "choice": choice,
            })
        })
        .collect();

    base["votes"] = serde_json::json!(vote_array);

    // Tally votes
    let mut tallies: HashMap<usize, usize> = HashMap::new();
    for choice in votes.values() {
        *tallies.entry(*choice).or_insert(0) += 1;
    }

    // Find winner by highest vote count
    if let Some((&winner_idx, _)) = tallies.iter().max_by_key(|(_, count)| *count) {
        if winner_idx < config.delegators.len() {
            base["winner_index"] = serde_json::json!(winner_idx);
            base["winner_delegator"] = serde_json::json!(config.delegators[winner_idx]);
            if let Some(responses) = base["responses"].as_array() {
                if let Some(winner) = responses.get(winner_idx) {
                    let resp = winner["response"].clone();
                    base["winner_response"] = resp.clone();
                    base["value"] = resp;
                }
            }
        }
    }
}

/// Aggregate multi-prompt outputs into the final step output artifact.
///
/// `outputs` maps a key (prompt index as string) -> response value.
pub fn aggregate_multi_prompt(
    outputs: &HashMap<String, serde_json::Value>,
    config: &MultiPromptConfig,
) -> serde_json::Value {
    let variations: Vec<serde_json::Value> = config
        .prompt_variations
        .iter()
        .enumerate()
        .map(|(i, prompt)| {
            let key = i.to_string();
            let response = outputs
                .get(&key)
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            // Truncate prompt for summary (first 80 chars)
            let summary: String = prompt.chars().take(80).collect();
            serde_json::json!({
                "prompt_index": i,
                "prompt_summary": summary,
                "response": response,
            })
        })
        .collect();

    // Default to first variation; actual selection requires a Phase 2 agent round
    let selected_index = 0;
    let selected_response = outputs.get("0").cloned().unwrap_or(serde_json::Value::Null);

    serde_json::json!({
        "type": "multi_prompt",
        "variations": variations,
        "selected_index": selected_index,
        "selected_response": selected_response,
        "value": selected_response,
    })
}

/// Apply selection result to a multi-prompt aggregation.
pub fn apply_selection(base: &mut serde_json::Value, selected_index: usize) {
    base["selected_index"] = serde_json::json!(selected_index);
    if let Some(variations) = base["variations"].as_array() {
        if let Some(selected) = variations.get(selected_index) {
            let resp = selected["response"].clone();
            base["selected_response"] = resp.clone();
            base["value"] = resp;
        }
    }
}

/// Aggregate matrixed outputs into the final step output artifact.
///
/// `outputs` maps a compound key "`{delegator}:{prompt_index}`" -> response value.
pub fn aggregate_matrixed(
    outputs: &HashMap<String, serde_json::Value>,
    config: &MatrixedConfig,
    step_name: &str,
) -> serde_json::Value {
    let matrix: Vec<Vec<serde_json::Value>> = config
        .delegators
        .iter()
        .map(|delegator| {
            config
                .prompt_variations
                .iter()
                .enumerate()
                .map(|(j, _)| {
                    let key = format!("{delegator}:{j}");
                    let response = outputs
                        .get(&key)
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    let temp_file = format!(".tickets/steps/{step_name}/{delegator}/{j}.md");
                    serde_json::json!({
                        "delegator": delegator,
                        "prompt_index": j,
                        "response": response,
                        "temp_file": temp_file,
                    })
                })
                .collect()
        })
        .collect();

    let temp_dir = format!(".tickets/steps/{step_name}/");
    let output_format = match config.output_format {
        MatrixedOutputFormat::Directory => "directory",
        MatrixedOutputFormat::Structured => "structured",
    };

    serde_json::json!({
        "type": "matrixed",
        "delegators": config.delegators,
        "prompt_variations": config.prompt_variations,
        "output_format": output_format,
        "matrix": matrix,
        "temp_dir": temp_dir,
        "aggregated_result": null,
        "value": null,
    })
}

/// Apply aggregation result to a matrixed output.
pub fn apply_aggregation(base: &mut serde_json::Value, result: serde_json::Value) {
    base["aggregated_result"] = result.clone();
    base["value"] = result;
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::schema::*;

    fn make_base_step(step_type: StepTypeTag) -> StepSchema {
        StepSchema {
            name: "test".to_string(),
            display_name: None,
            step_type,
            outputs: vec![],
            prompt: "Test prompt".to_string(),
            review_type: ReviewType::None,
            visual_config: None,
            on_reject: None,
            next_step: None,
            allowed_tools: vec!["Read".to_string()],
            agent: None,
            permissions: None,
            cli_args: None,
            permission_mode: PermissionMode::Default,
            json_schema: None,
            json_schema_file: None,
            artifact_patterns: vec![],
            classifier_config: None,
            rag_config: None,
            delegator_config: None,
            mcp_config: None,
            multi_model_config: None,
            multi_prompt_config: None,
            matrixed_config: None,
        }
    }

    // ── classifier_json_schema tests ────────────────────────────────

    #[test]
    fn test_classifier_schema_boolean() {
        let cfg = ClassifierConfig {
            output_type: ClassifierOutputType::Boolean,
            options: None,
            max_length: None,
            agent: None,
        };
        let schema = classifier_json_schema(&cfg);
        assert_eq!(schema["properties"]["value"]["type"], "boolean");
    }

    #[test]
    fn test_classifier_schema_number() {
        let cfg = ClassifierConfig {
            output_type: ClassifierOutputType::Number,
            options: None,
            max_length: None,
            agent: None,
        };
        let schema = classifier_json_schema(&cfg);
        assert_eq!(schema["properties"]["value"]["type"], "number");
    }

    #[test]
    fn test_classifier_schema_short_string_with_max() {
        let cfg = ClassifierConfig {
            output_type: ClassifierOutputType::ShortString,
            options: None,
            max_length: Some(100),
            agent: None,
        };
        let schema = classifier_json_schema(&cfg);
        assert_eq!(schema["properties"]["value"]["type"], "string");
        assert_eq!(schema["properties"]["value"]["maxLength"], 100);
    }

    #[test]
    fn test_classifier_schema_enum_with_options() {
        let cfg = ClassifierConfig {
            output_type: ClassifierOutputType::Enum,
            options: Some(vec![
                "low".to_string(),
                "medium".to_string(),
                "high".to_string(),
            ]),
            max_length: None,
            agent: None,
        };
        let schema = classifier_json_schema(&cfg);
        let enum_vals = schema["properties"]["value"]["enum"].as_array().unwrap();
        assert_eq!(enum_vals.len(), 3);
        assert_eq!(enum_vals[0], "low");
    }

    #[test]
    fn test_classifier_schema_has_required_value() {
        let cfg = ClassifierConfig {
            output_type: ClassifierOutputType::Boolean,
            options: None,
            max_length: None,
            agent: None,
        };
        let schema = classifier_json_schema(&cfg);
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("value")));
    }

    // ── prompt_augmentation tests ───────────────────────────────────

    #[test]
    fn test_task_no_augmentation() {
        let step = make_base_step(StepTypeTag::Task);
        assert!(prompt_augmentation(&step).is_empty());
    }

    #[test]
    fn test_classifier_augmentation_contains_type() {
        let mut step = make_base_step(StepTypeTag::Classifier);
        step.classifier_config = Some(ClassifierConfig {
            output_type: ClassifierOutputType::Enum,
            options: Some(vec!["a".to_string(), "b".to_string()]),
            max_length: None,
            agent: None,
        });
        let aug = prompt_augmentation(&step);
        assert!(aug.contains("Structured Output Required"));
        assert!(aug.contains("one of: a, b"));
    }

    #[test]
    fn test_rag_augmentation_lists_sources() {
        let mut step = make_base_step(StepTypeTag::Rag);
        step.rag_config = Some(RagConfig {
            sources: vec![
                RagSource::Glob {
                    pattern: "docs/**/*.md".to_string(),
                },
                RagSource::File {
                    path: "README.md".to_string(),
                },
            ],
            max_context_tokens: Some(30000),
            agent: None,
            allowed_tools: vec![],
        });
        let aug = prompt_augmentation(&step);
        assert!(aug.contains("docs/**/*.md"));
        assert!(aug.contains("README.md"));
        assert!(aug.contains("30000 tokens"));
    }

    #[test]
    fn test_delegator_augmentation_includes_flavor() {
        let mut step = make_base_step(StepTypeTag::Delegator);
        step.delegator_config = Some(DelegatorStepConfig {
            delegator: "claude-opus".to_string(),
            prompt_flavor: Some("You are a security expert.".to_string()),
            allowed_tools: vec![],
            permissions: None,
        });
        let aug = prompt_augmentation(&step);
        assert!(aug.contains("You are a security expert."));
    }

    #[test]
    fn test_mcp_augmentation_lists_tools() {
        let mut step = make_base_step(StepTypeTag::Mcp);
        step.mcp_config = Some(McpStepConfig {
            required_tools: vec![McpToolRef {
                server: "terraform".to_string(),
                tool: Some("plan".to_string()),
            }],
            optional_tools: vec![McpToolRef {
                server: "slack".to_string(),
                tool: None,
            }],
            agent: None,
            allowed_tools: vec![],
        });
        let aug = prompt_augmentation(&step);
        assert!(aug.contains("terraform/plan"));
        assert!(aug.contains("All tools from `slack`"));
    }

    // ── effective_agent tests ───────────────────────────────────────

    #[test]
    fn test_effective_agent_task_uses_step_agent() {
        let mut step = make_base_step(StepTypeTag::Task);
        step.agent = Some("claude-opus".to_string());
        assert_eq!(effective_agent(&step), Some("claude-opus"));
    }

    #[test]
    fn test_effective_agent_classifier_prefers_config() {
        let mut step = make_base_step(StepTypeTag::Classifier);
        step.agent = Some("fallback".to_string());
        step.classifier_config = Some(ClassifierConfig {
            output_type: ClassifierOutputType::Boolean,
            options: None,
            max_length: None,
            agent: Some("classifier-model".to_string()),
        });
        assert_eq!(effective_agent(&step), Some("classifier-model"));
    }

    #[test]
    fn test_effective_agent_classifier_falls_back_to_step_agent() {
        let mut step = make_base_step(StepTypeTag::Classifier);
        step.agent = Some("fallback".to_string());
        step.classifier_config = Some(ClassifierConfig {
            output_type: ClassifierOutputType::Boolean,
            options: None,
            max_length: None,
            agent: None,
        });
        assert_eq!(effective_agent(&step), Some("fallback"));
    }

    #[test]
    fn test_effective_agent_delegator_uses_config_delegator() {
        let mut step = make_base_step(StepTypeTag::Delegator);
        step.agent = Some("should-not-use".to_string());
        step.delegator_config = Some(DelegatorStepConfig {
            delegator: "claude-opus-security".to_string(),
            prompt_flavor: None,
            allowed_tools: vec![],
            permissions: None,
        });
        assert_eq!(effective_agent(&step), Some("claude-opus-security"));
    }

    // ── effective_allowed_tools tests ───────────────────────────────

    #[test]
    fn test_effective_tools_task_uses_step_tools() {
        let step = make_base_step(StepTypeTag::Task);
        assert_eq!(effective_allowed_tools(&step), &["Read".to_string()]);
    }

    #[test]
    fn test_effective_tools_rag_prefers_config() {
        let mut step = make_base_step(StepTypeTag::Rag);
        step.rag_config = Some(RagConfig {
            sources: vec![],
            max_context_tokens: None,
            agent: None,
            allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
        });
        let tools = effective_allowed_tools(&step);
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[1], "Grep");
    }

    #[test]
    fn test_effective_tools_rag_falls_back_to_step() {
        let mut step = make_base_step(StepTypeTag::Rag);
        step.rag_config = Some(RagConfig {
            sources: vec![],
            max_context_tokens: None,
            agent: None,
            allowed_tools: vec![],
        });
        assert_eq!(effective_allowed_tools(&step), &["Read".to_string()]);
    }

    // ── Aggregation tests ──────────────────────────────────────────

    fn make_multi_model_config() -> MultiModelConfig {
        MultiModelConfig {
            delegators: vec![
                "claude-opus".to_string(),
                "gemini-pro".to_string(),
                "codex-high".to_string(),
            ],
            voting_strategy: VotingStrategy::Majority,
            share_answers: true,
            voting_prompt: None,
            voting_mode: VotingMode::default(),
        }
    }

    #[test]
    fn test_aggregate_multi_model_collects_responses() {
        let config = make_multi_model_config();
        let mut outputs = HashMap::new();
        outputs.insert(
            "claude-opus".to_string(),
            serde_json::json!("Use approach A"),
        );
        outputs.insert(
            "gemini-pro".to_string(),
            serde_json::json!("Use approach B"),
        );
        outputs.insert(
            "codex-high".to_string(),
            serde_json::json!("Use approach A"),
        );

        let result = aggregate_multi_model(&outputs, &config);
        assert_eq!(result["type"], "multi_model");
        assert_eq!(result["responses"].as_array().unwrap().len(), 3);
        assert!(result["winner_delegator"].is_string());
        assert!(!result["winner_response"].is_null());
    }

    #[test]
    fn test_aggregate_multi_model_unanimous_picks_longest() {
        let config = MultiModelConfig {
            delegators: vec!["a".to_string(), "b".to_string()],
            voting_strategy: VotingStrategy::Unanimous,
            share_answers: false,
            voting_prompt: None,
            voting_mode: VotingMode::default(),
        };
        let mut outputs = HashMap::new();
        outputs.insert("a".to_string(), serde_json::json!("short"));
        outputs.insert(
            "b".to_string(),
            serde_json::json!("this is a much longer response"),
        );

        let result = aggregate_multi_model(&outputs, &config);
        assert_eq!(result["winner_delegator"], "b");
        assert_eq!(result["winner_index"], 1);
    }

    #[test]
    fn test_apply_votes_updates_winner() {
        let config = make_multi_model_config();
        let mut outputs = HashMap::new();
        outputs.insert("claude-opus".to_string(), serde_json::json!("A"));
        outputs.insert("gemini-pro".to_string(), serde_json::json!("B"));
        outputs.insert("codex-high".to_string(), serde_json::json!("C"));

        let mut result = aggregate_multi_model(&outputs, &config);

        let mut votes = HashMap::new();
        votes.insert("claude-opus".to_string(), 1usize); // votes for gemini
        votes.insert("gemini-pro".to_string(), 1usize); // votes for gemini
        votes.insert("codex-high".to_string(), 0usize); // votes for claude

        apply_votes(&mut result, &votes, &config);
        assert_eq!(result["winner_index"], 1);
        assert_eq!(result["winner_delegator"], "gemini-pro");
        assert_eq!(result["votes"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_aggregate_multi_prompt() {
        let config = MultiPromptConfig {
            prompt_variations: vec![
                "Approach as refactoring".to_string(),
                "Approach as greenfield".to_string(),
            ],
            selection_strategy: SelectionStrategy::ModelChoice,
            agent: None,
            selection_prompt: None,
        };
        let mut outputs = HashMap::new();
        outputs.insert("0".to_string(), serde_json::json!("refactoring plan"));
        outputs.insert("1".to_string(), serde_json::json!("greenfield plan"));

        let result = aggregate_multi_prompt(&outputs, &config);
        assert_eq!(result["type"], "multi_prompt");
        assert_eq!(result["variations"].as_array().unwrap().len(), 2);
        assert_eq!(result["selected_index"], 0); // default
    }

    #[test]
    fn test_apply_selection() {
        let config = MultiPromptConfig {
            prompt_variations: vec!["A".to_string(), "B".to_string()],
            selection_strategy: SelectionStrategy::ModelChoice,
            agent: None,
            selection_prompt: None,
        };
        let mut outputs = HashMap::new();
        outputs.insert("0".to_string(), serde_json::json!("plan A"));
        outputs.insert("1".to_string(), serde_json::json!("plan B"));

        let mut result = aggregate_multi_prompt(&outputs, &config);
        apply_selection(&mut result, 1);

        assert_eq!(result["selected_index"], 1);
        assert_eq!(result["selected_response"], "plan B");
        assert_eq!(result["value"], "plan B");
    }

    #[test]
    fn test_aggregate_matrixed() {
        let config = MatrixedConfig {
            delegators: vec!["claude".to_string(), "gemini".to_string()],
            prompt_variations: vec!["perf".to_string(), "security".to_string()],
            output_format: MatrixedOutputFormat::Structured,
            aggregation_prompt: None,
        };
        let mut outputs = HashMap::new();
        outputs.insert("claude:0".to_string(), serde_json::json!("claude-perf"));
        outputs.insert("claude:1".to_string(), serde_json::json!("claude-sec"));
        outputs.insert("gemini:0".to_string(), serde_json::json!("gemini-perf"));
        outputs.insert("gemini:1".to_string(), serde_json::json!("gemini-sec"));

        let result = aggregate_matrixed(&outputs, &config, "analysis");
        assert_eq!(result["type"], "matrixed");
        let matrix = result["matrix"].as_array().unwrap();
        assert_eq!(matrix.len(), 2); // 2 delegators
        assert_eq!(matrix[0].as_array().unwrap().len(), 2); // 2 prompts each
        assert_eq!(matrix[0][0]["response"], "claude-perf");
        assert_eq!(matrix[1][1]["response"], "gemini-sec");
    }

    #[test]
    fn test_apply_aggregation() {
        let config = MatrixedConfig {
            delegators: vec!["a".to_string(), "b".to_string()],
            prompt_variations: vec!["x".to_string(), "y".to_string()],
            output_format: MatrixedOutputFormat::Structured,
            aggregation_prompt: Some("Synthesize".to_string()),
        };
        let outputs = HashMap::new();
        let mut result = aggregate_matrixed(&outputs, &config, "test");
        assert!(result["aggregated_result"].is_null());

        apply_aggregation(&mut result, serde_json::json!("synthesized answer"));
        assert_eq!(result["aggregated_result"], "synthesized answer");
        assert_eq!(result["value"], "synthesized answer");
    }
}
