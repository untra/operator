//! Zed Extension for Operator
//!
//! Provides three integration layers:
//! 1. MCP context server — registers `operator mcp` so Zed's agent panel
//!    has native access to all operator tools and ticket resources.
//! 2. ACP agent setup — `/op-setup-agent` generates the config snippet
//!    for Zed's `agent_servers` settings.
//! 3. Slash commands — thin AI/human inference layer for quick operations
//!    with tab completion.

use serde::Deserialize;
use std::process::Command as StdCommand;
use zed_extension_api::{
    self as zed, Command, ContextServerId, ContextServerConfiguration, Project, SlashCommand,
    SlashCommandArgumentCompletion, SlashCommandOutput, SlashCommandOutputSection, Worktree,
};

const DEFAULT_API_URL: &str = "http://localhost:7008";

const KNOWN_BINARY_LOCATIONS: &[&str] = &[
    "/usr/local/bin/operator",
    "/opt/homebrew/bin/operator",
];

struct OperatorExtension {
    api_url: String,
    cached_binary_path: Option<String>,
}

impl OperatorExtension {
    fn find_operator_binary(&mut self, worktree: Option<&Worktree>) -> Option<String> {
        if let Some(ref path) = self.cached_binary_path {
            return Some(path.clone());
        }

        if let Some(wt) = worktree {
            if let Some(path) = wt.which("operator") {
                self.cached_binary_path = Some(path.clone());
                return Some(path);
            }
        }

        for location in KNOWN_BINARY_LOCATIONS {
            if std::fs::metadata(location).is_ok() {
                self.cached_binary_path = Some((*location).to_string());
                return Some((*location).to_string());
            }
        }

        // Try home directory cargo bin
        if let Ok(home) = std::env::var("HOME") {
            let cargo_bin = format!("{home}/.cargo/bin/operator");
            if std::fs::metadata(&cargo_bin).is_ok() {
                self.cached_binary_path = Some(cargo_bin.clone());
                return Some(cargo_bin);
            }
        }

        None
    }

    fn curl_get(&self, endpoint: &str) -> Result<String, String> {
        let url = format!("{}{}", self.api_url, endpoint);
        let output = StdCommand::new("curl")
            .args(["-s", "-f", &url])
            .output()
            .map_err(|e| format!("Failed to execute curl: {}", e))?;

        if output.status.success() {
            String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 response: {}", e))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("API request failed: {}", stderr))
        }
    }

    fn curl_post(&self, endpoint: &str, body: Option<&str>) -> Result<String, String> {
        let url = format!("{}{}", self.api_url, endpoint);
        let mut cmd = StdCommand::new("curl");
        cmd.args(["-s", "-f", "-X", "POST"]);

        if let Some(json_body) = body {
            cmd.args(["-H", "Content-Type: application/json", "-d", json_body]);
        }

        cmd.arg(&url);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute curl: {}", e))?;

        if output.status.success() {
            String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 response: {}", e))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("API request failed: {}", stderr))
        }
    }

    fn handle_status(&self) -> SlashCommandOutput {
        match self.curl_get("/api/v1/health") {
            Ok(json) => {
                if let Ok(health) = serde_json::from_str::<HealthResponse>(&json) {
                    let text = format!(
                        "## Operator Status\n\n\
                        **Status**: {}\n\
                        **Version**: {}\n\
                        **Uptime**: {} seconds\n\
                        **Queue Processing**: {}\n\n\
                        | Metric | Count |\n\
                        |--------|-------|\n\
                        | Queue | {} |\n\
                        | Active Agents | {} |\n\
                        | Completed Today | {} |",
                        health.status,
                        health.version,
                        health.uptime_seconds,
                        if health.queue_paused {
                            "paused"
                        } else {
                            "running"
                        },
                        health.queue_count,
                        health.active_agents,
                        health.completed_today
                    );
                    make_output(&text, "Operator Status")
                } else {
                    make_output(&format!("```json\n{}\n```", json), "Operator Status (raw)")
                }
            }
            Err(e) => make_error(&format!(
                "Failed to get Operator status.\n\n\
                **Error**: {}\n\n\
                Make sure Operator is running: `operator api`",
                e
            )),
        }
    }

