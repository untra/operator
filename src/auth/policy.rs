use super::identity::UserAuthority;
use crate::rest::error::ApiError;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Boundary {
    Instance,
    Project,
    Transport,
}

pub struct Operation {
    pub method: &'static str,
    pub path: &'static str,
    pub capability: &'static str,
    pub boundary: Boundary,
}

pub const OPERATIONS: &[Operation] = &[
    Operation {
        method: "GET",
        path: "/api/v1/license",
        capability: "license:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/license",
        capability: "license:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "DELETE",
        path: "/api/v1/license",
        capability: "license:delete",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/targets",
        capability: "targets:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/targets",
        capability: "targets:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/targets/{name}",
        capability: "targets:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "DELETE",
        path: "/api/v1/targets/{name}",
        capability: "targets:delete",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/targets/{name}/probe",
        capability: "targets:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/campaigns",
        capability: "campaigns:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/campaigns",
        capability: "campaigns:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/campaigns/{id}",
        capability: "campaigns:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/tickets/{id}/campaign",
        capability: "tickets:update",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/campaigns/{id}/launch",
        capability: "campaigns:launch",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/campaigns/{id}/pause",
        capability: "campaigns:pause",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/campaigns/{id}/status",
        capability: "campaigns:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/profiles",
        capability: "profiles:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/profiles",
        capability: "profiles:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/profiles/{profile_id}",
        capability: "profiles:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PATCH",
        path: "/api/v1/profiles/{profile_id}",
        capability: "profiles:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/health",
        capability: "health:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/status",
        capability: "status:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/sections",
        capability: "sections:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/connections",
        capability: "connections:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/connections/mcp/clients",
        capability: "connections:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/integrations",
        capability: "integrations:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/setup/status",
        capability: "setup:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/setup/steps",
        capability: "setup:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/setup/collections",
        capability: "setup:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/setup/initialize",
        capability: "setup:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/issuetypes",
        capability: "issuetypes:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/issuetypes",
        capability: "issuetypes:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/issuetypes/{key}",
        capability: "issuetypes:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/issuetypes/{key}",
        capability: "issuetypes:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "DELETE",
        path: "/api/v1/issuetypes/{key}",
        capability: "issuetypes:delete",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/issuetypes/{key}/document",
        capability: "issuetypes:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/issuetypes/{key}/steps",
        capability: "issuetypes:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/issuetypes/{key}/steps/{step_name}",
        capability: "issuetypes:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/issuetypes/{key}/steps/{step_name}",
        capability: "issuetypes:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/issuetypes/{key}/workflow-preview",
        capability: "issuetypes:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/collections",
        capability: "collections:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/collections/active",
        capability: "collections:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/collections/{name}",
        capability: "collections:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/collections/{name}/activate",
        capability: "collections:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/queue/kanban",
        capability: "queue:read",
        boundary: Boundary::Project,
    },
    Operation {
        method: "GET",
        path: "/api/v1/queue/status",
        capability: "queue:read",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/queue/pause",
        capability: "queue:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/queue/resume",
        capability: "queue:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/queue/sync",
        capability: "queue:sync",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/queue/sync/{provider}/{project_key}",
        capability: "queue:execute",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/agents/active",
        capability: "agents:read",
        boundary: Boundary::Project,
    },
    Operation {
        method: "GET",
        path: "/api/v1/agents/{agent_id}",
        capability: "agents:read",
        boundary: Boundary::Project,
    },
    Operation {
        method: "GET",
        path: "/api/v1/agents/{agent_id}/connect/terminal",
        capability: "agents:read",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/agents/{agent_id}/approve",
        capability: "agents:approve",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/agents/{agent_id}/reject",
        capability: "agents:reject",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/agents/{agent_id}/focus",
        capability: "agents:focus",
        boundary: Boundary::Project,
    },
    Operation {
        method: "GET",
        path: "/api/v1/projects",
        capability: "projects:read",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/projects/{name}/assess",
        capability: "projects:create",
        boundary: Boundary::Project,
    },
    Operation {
        method: "GET",
        path: "/api/v1/tickets/{id}",
        capability: "tickets:read",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/tickets",
        capability: "tickets:create",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/tickets/preview",
        capability: "tickets:read",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/tickets/batch/preview",
        capability: "tickets:read",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/tickets/batch",
        capability: "tickets:create",
        boundary: Boundary::Project,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/tickets/{id}/status",
        capability: "tickets:update",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/alerts",
        capability: "alerts:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/tickets/{id}/launch",
        capability: "tickets:launch",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/tickets/{id}/steps/{step}/complete",
        capability: "tickets:complete",
        boundary: Boundary::Project,
    },
    Operation {
        method: "POST",
        path: "/api/v1/tickets/{id}/workflow-export",
        capability: "tickets:read",
        boundary: Boundary::Project,
    },
    Operation {
        method: "GET",
        path: "/api/v1/workflow-formats",
        capability: "workflow-formats:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/kanban/providers",
        capability: "kanban:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/kanban/{provider}/{project_key}/issuetypes",
        capability: "kanban:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/kanban/{provider}/{project_key}/statuses",
        capability: "kanban:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/kanban/{provider}/{project_key}/issuetypes/sync",
        capability: "kanban:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/kanban/validate",
        capability: "kanban:execute",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/kanban/projects",
        capability: "kanban:execute",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/kanban/statuses",
        capability: "kanban:execute",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/kanban/config",
        capability: "kanban:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/kanban/session-env",
        capability: "kanban:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/git/providers",
        capability: "git:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/git/validate",
        capability: "git:execute",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/git/config",
        capability: "git:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/git/session-env",
        capability: "git:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/skills",
        capability: "skills:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/llm-tools",
        capability: "llm-tools:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/llm-tools/default",
        capability: "llm-tools:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/llm-tools/default",
        capability: "llm-tools:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/delegators",
        capability: "delegators:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/delegators",
        capability: "delegators:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/delegators/from-tool",
        capability: "delegators:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/delegators/import-profile",
        capability: "delegators:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/delegators/{name}/profile",
        capability: "delegators:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/delegators/{name}",
        capability: "delegators:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/delegators/{name}",
        capability: "delegators:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "DELETE",
        path: "/api/v1/delegators/{name}",
        capability: "delegators:delete",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/model-servers",
        capability: "model-servers:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "POST",
        path: "/api/v1/model-servers",
        capability: "model-servers:create",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/model-servers/kinds",
        capability: "model-servers:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/model-servers/kinds/{slug}/models",
        capability: "model-servers:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/model-servers/{name}",
        capability: "model-servers:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PUT",
        path: "/api/v1/model-servers/{name}",
        capability: "model-servers:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "DELETE",
        path: "/api/v1/model-servers/{name}",
        capability: "model-servers:delete",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/model-servers/{name}/models",
        capability: "model-servers:execute",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/configuration",
        capability: "configuration:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "PATCH",
        path: "/api/v1/configuration",
        capability: "configuration:update",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/execution-targets",
        capability: "execution-targets:read",
        boundary: Boundary::Instance,
    },
    Operation {
        method: "GET",
        path: "/api/v1/mcp/descriptor",
        capability: "mcp:read",
        boundary: Boundary::Transport,
    },
    Operation {
        method: "GET",
        path: "/api/v1/mcp/sse",
        capability: "mcp:connect",
        boundary: Boundary::Transport,
    },
    Operation {
        method: "POST",
        path: "/api/v1/mcp/message",
        capability: "mcp:connect",
        boundary: Boundary::Transport,
    },
    Operation {
        method: "GET",
        path: "/api/v1/telemetry/sessions/{session_id}",
        capability: "telemetry:read",
        boundary: Boundary::Instance,
    },
];

