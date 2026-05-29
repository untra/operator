#![allow(dead_code)]

//! Step manager for handling workflow step transitions

use anyhow::{Context, Result};
use handlebars::Handlebars;

use crate::api::GitHubClient;
use crate::config::Config;
use crate::pr_config::PrConfig;
use crate::queue::Ticket;
use crate::templates::schema::{StepOutput, StepSchema, StepStatus, TemplateSchema};
use crate::templates::TemplateType;

/// Manages step transitions for tickets
pub struct StepManager {
    config: Config,
}

impl StepManager {
    /// Create a new step manager
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Get the template schema for a ticket type
    pub fn get_template(&self, ticket_type: &str) -> Option<TemplateSchema> {
        TemplateType::from_key(ticket_type)
            .and_then(|tt| TemplateSchema::from_json(tt.schema()).ok())
    }

    /// Get the current step schema for a ticket
    pub fn current_step(&self, ticket: &Ticket) -> Option<StepSchema> {
        let template = self.get_template(&ticket.ticket_type)?;
        let step_name = if ticket.step.is_empty() {
            template.first_step()?.name.clone()
        } else {
            ticket.step.clone()
        };
        template.get_step(&step_name).cloned()
    }

    /// Get the next step schema after the current step
    pub fn next_step(&self, ticket: &Ticket) -> Option<StepSchema> {
        let current = self.current_step(ticket)?;
        let next_name = current.next_step.as_ref()?;
        let template = self.get_template(&ticket.ticket_type)?;
        template.get_step(next_name).cloned()
    }

    /// Get all steps for a ticket type
    pub fn all_steps(&self, ticket: &Ticket) -> Vec<StepSchema> {
        self.get_template(&ticket.ticket_type)
            .map(|t| t.steps)
            .unwrap_or_default()
    }

    /// Get the first step for a ticket type
    pub fn first_step(&self, ticket_type: &str) -> Option<StepSchema> {
        self.get_template(ticket_type)?.first_step().cloned()
    }

    /// Check if step can proceed (e.g., PR approved for "pr" step)
    /// Returns true if:
    /// - Step doesn't require review, OR
    /// - Step requires review and is approved (for PR steps, checks GitHub)
    pub async fn can_proceed(
        &self,
        ticket: &Ticket,
        github: Option<&GitHubClient>,
        pr_config: Option<&PrConfig>,
        pr_number: Option<u64>,
    ) -> Result<bool> {
        let current = match self.current_step(ticket) {
            Some(s) => s,
            None => return Ok(true),
        };

        if !current.requires_review() {
            return Ok(true);
        }

        // If this is a PR step, check GitHub for approval
        if self.is_pr_step(ticket) {
            if let (Some(gh), Some(config), Some(pr_num)) = (github, pr_config, pr_number) {
                if let Some(ref repo) = config.github_repo {
                    if let Some((owner, repo_name)) = GitHubClient::parse_repo(repo) {
                        return gh.is_pr_ready_to_merge(owner, repo_name, pr_num).await;
                    }
                }
            }
        }

        // For non-PR review steps, assume human approval is needed
        Ok(false)
    }

    /// Get the name of the next step (without schema)
    pub fn next_step_name(&self, ticket: &Ticket) -> Option<String> {
        self.current_step(ticket)?.next_step
    }

    /// Check if current step is a PR step
    pub fn is_pr_step(&self, ticket: &Ticket) -> bool {
        self.current_step(ticket)
            .is_some_and(|s| s.outputs.contains(&StepOutput::Pr))
    }

    /// Check if current step requires human review
    pub fn requires_review(&self, ticket: &Ticket) -> bool {
        self.current_step(ticket)
            .is_some_and(|s| s.requires_review())
    }

    /// Check if current step is the final step
    pub fn is_final_step(&self, ticket: &Ticket) -> bool {
        self.current_step(ticket)
            .is_none_or(|s| s.next_step.is_none())
    }