    fn handle_queue(&self) -> SlashCommandOutput {
        match self.curl_get("/api/v1/tickets/queue") {
            Ok(json) => {
                if let Ok(response) = serde_json::from_str::<TicketsResponse>(&json) {
                    if response.tickets.is_empty() {
                        make_output("## Queue\n\n*No tickets in queue*", "Queue")
                    } else {
                        let mut text =
                            "## Queue\n\n| ID | Project | Type | Title |\n|---|---|---|---|\n"
                                .to_string();
                        for ticket in &response.tickets {
                            text.push_str(&format!(
                                "| {} | {} | {} | {} |\n",
                                ticket.id,
                                ticket.project.as_deref().unwrap_or("-"),
                                ticket.issue_type.as_deref().unwrap_or("-"),
                                ticket.title.as_deref().unwrap_or("-")
                            ));
                        }
                        text.push_str(&format!(
                            "\n*{} ticket(s) in queue*",
                            response.tickets.len()
                        ));
                        make_output(&text, "Queue")
                    }
                } else {
                    make_output(&format!("```json\n{}\n```", json), "Queue (raw)")
                }
            }
            Err(e) => make_error(&format!("Failed to fetch queue: {}", e)),
        }
    }

    fn handle_launch(&self, ticket_id: &str) -> SlashCommandOutput {
        let body = r#"{"provider":null,"wrapper":"terminal","model":"sonnet","yolo_mode":false,"retry_reason":null,"resume_session_id":null}"#;
        match self.curl_post(&format!("/api/v1/tickets/{}/launch", ticket_id), Some(body)) {
            Ok(json) => {
                if let Ok(response) = serde_json::from_str::<LaunchResponse>(&json) {
                    let worktree_msg = if response.worktree_created {
                        " (worktree created)"
                    } else {
                        ""
                    };
                    let text = format!(
                        "## Launched: {}{}\n\n\
                        **Working Directory**: `{}`\n\
                        **Terminal**: {}\n\n\
                        Run this command in your terminal:\n\
                        ```bash\n{}\n```",
                        response.ticket_id,
                        worktree_msg,
                        response.working_directory,
                        response.terminal_name,
                        response.command
                    );
                    make_output(&text, &format!("Launched {}", response.ticket_id))
                } else {
                    make_output(&format!("```json\n{}\n```", json), "Launch Response")
                }
            }
            Err(e) => make_error(&format!("Failed to launch ticket {}: {}", ticket_id, e)),
        }
    }

    fn handle_active(&self) -> SlashCommandOutput {
        match self.curl_get("/api/v1/agents/active") {
            Ok(json) => {
                if let Ok(response) = serde_json::from_str::<AgentsResponse>(&json) {
                    if response.agents.is_empty() {
                        make_output("## Active Agents\n\n*No active agents*", "Active Agents")
                    } else {
                        let mut text =
                            "## Active Agents\n\n| ID | Ticket | Project | Status |\n|---|---|---|---|\n"
                                .to_string();
                        for agent in &response.agents {
                            text.push_str(&format!(
                                "| {} | {} | {} | {} |\n",
                                &agent.id[..8.min(agent.id.len())],
                                agent.ticket_id,
                                agent.project,
                                agent.status
                            ));
                        }
                        text.push_str(&format!("\n*{} active agent(s)*", response.agents.len()));
                        make_output(&text, "Active Agents")
                    }
                } else {
                    make_output(&format!("```json\n{}\n```", json), "Active Agents (raw)")
                }
            }
            Err(e) => make_error(&format!("Failed to fetch active agents: {}", e)),
        }
    }

    fn handle_completed(&self) -> SlashCommandOutput {
        match self.curl_get("/api/v1/tickets/completed") {
            Ok(json) => {
                if let Ok(response) = serde_json::from_str::<TicketsResponse>(&json) {
                    if response.tickets.is_empty() {
                        make_output(
                            "## Completed Tickets\n\n*No recently completed tickets*",
                            "Completed",
                        )
                    } else {
                        let mut text =
                            "## Completed Tickets\n\n| ID | Project | Type | Title |\n|---|---|---|---|\n"
                                .to_string();
                        for ticket in response.tickets.iter().take(10) {
                            text.push_str(&format!(
                                "| {} | {} | {} | {} |\n",
                                ticket.id,
                                ticket.project.as_deref().unwrap_or("-"),
                                ticket.issue_type.as_deref().unwrap_or("-"),
                                ticket.title.as_deref().unwrap_or("-")
                            ));
                        }
                        text.push_str(&format!(
                            "\n*Showing {} of {} completed ticket(s)*",
                            10.min(response.tickets.len()),
                            response.tickets.len()
                        ));
                        make_output(&text, "Completed")
                    }
                } else {
                    make_output(&format!("```json\n{}\n```", json), "Completed (raw)")
                }
            }
            Err(e) => make_error(&format!("Failed to fetch completed tickets: {}", e)),
        }
    }

