//! LLM usage statistics per project.
//!
//! Tracks which LLM tools have been used on each project,
//! along with usage history, preferences, and success metrics.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// LLM usage statistics for a project
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct ProjectLlmStats {
    /// Project name
    pub project: String,

    /// Preferred LLM tool for this project (user override)
    #[serde(default)]
    pub preferred_tool: Option<String>,

    /// Preferred model for this project (user override)
    #[serde(default)]
    pub preferred_model: Option<String>,

    /// Usage history per LLM tool
    #[serde(default)]
    pub tool_usage: HashMap<String, LlmToolUsage>,

    /// Last updated timestamp
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

/// Usage statistics for a specific LLM tool
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct LlmToolUsage {
    /// Tool name (e.g., "claude", "gemini")
    pub tool: String,

    /// Total number of tickets processed
    #[serde(default)]
    pub ticket_count: u64,

    /// Number of successful completions
    #[serde(default)]
    pub success_count: u64,

    /// Number of failures/abandonments
    #[serde(default)]
    pub failure_count: u64,

    /// Total time spent (in seconds)
    #[serde(default)]
    pub total_time_secs: u64,

    /// Last used timestamp
    #[ts(type = "string")]
    pub last_used: DateTime<Utc>,

    /// Per-model breakdown
    #[serde(default)]
    pub model_usage: HashMap<String, LlmModelUsage>,
}

/// Usage statistics for a specific model
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct LlmModelUsage {
    /// Model name/alias
    pub model: String,

    /// Number of tickets
    #[serde(default)]
    pub ticket_count: u64,

    /// Success count
    #[serde(default)]
    pub success_count: u64,

    /// Failure count
    #[serde(default)]
    pub failure_count: u64,

    /// Total time (seconds)
    #[serde(default)]
    pub total_time_secs: u64,
}

impl ProjectLlmStats {
    /// Create new stats for a project
    pub fn new(project: &str) -> Self {
        Self {
            project: project.to_string(),
            preferred_tool: None,
            preferred_model: None,
            tool_usage: HashMap::new(),
            updated_at: Utc::now(),
        }
    }

    /// Record a ticket completion
    pub fn record_completion(
        &mut self,
        tool: &str,
        model: &str,
        success: bool,
        duration_secs: u64,
    ) {
        let tool_usage = self
            .tool_usage
            .entry(tool.to_string())
            .or_insert_with(|| LlmToolUsage {
                tool: tool.to_string(),
                last_used: Utc::now(),
                ..Default::default()
            });

        tool_usage.ticket_count += 1;
        tool_usage.total_time_secs += duration_secs;
        tool_usage.last_used = Utc::now();

        if success {
            tool_usage.success_count += 1;
        } else {
            tool_usage.failure_count += 1;
        }

        // Update model-level stats
        let model_usage = tool_usage
            .model_usage
            .entry(model.to_string())
            .or_insert_with(|| LlmModelUsage {
                model: model.to_string(),
                ..Default::default()
            });

        model_usage.ticket_count += 1;
        model_usage.total_time_secs += duration_secs;
        if success {
            model_usage.success_count += 1;
        } else {
            model_usage.failure_count += 1;
        }

        self.updated_at = Utc::now();
    }

    /// Get success rate for a tool (0.0 - 1.0)
    pub fn success_rate(&self, tool: &str) -> Option<f64> {
        self.tool_usage.get(tool).map(|u| {
            let total = u.success_count + u.failure_count;
            if total > 0 {
                u.success_count as f64 / total as f64
            } else {
                0.0
            }
        })
    }

    /// Get the most-used tool for this project
    pub fn most_used_tool(&self) -> Option<&str> {
        self.tool_usage
            .values()
            .max_by_key(|u| u.ticket_count)
            .map(|u| u.tool.as_str())
    }

    /// Get the most-used model for a given tool
    pub fn most_used_model(&self, tool: &str) -> Option<&str> {
        self.tool_usage.get(tool).and_then(|t| {
            t.model_usage
                .values()
                .max_by_key(|m| m.ticket_count)
                .map(|m| m.model.as_str())
        })
    }

    /// Get total ticket count across all tools
    pub fn total_tickets(&self) -> u64 {
        self.tool_usage.values().map(|u| u.ticket_count).sum()
    }

    /// Get overall success rate across all tools
    pub fn overall_success_rate(&self) -> f64 {
        let total_success: u64 = self.tool_usage.values().map(|u| u.success_count).sum();
        let total_failure: u64 = self.tool_usage.values().map(|u| u.failure_count).sum();
        let total = total_success + total_failure;
        if total > 0 {
            total_success as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get average time per ticket (in seconds)
    pub fn avg_time_per_ticket(&self) -> Option<u64> {
        let total_tickets = self.total_tickets();
        if total_tickets > 0 {
            let total_time: u64 = self.tool_usage.values().map(|u| u.total_time_secs).sum();
            Some(total_time / total_tickets)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_project_stats() {
        let stats = ProjectLlmStats::new("test-project");
        assert_eq!(stats.project, "test-project");
        assert!(stats.preferred_tool.is_none());
        assert!(stats.tool_usage.is_empty());
    }

    #[test]
    fn test_record_completion_updates_stats() {
        let mut stats = ProjectLlmStats::new("test-project");

        stats.record_completion("claude", "opus", true, 300);

        let tool_usage = stats.tool_usage.get("claude").unwrap();
        assert_eq!(tool_usage.ticket_count, 1);
        assert_eq!(tool_usage.success_count, 1);
        assert_eq!(tool_usage.failure_count, 0);
        assert_eq!(tool_usage.total_time_secs, 300);

        let model_usage = tool_usage.model_usage.get("opus").unwrap();
        assert_eq!(model_usage.ticket_count, 1);
        assert_eq!(model_usage.success_count, 1);
    }

    #[test]
    fn test_record_failure() {
        let mut stats = ProjectLlmStats::new("test-project");

        stats.record_completion("claude", "opus", false, 100);

        let tool_usage = stats.tool_usage.get("claude").unwrap();
        assert_eq!(tool_usage.success_count, 0);
        assert_eq!(tool_usage.failure_count, 1);
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut stats = ProjectLlmStats::new("test-project");

        stats.record_completion("claude", "opus", true, 100);
        stats.record_completion("claude", "opus", true, 100);
        stats.record_completion("claude", "opus", false, 100);

        let rate = stats.success_rate("claude").unwrap();
        assert!((rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_success_rate_empty() {
        let stats = ProjectLlmStats::new("test-project");
        assert!(stats.success_rate("claude").is_none());
    }

    #[test]
    fn test_most_used_tool() {
        let mut stats = ProjectLlmStats::new("test-project");

        stats.record_completion("claude", "opus", true, 100);
        stats.record_completion("claude", "opus", true, 100);
        stats.record_completion("gemini", "pro", true, 100);

        assert_eq!(stats.most_used_tool(), Some("claude"));
    }

    #[test]
    fn test_most_used_model() {
        let mut stats = ProjectLlmStats::new("test-project");

        stats.record_completion("claude", "opus", true, 100);
        stats.record_completion("claude", "sonnet", true, 100);
        stats.record_completion("claude", "sonnet", true, 100);

        assert_eq!(stats.most_used_model("claude"), Some("sonnet"));
    }

    #[test]
    fn test_total_tickets() {
        let mut stats = ProjectLlmStats::new("test-project");

        stats.record_completion("claude", "opus", true, 100);
        stats.record_completion("gemini", "pro", true, 100);
        stats.record_completion("codex", "gpt-4o", true, 100);

        assert_eq!(stats.total_tickets(), 3);
    }

    #[test]
    fn test_overall_success_rate() {
        let mut stats = ProjectLlmStats::new("test-project");

        stats.record_completion("claude", "opus", true, 100);
        stats.record_completion("gemini", "pro", false, 100);

        assert!((stats.overall_success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_avg_time_per_ticket() {
        let mut stats = ProjectLlmStats::new("test-project");

        stats.record_completion("claude", "opus", true, 300);
        stats.record_completion("claude", "opus", true, 600);

        assert_eq!(stats.avg_time_per_ticket(), Some(450));
    }

    #[test]
    fn test_multiple_models_per_tool() {
        let mut stats = ProjectLlmStats::new("test-project");

        stats.record_completion("claude", "opus", true, 100);
        stats.record_completion("claude", "sonnet", true, 100);
        stats.record_completion("claude", "haiku", true, 100);

        let tool_usage = stats.tool_usage.get("claude").unwrap();
        assert_eq!(tool_usage.model_usage.len(), 3);
        assert_eq!(tool_usage.ticket_count, 3);
    }

    #[test]
    fn test_export_bindings_projectllmstats() {
        let _ = ProjectLlmStats::export_to_string();
    }

    #[test]
    fn test_export_bindings_llmtoolusage() {
        let _ = LlmToolUsage::export_to_string();
    }

    #[test]
    fn test_export_bindings_llmmodelusage() {
        let _ = LlmModelUsage::export_to_string();
    }
}