    /// Get the status category for the current step based on step properties
    pub fn current_status(&self, ticket: &Ticket) -> StepStatus {
        let template = match self.get_template(&ticket.ticket_type) {
            Some(t) => t,
            None => return StepStatus::Todo,
        };

        let step_name = if ticket.step.is_empty() {
            match template.first_step() {
                Some(s) => s.name.clone(),
                None => return StepStatus::Todo,
            }
        } else {
            ticket.step.clone()
        };

        let step = match template.get_step(&step_name) {
            Some(s) => s,
            None => return StepStatus::Todo,
        };

        let is_first = template.first_step().is_some_and(|s| s.name == step_name);
        let is_last = step.next_step.is_none();

        step.derived_status(is_first, is_last)
    }

    /// Get prompt for current step with variable substitution
    pub fn get_step_prompt(&self, ticket: &Ticket, pr_config: Option<&PrConfig>) -> Result<String> {
        let step = self
            .current_step(ticket)
            .context("No current step found for ticket")?;

        self.render_prompt(&step.prompt, ticket, pr_config)
    }

    /// Build the Handlebars interpolation context for a ticket.
    ///
    /// This is the single source of truth for the variable surface available
    /// to step prompts (`id`, `ticket_type`, `project`, `summary`, `priority`,
    /// `step`, optional PR vars, and `steps.<name>.*` step outputs). Other
    /// renderers (e.g. workflow export) reuse this so prompts interpolate
    /// identically to how they would at launch time.
    pub fn build_ticket_context(
        ticket: &Ticket,
        pr_config: Option<&PrConfig>,
    ) -> serde_json::Value {
        let mut data = serde_json::Map::new();
        data.insert("id".to_string(), serde_json::json!(ticket.id));
        data.insert(
            "ticket_type".to_string(),
            serde_json::json!(ticket.ticket_type),
        );
        data.insert("project".to_string(), serde_json::json!(ticket.project));
        data.insert("summary".to_string(), serde_json::json!(ticket.summary));
        data.insert("priority".to_string(), serde_json::json!(ticket.priority));
        data.insert("step".to_string(), serde_json::json!(ticket.step));

        // Add PR config data if available
        if let Some(config) = pr_config {
            data.insert(
                "branch_name".to_string(),
                serde_json::json!(config.generate_branch_name(ticket)),
            );
            data.insert(
                "pr_title".to_string(),
                serde_json::json!(config.generate_title(ticket)),
            );
            data.insert(
                "base_branch".to_string(),
                serde_json::json!(config.base_branch),
            );
        }

        // Load step output artifacts into {{ steps.{name}.* }} context
        let steps_data = Self::load_step_outputs(ticket);
        if !steps_data.is_empty() {
            data.insert("steps".to_string(), serde_json::Value::Object(steps_data));
        }

        serde_json::Value::Object(data)
    }

    /// Render a prompt template with ticket data
    fn render_prompt(
        &self,
        template: &str,
        ticket: &Ticket,
        pr_config: Option<&PrConfig>,
    ) -> Result<String> {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(false);

        let data = Self::build_ticket_context(ticket, pr_config);

        hbs.render_template(template, &data)
            .context("Failed to render step prompt")
    }