    fn handle_ticket(&self, ticket_id: &str) -> SlashCommandOutput {
        match self.curl_get(&format!("/api/v1/tickets/{}", ticket_id)) {
            Ok(json) => {
                if let Ok(ticket) = serde_json::from_str::<TicketDetail>(&json) {
                    let text = format!(
                        "## Ticket: {}\n\n\
                        **Title**: {}\n\
                        **Type**: {}\n\
                        **Project**: {}\n\
                        **Status**: {}\n\
                        **Priority**: {}\n\n\
                        ### Description\n\n{}",
                        ticket.id,
                        ticket.title.as_deref().unwrap_or("-"),
                        ticket.issue_type.as_deref().unwrap_or("-"),
                        ticket.project.as_deref().unwrap_or("-"),
                        ticket.status.as_deref().unwrap_or("-"),
                        ticket.priority.unwrap_or(0),
                        ticket.description.as_deref().unwrap_or("*No description*")
                    );
                    make_output(&text, &format!("Ticket {}", ticket.id))
                } else {
                    make_output(&format!("```json\n{}\n```", json), "Ticket (raw)")
                }
            }
            Err(e) => make_error(&format!("Failed to fetch ticket {}: {}", ticket_id, e)),
        }
    }

    fn handle_pause(&self) -> SlashCommandOutput {
        match self.curl_post("/api/v1/queue/pause", None) {
            Ok(json) => {
                if let Ok(response) = serde_json::from_str::<MessageResponse>(&json) {
                    make_output(
                        &format!("## Queue Paused\n\n{}", response.message),
                        "Queue Paused",
                    )
                } else {
                    make_output(
                        "## Queue Paused\n\nQueue processing has been paused.",
                        "Queue Paused",
                    )
                }
            }
            Err(e) => make_error(&format!("Failed to pause queue: {}", e)),
        }
    }

    fn handle_resume(&self) -> SlashCommandOutput {
        match self.curl_post("/api/v1/queue/resume", None) {
            Ok(json) => {
                if let Ok(response) = serde_json::from_str::<MessageResponse>(&json) {
                    make_output(
                        &format!("## Queue Resumed\n\n{}", response.message),
                        "Queue Resumed",
                    )
                } else {
                    make_output(
                        "## Queue Resumed\n\nQueue processing has been resumed.",
                        "Queue Resumed",
                    )
                }
            }
            Err(e) => make_error(&format!("Failed to resume queue: {}", e)),
        }
    }

    fn handle_sync(&self) -> SlashCommandOutput {
        match self.curl_post("/api/v1/kanban/sync", None) {
            Ok(json) => {
                if let Ok(response) = serde_json::from_str::<SyncResponse>(&json) {
                    let text = format!(
                        "## Kanban Sync Complete\n\n\
                        **Created**: {}\n\
                        **Skipped**: {}\n\
                        **Errors**: {}",
                        response.created.len(),
                        response.skipped.len(),
                        response.errors.len()
                    );
                    make_output(&text, "Kanban Sync")
                } else {
                    make_output(&format!("```json\n{}\n```", json), "Kanban Sync (raw)")
                }
            }
            Err(e) => make_error(&format!("Failed to sync kanban: {}", e)),
        }
    }

    fn handle_approve(&self, agent_id: &str) -> SlashCommandOutput {
        match self.curl_post(&format!("/api/v1/agents/{}/approve", agent_id), None) {
            Ok(json) => {
                if let Ok(response) = serde_json::from_str::<MessageResponse>(&json) {
                    make_output(
                        &format!("## Review Approved\n\n{}", response.message),
                        "Review Approved",
                    )
                } else {
                    make_output(
                        &format!("## Review Approved\n\nAgent {} approved.", agent_id),
                        "Review Approved",
                    )
                }
            }
            Err(e) => make_error(&format!("Failed to approve agent {}: {}", agent_id, e)),
        }
    }

