use std::collections::HashMap;
use std::time::{Duration, Instant};

use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::agents::connect::{self, ConnectTerminal};
use crate::auth::scope::Principal;
use crate::licensing::{require_premium, PremiumFeature};
use crate::rest::dto::auth::PrincipalKind;
use crate::rest::error::ApiError;
use crate::rest::middleware::auth::Authenticated;
use crate::rest::state::{ApiState, RequestState as State};
use crate::state::State as OperatorState;

const LIFETIME: Duration = Duration::from_mins(2);
const MAX_PENDING: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct AttachDestination {
    pub path: String,
    pub authority: Option<String>,
}

#[derive(Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct AttachReference {
    pub reference: Uuid,
    pub destination: AttachDestination,
}

#[derive(PartialEq)]
struct Binding {
    subject: String,
    public_origin: Option<String>,
    profile: Uuid,
    project: String,
    agent: String,
    destination: AttachDestination,
    terminal: ConnectTerminal,
}

#[derive(Default)]
pub(crate) struct AttachRegistry(HashMap<Uuid, (Instant, Binding)>);

impl AttachRegistry {
    fn issue(&mut self, binding: Binding, now: Instant) -> Result<AttachReference, ApiError> {
        self.0.retain(|_, (expires, _)| *expires > now);
        if self.0.len() >= MAX_PENDING {
            return Err(ApiError::RateLimited {
                retry_after_secs: LIFETIME.as_secs(),
            });
        }
        let reference = Uuid::new_v4();
        let destination = binding.destination.clone();
        self.0.insert(reference, (now + LIFETIME, binding));
        Ok(AttachReference {
            reference,
            destination,
        })
    }

    fn redeem(&mut self, reference: Uuid, current: &Binding, now: Instant) -> Result<(), ApiError> {
        if self
            .0
            .get(&reference)
            .is_some_and(|(expires, old)| now < *expires && old == current)
        {
            self.0.remove(&reference);
            Ok(())
        } else {
            Err(ApiError::Forbidden(
                "Invalid or expired terminal attachment".into(),
            ))
        }
    }
}

fn binding(state: &ApiState, principal: &Principal, agent_id: &str) -> Result<Binding, ApiError> {
    let config = state.config();
    let local = principal.kind == PrincipalKind::LocalProcess;
    if local != (state.auth.deployment_mode == crate::config::DeploymentMode::Local) {
        return Err(ApiError::Forbidden(
            "Credential does not match terminal deployment mode".into(),
        ));
    }
    if !local {
        require_premium(&config, PremiumFeature::RemoteTargets)?;
        if !matches!(
            principal.kind,
            PrincipalKind::AccessToken | PrincipalKind::OidcToken | PrincipalKind::Session
        ) || principal
            .authority
            .as_ref()
            .is_none_or(|a| a.key_id.is_some())
        {
            return Err(ApiError::Forbidden(
                "Sign in with an authorized Operator account".into(),
            ));
        }
    }
    let snapshot =
        OperatorState::load(&config).map_err(|e| ApiError::InternalError(e.to_string()))?;
    let agent = snapshot
        .agents
        .iter()
        .find(|a| a.id == agent_id && state.may_access_project(&a.project))
        .ok_or_else(|| ApiError::NotFound("Agent not found".into()))?;
    let remote = connect::resolve_remote(&config, agent);
    if !local && remote.is_none() {
        return Err(ApiError::Conflict(
            "No remote workspace is configured for this agent".into(),
        ));
    }
    if agent.remote_host.is_some() && local {
        return Err(ApiError::Forbidden(
            "Remote attach requires an authorized Operator account".into(),
        ));
    }
    let probes = connect::ConnectProbes::default();
    let mut terminal = connect::connect_terminal(&super::agents::connect_context(
        &config,
        agent,
        remote.as_ref(),
        &probes,
    ));
    terminal.command = None;
    if terminal.action.is_none() {
        return Err(ApiError::Conflict(
            "Agent has no supported terminal action".into(),
        ));
    }
    let path = terminal.cwd.clone().unwrap_or_else(|| {
        config
            .projects_path()
            .join(&agent.project)
            .to_string_lossy()
            .into_owned()
    });
    terminal.cwd = Some(path.clone());
    Ok(Binding {
        subject: principal.subject.clone(),
        public_origin: config.rest_api.public_base_url(),
        profile: config.profile.id,
        project: agent.project.clone(),
        agent: agent_id.to_owned(),
        destination: AttachDestination {
            path,
            authority: terminal.authority.clone(),
        },
        terminal,
    })
}

#[utoipa::path(post, path="/api/v1/agents/{agent_id}/connect/attach", tag="Agents", params(("agent_id"=String,Path)), responses((status=200,body=AttachReference)))]
pub async fn issue(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(agent): Path<String>,
) -> Result<Json<AttachReference>, ApiError> {
    let current = binding(&state, &principal, &agent)?;
    Ok(Json(
        state
            .attachments
            .lock()
            .await
            .issue(current, Instant::now())?,
    ))
}

#[utoipa::path(post, path="/api/v1/agents/{agent_id}/connect/redeem", tag="Agents", params(("agent_id"=String,Path)), request_body=AttachReference, responses((status=200,body=ConnectTerminal)))]
pub async fn redeem(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(agent): Path<String>,
    Json(request): Json<AttachReference>,
) -> Result<Json<ConnectTerminal>, ApiError> {
    let current = binding(&state, &principal, &agent)?;
    if current.destination != request.destination {
        return Err(ApiError::Forbidden("Terminal workspace has changed".into()));
    }
    state
        .attachments
        .lock()
        .await
        .redeem(request.reference, &current, Instant::now())?;
    Ok(Json(current.terminal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::connect::{SessionPhase, TerminalAction};

    fn bound() -> Binding {
        Binding {
            subject: "user".into(),
            public_origin: Some("https://operator.example".into()),
            profile: Uuid::nil(),
            project: "p".into(),
            agent: "a".into(),
            destination: AttachDestination {
                authority: Some("ssh-remote+fixture".into()),
                path: "/work".into(),
            },
            terminal: ConnectTerminal {
                phase: SessionPhase::Live,
                terminal_name: "ticket".into(),
                ticket_id: "ticket".into(),
                cwd: Some("/work".into()),
                command: None,
                authority: Some("ssh-remote+fixture".into()),
                action: Some(TerminalAction::Tmux {
                    session: "ticket".into(),
                    socket: None,
                }),
            },
        }
    }

    #[test]
    fn test_attach_reference_checks_binding_expiry_and_single_use() {
        let mut registry = AttachRegistry::default();
        let now = Instant::now();
        let reference = registry.issue(bound(), now).unwrap().reference;
        let mut other = bound();
        other.subject = "another".into();
        assert!(registry.redeem(reference, &other, now).is_err());
        let mut moved = bound();
        moved.destination.path = "/elsewhere".into();
        assert!(registry.redeem(reference, &moved, now).is_err());
        assert!(registry.redeem(reference, &bound(), now).is_ok());
        assert!(registry.redeem(reference, &bound(), now).is_err());
        let expired = registry.issue(bound(), now).unwrap().reference;
        assert!(registry.redeem(expired, &bound(), now + LIFETIME).is_err());
    }
}