    /// Read a single sub-agent's step output file at
    /// `{worktree}/.tickets/steps/{step_name}/{key}.json`.
    ///
    /// Returns `Value::Null` if the file is missing or can't be parsed;
    /// callers should treat that as "sub-agent did not produce output".
    pub fn read_agent_step_output(
        ticket: &Ticket,
        step_name: &str,
        key: &str,
    ) -> serde_json::Value {
        let Some(worktree) = ticket.worktree_path.as_deref() else {
            return serde_json::Value::Null;
        };
        let path = std::path::PathBuf::from(worktree)
            .join(".tickets")
            .join("steps")
            .join(step_name)
            .join(format!("{key}.json"));
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or(serde_json::Value::Null),
            Err(_) => serde_json::Value::Null,
        }
    }

    /// Write a per-sub-agent output file at
    /// `{worktree}/.tickets/steps/{step_name}/{key}.json`.
    pub fn write_agent_step_output(
        ticket: &Ticket,
        step_name: &str,
        key: &str,
        output: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let worktree = ticket
            .worktree_path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("ticket {} has no worktree_path", ticket.id))?;
        let dir = std::path::PathBuf::from(worktree)
            .join(".tickets")
            .join("steps")
            .join(step_name);
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("create_dir_all {}", dir.display()))?;
        let path = dir.join(format!("{key}.json"));
        let contents = serde_json::to_string_pretty(output)?;
        std::fs::write(&path, contents).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }

    /// Write the aggregated step output artifact at
    /// `{worktree}/.tickets/steps/{step_name}.output.json`.
    /// This is the file `load_step_outputs` reads into `{{ steps.{name}.* }}`.
    pub fn write_step_output_artifact(
        ticket: &Ticket,
        step_name: &str,
        output: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let worktree = ticket
            .worktree_path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("ticket {} has no worktree_path", ticket.id))?;
        let steps_dir = std::path::PathBuf::from(worktree)
            .join(".tickets")
            .join("steps");
        std::fs::create_dir_all(&steps_dir)
            .with_context(|| format!("create_dir_all {}", steps_dir.display()))?;
        let path = steps_dir.join(format!("{step_name}.output.json"));
        let contents = serde_json::to_string_pretty(output)?;
        std::fs::write(&path, contents).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }

    /// Load step output artifact JSON files from `.tickets/steps/` in the ticket's worktree
    fn load_step_outputs(ticket: &Ticket) -> serde_json::Map<String, serde_json::Value> {
        let mut steps_map = serde_json::Map::new();

        let base_path = match &ticket.worktree_path {
            Some(p) => std::path::PathBuf::from(p),
            None => return steps_map,
        };

        let steps_dir = base_path.join(".tickets").join("steps");
        let entries = match std::fs::read_dir(&steps_dir) {
            Ok(e) => e,
            Err(_) => return steps_map,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            // Extract step name from "{step_name}.output.json"
            let filename = match path.file_stem().and_then(|s| s.to_str()) {
                Some(f) => f.to_string(),
                None => continue,
            };
            let step_name = filename.strip_suffix(".output").unwrap_or(&filename);

            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                    steps_map.insert(step_name.to_string(), value);
                }
            }
        }

        steps_map
    }

    /// Get allowed tools for current step
    pub fn get_allowed_tools(&self, ticket: &Ticket) -> Vec<String> {
        self.current_step(ticket)
            .map(|s| s.allowed_tools)
            .unwrap_or_default()
    }

    /// Get the step to go to on rejection
    pub fn get_rejection_step(&self, ticket: &Ticket) -> Option<(String, String)> {
        let current = self.current_step(ticket)?;
        let on_reject = current.on_reject?;
        Some((on_reject.goto_step, on_reject.prompt))
    }

    /// Render the rejection prompt with the rejection reason substituted
    pub fn render_rejection_prompt(
        &self,
        ticket: &Ticket,
        rejection_reason: &str,
    ) -> Option<String> {
        let current = self.current_step(ticket)?;
        let on_reject = current.on_reject?;

        // Replace the rejection_reason placeholder
        let rendered = on_reject
            .prompt
            .replace("{{ rejection_reason }}", rejection_reason);

        Some(rendered)
    }

    /// Check if current step outputs a plan or review (requires rejection feedback)
    pub fn requires_rejection_feedback(&self, ticket: &Ticket) -> bool {
        self.current_step(ticket)
            .is_some_and(|s| s.outputs_plan() || s.outputs_review())
    }

    /// Get step progress as (`current_index`, `total_steps`, `step_names`)
    pub fn get_progress(&self, ticket: &Ticket) -> (usize, usize, Vec<String>) {
        let steps = self.all_steps(ticket);
        let total = steps.len();
        let step_names: Vec<String> = steps.iter().map(|s| s.name.clone()).collect();

        let current_idx = if ticket.step.is_empty() {
            0
        } else {
            step_names
                .iter()
                .position(|n| n == &ticket.step)
                .unwrap_or(0)
        };

        (current_idx, total, step_names)
    }

    /// Format step progress for display
    /// Returns something like: "[plan] > implement > test > pr"
    pub fn format_progress(&self, ticket: &Ticket) -> String {
        let (current_idx, _, step_names) = self.get_progress(ticket);

        step_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                if i == current_idx {
                    format!("[{name}]")
                } else {
                    name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" > ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_ticket(ticket_type: &str, step: &str) -> Ticket {
        Ticket {
            filename: "20241221-1430-FEAT-gamesvc-test.md".to_string(),
            filepath: "/test/path".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: ticket_type.to_string(),
            project: "gamesvc".to_string(),
            id: "FEAT-1234".to_string(),
            summary: "Test feature".to_string(),
            priority: "P2-medium".to_string(),
            status: "queued".to_string(),
            step: step.to_string(),
            content: "Test content".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
        }
    }

    #[test]
    fn test_current_step() {
        let config = Config::default();
        let manager = StepManager::new(&config);

        let ticket = make_test_ticket("FEAT", "plan");
        let step = manager.current_step(&ticket);
        assert!(step.is_some());
        assert_eq!(step.unwrap().name, "plan");
    }

    #[test]
    fn test_next_step() {
        let config = Config::default();
        let manager = StepManager::new(&config);

        let ticket = make_test_ticket("FEAT", "plan");
        let next = manager.next_step(&ticket);
        assert!(next.is_some());
        assert_eq!(next.unwrap().name, "build");
    }

    #[test]
    fn test_is_pr_step() {
        let config = Config::default();
        let manager = StepManager::new(&config);

        // deploy step now outputs pr
        let ticket = make_test_ticket("FEAT", "deploy");
        assert!(manager.is_pr_step(&ticket)); // deploy outputs pr

        let ticket = make_test_ticket("FEAT", "plan");
        assert!(!manager.is_pr_step(&ticket)); // plan step doesn't output pr
    }

    #[test]
    fn test_format_progress() {
        let config = Config::default();
        let manager = StepManager::new(&config);

        let ticket = make_test_ticket("FEAT", "code");
        let progress = manager.format_progress(&ticket);
        assert!(progress.contains("[code]"));
        assert!(progress.contains("plan >"));
    }

    #[test]
    fn test_load_step_outputs_no_worktree() {
        let ticket = make_test_ticket("FEAT", "plan");
        let outputs = StepManager::load_step_outputs(&ticket);
        assert!(outputs.is_empty());
    }

    #[test]
    fn test_load_step_outputs_with_artifacts() {
        let tmp = tempfile::tempdir().unwrap();
        let steps_dir = tmp.path().join(".tickets").join("steps");
        std::fs::create_dir_all(&steps_dir).unwrap();

        // Write a classifier output artifact
        std::fs::write(
            steps_dir.join("classify.output.json"),
            r#"{"output_type": "enum", "value": "high"}"#,
        )
        .unwrap();

        // Write a multi-model output artifact
        std::fs::write(
            steps_dir.join("consensus.output.json"),
            r#"{"winner_response": "Use approach A"}"#,
        )
        .unwrap();

        let mut ticket = make_test_ticket("FEAT", "plan");
        ticket.worktree_path = Some(tmp.path().to_string_lossy().to_string());

        let outputs = StepManager::load_step_outputs(&ticket);
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs["classify"]["value"], "high");
        assert_eq!(outputs["consensus"]["winner_response"], "Use approach A");
    }

    #[test]
    fn test_load_step_outputs_ignores_non_json() {
        let tmp = tempfile::tempdir().unwrap();
        let steps_dir = tmp.path().join(".tickets").join("steps");
        std::fs::create_dir_all(&steps_dir).unwrap();

        std::fs::write(steps_dir.join("notes.txt"), "not json").unwrap();
        std::fs::write(steps_dir.join("valid.output.json"), r#"{"value": true}"#).unwrap();

        let mut ticket = make_test_ticket("FEAT", "plan");
        ticket.worktree_path = Some(tmp.path().to_string_lossy().to_string());

        let outputs = StepManager::load_step_outputs(&ticket);
        assert_eq!(outputs.len(), 1);
        assert!(outputs.contains_key("valid"));
    }

    #[test]
    fn test_render_prompt_with_step_outputs() {
        let tmp = tempfile::tempdir().unwrap();
        let steps_dir = tmp.path().join(".tickets").join("steps");
        std::fs::create_dir_all(&steps_dir).unwrap();

        std::fs::write(
            steps_dir.join("classify.output.json"),
            r#"{"value": "critical"}"#,
        )
        .unwrap();

        let config = Config::default();
        let manager = StepManager::new(&config);

        let mut ticket = make_test_ticket("FEAT", "plan");
        ticket.worktree_path = Some(tmp.path().to_string_lossy().to_string());

        let result = manager
            .render_prompt("Severity is {{ steps.classify.value }}", &ticket, None)
            .unwrap();

        assert_eq!(result, "Severity is critical");
    }

    #[test]
    fn test_render_prompt_missing_step_output_is_empty() {
        let config = Config::default();
        let manager = StepManager::new(&config);

        let ticket = make_test_ticket("FEAT", "plan");
        let result = manager
            .render_prompt("Severity is {{ steps.classify.value }}", &ticket, None)
            .unwrap();

        // strict_mode is false, so missing values render as empty string
        assert_eq!(result, "Severity is ");
    }

    // ─── Per-sub-agent artifact tests ─────────────────────────────────

    #[test]
    fn test_read_agent_step_output_missing_returns_null() {
        let tmp = tempfile::tempdir().unwrap();
        let mut ticket = make_test_ticket("FEAT", "plan");
        ticket.worktree_path = Some(tmp.path().to_string_lossy().to_string());

        let out = StepManager::read_agent_step_output(&ticket, "review", "agent-A");
        assert_eq!(out, serde_json::Value::Null);
    }

    #[test]
    fn test_read_agent_step_output_no_worktree_returns_null() {
        let ticket = make_test_ticket("FEAT", "plan");
        let out = StepManager::read_agent_step_output(&ticket, "review", "agent-A");
        assert_eq!(out, serde_json::Value::Null);
    }

    #[test]
    fn test_write_and_read_agent_step_output_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let mut ticket = make_test_ticket("FEAT", "plan");
        ticket.worktree_path = Some(tmp.path().to_string_lossy().to_string());

        let payload = serde_json::json!({
            "summary": "looks good",
            "score": 87,
        });

        StepManager::write_agent_step_output(&ticket, "review", "agent-A", &payload).unwrap();

        // File exists at expected path
        let expected = tmp
            .path()
            .join(".tickets")
            .join("steps")
            .join("review")
            .join("agent-A.json");
        assert!(
            expected.exists(),
            "output file should exist at {expected:?}"
        );

        let round = StepManager::read_agent_step_output(&ticket, "review", "agent-A");
        assert_eq!(round, payload);
    }

    #[test]
    fn test_write_step_output_artifact_creates_file_read_by_load_step_outputs() {
        let tmp = tempfile::tempdir().unwrap();
        let mut ticket = make_test_ticket("FEAT", "plan");
        ticket.worktree_path = Some(tmp.path().to_string_lossy().to_string());

        let aggregated = serde_json::json!({
            "type": "multi_model",
            "winner_response": "Use approach A",
        });

        StepManager::write_step_output_artifact(&ticket, "consensus", &aggregated).unwrap();

        // load_step_outputs picks it up under the step name
        let outputs = StepManager::load_step_outputs(&ticket);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs["consensus"]["winner_response"], "Use approach A");
    }

    #[test]
    fn test_write_agent_step_output_without_worktree_errors() {
        let ticket = make_test_ticket("FEAT", "plan");
        let err = StepManager::write_agent_step_output(
            &ticket,
            "review",
            "agent-A",
            &serde_json::json!({}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("worktree_path"));
    }
}