    fn handle_reject(&self, args: &str) -> SlashCommandOutput {
        let parts: Vec<&str> = args.splitn(2, ' ').collect();
        if parts.len() < 2 {
            return make_error(
                "Usage: /op-reject AGENT-ID REASON\n\nPlease provide both agent ID and rejection reason.",
            );
        }

        let agent_id = parts[0];
        let reason = parts[1];

        let body = format!(r#"{{"reason":"{}"}}"#, reason.replace('"', "\\\""));
        match self.curl_post(&format!("/api/v1/agents/{}/reject", agent_id), Some(&body)) {
            Ok(json) => {
                if let Ok(response) = serde_json::from_str::<MessageResponse>(&json) {
                    make_output(
                        &format!("## Review Rejected\n\n{}", response.message),
                        "Review Rejected",
                    )
                } else {
                    make_output(
                        &format!(
                            "## Review Rejected\n\nAgent {} rejected.\n\n**Reason**: {}",
                            agent_id, reason
                        ),
                        "Review Rejected",
                    )
                }
            }
            Err(e) => make_error(&format!("Failed to reject agent {}: {}", agent_id, e)),
        }
    }

    fn handle_setup_agent(&self, worktree: Option<&Worktree>) -> SlashCommandOutput {
        let binary_path = match find_operator_binary_oneshot(worktree) {
            Some(path) => path,
            None => {
                return make_error(
                    "Could not find `operator` binary.\n\n\
                    Install operator first, then re-run this command.\n\n\
                    ```bash\n\
                    cargo install --path /path/to/operator\n\
                    ```",
                );
            }
        };

        let snippet = format!(
            r#"{{
  "agent_servers": {{
    "operator": {{
      "command": "{}",
      "args": ["acp"],
      "env": {{}}
    }}
  }}
}}"#,
            binary_path.replace('\\', "\\\\").replace('"', "\\\"")
        );

        let text = format!(
            "## ACP Agent Server Setup\n\n\
            Add the following to your Zed settings (`~/.config/zed/settings.json`):\n\n\
            ```json\n{}\n```\n\n\
            **Binary**: `{}`\n\n\
            After adding this config, restart Zed. Operator will appear as an agent \
            in the Agent Panel. You can send prompts to it and it will delegate to \
            Claude Code via the ACP protocol.\n\n\
            This is a one-time setup step.",
            snippet, binary_path
        );

        make_output(&text, "ACP Agent Setup")
    }

    fn get_queue_ticket_ids(&self) -> Vec<String> {
        if let Ok(json) = self.curl_get("/api/v1/tickets/queue") {
            if let Ok(response) = serde_json::from_str::<TicketsResponse>(&json) {
                return response.tickets.into_iter().map(|t| t.id).collect();
            }
        }
        Vec::new()
    }

    fn get_awaiting_agent_ids(&self) -> Vec<(String, String)> {
        if let Ok(json) = self.curl_get("/api/v1/agents/active") {
            if let Ok(response) = serde_json::from_str::<AgentsResponse>(&json) {
                return response
                    .agents
                    .into_iter()
                    .filter(|a| a.status == "awaiting_input")
                    .map(|a| (a.id, a.ticket_id))
                    .collect();
            }
        }
        Vec::new()
    }
}

