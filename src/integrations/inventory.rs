//! Capability inventory for surface parity testing.
//!
//! Defines every operator capability and which surfaces expose it:
//! slash commands (Zed), MCP tools, REST routes, and TUI keybindings.

/// A single operator capability and the surfaces where it appears.
#[derive(Debug, Clone)]
pub struct Capability {
    /// Human-readable capability name
    pub name: &'static str,
    /// Zed slash command key (e.g. "op-pause"), or None if not exposed
    pub slash_command: Option<&'static str>,
    /// MCP tool name (e.g. "`operator_pause_queue`"), or None if not exposed
    pub mcp_tool: Option<&'static str>,
    /// REST endpoint in "METHOD /path" form (axum-style params), or None
    pub rest_endpoint: Option<&'static str>,
    /// TUI keybinding description substring to match, or None
    pub tui_action: Option<&'static str>,
}

/// Returns the full list of operator capabilities with their surface mappings.
///
/// Each entry declares which surfaces expose that capability. A `None` value
/// means the capability is intentionally absent from that surface.
pub fn all_capabilities() -> Vec<Capability> {
    vec![
        Capability {
            name: "Status",
            slash_command: Some("op-status"),
            mcp_tool: Some("operator_status"),
            rest_endpoint: Some("GET /api/v1/status"),
            tui_action: None, // status panel is a view, not a keybinding action
        },
        Capability {
            name: "List Queue",
            slash_command: Some("op-queue"),
            mcp_tool: Some("operator_list_tickets"),
            rest_endpoint: Some("GET /api/v1/queue/kanban"),
            tui_action: Some("Focus Queue panel"),
        },
        Capability {
            name: "Launch Ticket",
            slash_command: Some("op-launch"),
            mcp_tool: Some("operator_launch_ticket"),
            rest_endpoint: Some("POST /api/v1/tickets/:id/launch"),
            tui_action: Some("Launch selected ticket"),
        },
        Capability {
            name: "Active Agents",
            slash_command: Some("op-active"),
            mcp_tool: None,
            rest_endpoint: Some("GET /api/v1/agents/active"),
            tui_action: Some("Focus Agents panel"),
        },
        Capability {
            name: "Completed Tickets",
            slash_command: Some("op-completed"),
            mcp_tool: Some("operator_list_tickets"),
            rest_endpoint: None,
            tui_action: None,
        },
        Capability {
            name: "Ticket Details",
            slash_command: Some("op-ticket"),
            mcp_tool: None,
            rest_endpoint: Some("GET /api/v1/tickets/:id"),
            tui_action: None,
        },
        Capability {
            name: "Pause Queue",
            slash_command: Some("op-pause"),
            mcp_tool: Some("operator_pause_queue"),
            rest_endpoint: Some("POST /api/v1/queue/pause"),
            tui_action: Some("Pause queue"),
        },
        Capability {
            name: "Resume Queue",
            slash_command: Some("op-resume"),
            mcp_tool: Some("operator_resume_queue"),
            rest_endpoint: Some("POST /api/v1/queue/resume"),
            tui_action: Some("Resume queue"),
        },
        Capability {
            name: "Sync Kanban",
            slash_command: Some("op-sync"),
            mcp_tool: Some("operator_sync_kanban"),
            rest_endpoint: Some("POST /api/v1/queue/sync"),
            tui_action: Some("Sync kanban"),
        },
        Capability {
            name: "Approve Review",
            slash_command: Some("op-approve"),
            mcp_tool: Some("operator_approve_agent"),
            rest_endpoint: Some("POST /api/v1/agents/:agent_id/approve"),
            tui_action: Some("Approve review"),
        },
        Capability {
            name: "Reject Review",
            slash_command: Some("op-reject"),
            mcp_tool: Some("operator_reject_agent"),
            rest_endpoint: Some("POST /api/v1/agents/:agent_id/reject"),
            tui_action: Some("Reject review"),
        },
        Capability {
            name: "Setup Agent",
            slash_command: Some("op-setup-agent"),
            mcp_tool: None,
            rest_endpoint: None,
            tui_action: None, // Zed-only capability
        },
        Capability {
            name: "Setup",
            slash_command: Some("op-setup"),
            mcp_tool: None,
            rest_endpoint: None,
            tui_action: None, // Zed-only diagnostic command
        },
        Capability {
            name: "Help",
            slash_command: Some("op-help"),
            mcp_tool: None,
            rest_endpoint: None,
            tui_action: None, // Zed-only help listing
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_capabilities_non_empty() {
        let caps = all_capabilities();
        assert!(!caps.is_empty(), "Capability list should not be empty");
    }

    #[test]
    fn test_all_capabilities_have_names() {
        for cap in all_capabilities() {
            assert!(!cap.name.is_empty(), "Every capability must have a name");
        }
    }

    #[test]
    fn test_all_capabilities_have_at_least_one_surface() {
        for cap in all_capabilities() {
            let has_any = cap.slash_command.is_some()
                || cap.mcp_tool.is_some()
                || cap.rest_endpoint.is_some()
                || cap.tui_action.is_some();
            assert!(
                has_any,
                "Capability '{}' must appear on at least one surface",
                cap.name
            );
        }
    }

    #[test]
    fn test_capability_count() {
        let caps = all_capabilities();
        assert_eq!(caps.len(), 14, "Expected 14 capabilities in the inventory");
    }
}
