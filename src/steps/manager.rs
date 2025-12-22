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

        if !current.requires_review {
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
            .map(|s| s.outputs.contains(&StepOutput::Pr))
            .unwrap_or(false)
    }

    /// Check if current step requires human review
    pub fn requires_review(&self, ticket: &Ticket) -> bool {
        self.current_step(ticket)
            .map(|s| s.requires_review)
            .unwrap_or(false)
    }

    /// Check if current step is the final step
    pub fn is_final_step(&self, ticket: &Ticket) -> bool {
        self.current_step(ticket)
            .map(|s| s.next_step.is_none())
            .unwrap_or(true)
    }

    /// Get the status category for the current step based on step properties
    pub fn current_status(&self, ticket: &Ticket) -> StepStatus {
        let template = match self.get_template(&ticket.ticket_type) {
            Some(t) => t,
            None => return StepStatus::TODO,
        };

        let step_name = if ticket.step.is_empty() {
            match template.first_step() {
                Some(s) => s.name.clone(),
                None => return StepStatus::TODO,
            }
        } else {
            ticket.step.clone()
        };

        let step = match template.get_step(&step_name) {
            Some(s) => s,
            None => return StepStatus::TODO,
        };

        let is_first = template
            .first_step()
            .map(|s| s.name == step_name)
            .unwrap_or(false);
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

    /// Render a prompt template with ticket data
    fn render_prompt(
        &self,
        template: &str,
        ticket: &Ticket,
        pr_config: Option<&PrConfig>,
    ) -> Result<String> {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(false);

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

        hbs.render_template(template, &serde_json::Value::Object(data))
            .context("Failed to render step prompt")
    }

    /// Get allowed tools for current step
    pub fn get_allowed_tools(&self, ticket: &Ticket) -> Vec<String> {
        self.current_step(ticket)
            .map(|s| s.allowed_tools.clone())
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
            .map(|s| s.outputs_plan() || s.outputs_review())
            .unwrap_or(false)
    }

    /// Get step progress as (current_index, total_steps, step_names)
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
                    format!("[{}]", name)
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

        // deploy step outputs review, not pr
        let ticket = make_test_ticket("FEAT", "deploy");
        assert!(!manager.is_pr_step(&ticket)); // deploy outputs review, not pr

        let ticket = make_test_ticket("FEAT", "plan");
        assert!(!manager.is_pr_step(&ticket));
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
}