impl zed::Extension for OperatorExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        OperatorExtension {
            api_url: DEFAULT_API_URL.to_string(),
            cached_binary_path: None,
        }
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Command, String> {
        // Try known locations first; fall back to bare "operator" so Zed's
        // process spawn does its own PATH resolution.
        let binary_path = self
            .find_operator_binary(None)
            .unwrap_or_else(|| "operator".to_string());

        Ok(Command {
            command: binary_path,
            args: vec!["mcp".to_string()],
            env: vec![],
        })
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>, String> {
        Ok(Some(ContextServerConfiguration {
            installation_instructions: "\
## Install Operator

Operator provides multi-agent orchestration for Claude Code.

### From source (Rust)
```bash
cd /path/to/operator
cargo install --path .
```

### Verify installation
```bash
operator --version
operator mcp  # should start MCP server on stdio
```

After installing, reload the Zed extension to pick up the binary."
                .to_string(),
            settings_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "api_url": {
                        "type": "string",
                        "default": "http://localhost:7008",
                        "description": "Operator REST API URL (used by slash commands)"
                    }
                }
            })
            .to_string(),
            default_settings: serde_json::json!({
                "api_url": "http://localhost:7008"
            })
            .to_string(),
        }))
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        let arg = args.join(" ");

        match command.name.as_str() {
            "op-status" => Ok(self.handle_status()),
            "op-queue" => Ok(self.handle_queue()),
            "op-launch" => {
                if arg.is_empty() {
                    Ok(make_error("Usage: /op-launch TICKET-ID"))
                } else {
                    Ok(self.handle_launch(&arg))
                }
            }
            "op-active" => Ok(self.handle_active()),
            "op-completed" => Ok(self.handle_completed()),
            "op-ticket" => {
                if arg.is_empty() {
                    Ok(make_error("Usage: /op-ticket TICKET-ID"))
                } else {
                    Ok(self.handle_ticket(&arg))
                }
            }
            "op-pause" => Ok(self.handle_pause()),
            "op-resume" => Ok(self.handle_resume()),
            "op-sync" => Ok(self.handle_sync()),
            "op-approve" => {
                if arg.is_empty() {
                    Ok(make_error("Usage: /op-approve AGENT-ID"))
                } else {
                    Ok(self.handle_approve(&arg))
                }
            }
            "op-reject" => {
                if arg.is_empty() {
                    Ok(make_error("Usage: /op-reject AGENT-ID REASON"))
                } else {
                    Ok(self.handle_reject(&arg))
                }
            }
            "op-setup-agent" => Ok(self.handle_setup_agent(worktree)),
            _ => Err(format!("Unknown command: {}", command.name)),
        }
    }

    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "op-launch" | "op-ticket" => {
                let ticket_ids = self.get_queue_ticket_ids();
                Ok(ticket_ids
                    .into_iter()
                    .map(|id| SlashCommandArgumentCompletion {
                        label: id.clone(),
                        new_text: id,
                        run_command: true,
                    })
                    .collect())
            }
            "op-approve" => {
                let agents = self.get_awaiting_agent_ids();
                Ok(agents
                    .into_iter()
                    .map(|(id, ticket_id)| SlashCommandArgumentCompletion {
                        label: format!("{} ({})", &id[..8.min(id.len())], ticket_id),
                        new_text: id,
                        run_command: true,
                    })
                    .collect())
            }
            "op-reject" => {
                let agents = self.get_awaiting_agent_ids();
                Ok(agents
                    .into_iter()
                    .map(|(id, ticket_id)| SlashCommandArgumentCompletion {
                        label: format!("{} ({})", &id[..8.min(id.len())], ticket_id),
                        new_text: format!("{} ", id),
                        run_command: false,
                    })
                    .collect())
            }
            _ => Ok(Vec::new()),
        }
    }
}

fn find_operator_binary_oneshot(worktree: Option<&Worktree>) -> Option<String> {
    if let Some(wt) = worktree {
        if let Some(path) = wt.which("operator") {
            return Some(path);
        }
    }

    for location in KNOWN_BINARY_LOCATIONS {
        if std::fs::metadata(location).is_ok() {
            return Some((*location).to_string());
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let cargo_bin = format!("{home}/.cargo/bin/operator");
        if std::fs::metadata(&cargo_bin).is_ok() {
            return Some(cargo_bin);
        }
    }

    None
}

fn make_output(text: &str, label: &str) -> SlashCommandOutput {
    SlashCommandOutput {
        text: text.to_string(),
        sections: vec![SlashCommandOutputSection {
            range: (0..text.len()).into(),
            label: label.to_string(),
        }],
    }
}

fn make_error(message: &str) -> SlashCommandOutput {
    let text = format!("## Error\n\n{}", message);
    SlashCommandOutput {
        text: text.clone(),
        sections: vec![SlashCommandOutputSection {
            range: (0..text.len()).into(),
            label: "Error".to_string(),
        }],
    }
}

#[derive(Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    queue_paused: bool,
    queue_count: usize,
    active_agents: usize,
    completed_today: usize,
}

#[derive(Deserialize)]
struct TicketsResponse {
    tickets: Vec<TicketSummary>,
}

#[derive(Deserialize)]
struct TicketSummary {
    id: String,
    title: Option<String>,
    project: Option<String>,
    issue_type: Option<String>,
}

#[derive(Deserialize)]
struct TicketDetail {
    id: String,
    title: Option<String>,
    project: Option<String>,
    issue_type: Option<String>,
    status: Option<String>,
    priority: Option<i32>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct AgentsResponse {
    agents: Vec<AgentSummary>,
}

#[derive(Deserialize)]
struct AgentSummary {
    id: String,
    ticket_id: String,
    project: String,
    status: String,
}

#[derive(Deserialize)]
struct LaunchResponse {
    ticket_id: String,
    terminal_name: String,
    working_directory: String,
    command: String,
    worktree_created: bool,
}

#[derive(Deserialize)]
struct MessageResponse {
    message: String,
}

#[derive(Deserialize)]
struct SyncResponse {
    created: Vec<String>,
    skipped: Vec<String>,
    errors: Vec<String>,
}

zed::register_extension!(OperatorExtension);