pub fn capabilities() -> Vec<String> {
    OPERATIONS
        .iter()
        .map(|o| o.capability.to_string())
        .chain(
            crate::mcp::tools::all_tool_definitions()
                .iter()
                .map(|t| format!("mcp:{}", t.name)),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[derive(serde::Serialize, utoipa::ToSchema, ts_rs::TS)]
#[ts(export)]
pub struct CapabilityDescription {
    pub capability: String,
    pub resource: String,
    pub label: String,
    pub description: String,
}

pub fn capability_descriptions() -> Vec<CapabilityDescription> {
    let tools = crate::mcp::tools::all_tool_definitions();
    capabilities().into_iter().map(|capability| {
        let (resource, action) = capability.split_once(':').unwrap_or((&capability, ""));
        let label = format!("{} {}", action.replace('_', " "), resource.replace('-', " "));
        let description = tools.iter().find(|tool| capability == format!("mcp:{}", tool.name))
            .map_or_else(|| {
                let boundary = OPERATIONS.iter().find(|operation| operation.capability == capability).map(|operation| operation.boundary);
                match boundary {
                    Some(Boundary::Project) => "Applies only to selected projects.",
                    Some(Boundary::Instance) => "Requires permission for selected instance operations.",
                    _ => "Connects the MCP transport; tool permissions are granted separately.",
                }.to_string()
            }, |tool| tool.description.clone());
        CapabilityDescription { resource: resource.to_string(), label, description, capability }
    }).collect()
}

pub fn browser_identity_route(path: &str) -> bool {
    path.starts_with("/api/v1/auth/keys")
        || path.starts_with("/api/v1/auth/users")
        || path.starts_with("/api/v1/auth/oidc/connections")
}

pub fn authorize(authority: &UserAuthority, method: &str, path: &str) -> Result<(), ApiError> {
    let denied = || ApiError::Forbidden("This credential does not permit the operation".into());
    if path.starts_with("/api/v1/auth/") {
        if authority.key_id.is_some() {
            return Err(denied());
        }
        if matches!(
            path,
            "/api/v1/auth/session"
                | "/api/v1/auth/csrf"
                | "/api/v1/auth/logout"
                | "/api/v1/auth/capabilities"
                | "/api/v1/auth/keys"
                | "/api/v1/auth/keys/{id}"
                | "/api/v1/auth/keys/{id}/rotate"
        ) {
            return Ok(());
        }
        if path == "/api/v1/auth/device/approve" {
            return Err(denied());
        }
        return if authority.user.administrator {
            Ok(())
        } else {
            Err(denied())
        };
    }
    if authority.key_id.is_none() && authority.user.administrator {
        return Ok(());
    }
    if path == "/api/v1/profiles" || path.starts_with("/api/v1/profiles/") {
        if method != "GET" {
            return Err(denied());
        }
        if authority.key_id.is_none() {
            return Ok(());
        }
    }
    let operation = OPERATIONS
        .iter()
        .find(|o| o.method == method && o.path == path)
        .ok_or_else(denied)?;
    if !authority
        .grants
        .capabilities
        .iter()
        .any(|c| c == operation.capability)
        || (operation.boundary == Boundary::Instance && !authority.grants.instance_wide)
    {
        return Err(denied());
    }
    Ok(())
}

pub fn project_allowed(authority: Option<&UserAuthority>, project: &str) -> bool {
    authority.is_none_or(|a| {
        (a.key_id.is_none() && a.user.administrator)
            || a.grants.projects.iter().any(|p| p == project)
    })
}

pub fn tool_allowed(authority: Option<&UserAuthority>, name: &str) -> bool {
    authority.is_none_or(|a| {
        if a.key_id.is_none() && a.user.administrator {
            return true;
        }
        let project_tool = matches!(
            name,
            "operator_list_tickets"
                | "operator_create_ticket"
                | "operator_claim_ticket"
                | "operator_complete_ticket"
                | "operator_return_to_queue"
                | "operator_launch_ticket"
                | "operator_approve_agent"
                | "operator_reject_agent"
                | "operator_focus_agent"
                | "operator_preview_ticket_batch"
                | "operator_create_tickets_bulk"
        );
        a.grants.capabilities.contains(&format!("mcp:{name}"))
            && (project_tool || a.grants.instance_wide)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::scope::{Access, ROUTE_RULES};

    #[test]
    fn test_every_application_route_has_an_explicit_policy() {
        for rule in ROUTE_RULES {
            if !matches!(rule.access, Access::Scoped(_)) || rule.path.starts_with("/api/v1/auth/") {
                continue;
            }
            assert_eq!(
                OPERATIONS
                    .iter()
                    .filter(|o| o.method == rule.method && o.path == rule.path)
                    .count(),
                1,
                "{} {}",
                rule.method,
                rule.path
            );
        }
    }
}
