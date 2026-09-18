use std::time::Duration;

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::agents::delegator_resolution::resolve_named_target;
use crate::config::{Config, TargetDef, TargetKind};
use crate::licensing::{self, PremiumFeature};
use crate::rest::{error::ApiError, state::ApiState};

const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
const SSH_CONNECT_TIMEOUT: &str = "ConnectTimeout=10";
const MAX_TARGET_NAME_LENGTH: usize = 64;

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct TargetResponse {
    #[serde(flatten)]
    #[schema(value_type = Object)]
    pub target: TargetDef,
    pub premium: bool,
    pub entitled: bool,
    pub user_declared: bool,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct TargetsResponse {
    pub targets: Vec<TargetResponse>,
    pub total: usize,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct TargetProbeResponse {
    pub reachable: bool,
    pub message: String,
}

fn response(config: &Config, target: TargetDef) -> TargetResponse {
    projection(config, licensing::entitlements(config), target)
}

/// Built from entitlements resolved once by the caller, so listing N targets
/// does not verify the licence N times.
fn projection(
    config: &Config,
    entitlements: licensing::Entitlements,
    target: TargetDef,
) -> TargetResponse {
    let premium = matches!(target.kind, TargetKind::Ssh(_) | TargetKind::Coder(_));
    TargetResponse {
        entitled: !premium || entitlements.allows(PremiumFeature::RemoteTargets),
        user_declared: config.targets.iter().any(|entry| entry.name == target.name),
        target,
        premium,
    }
}

fn validate(target: &TargetDef) -> Result<(), ApiError> {
    if target.name.is_empty()
        || target.name.len() > MAX_TARGET_NAME_LENGTH
        || !target
            .name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    {
        return Err(ApiError::ValidationError(
            "Target names must contain 1-64 lowercase letters, digits, hyphens, or underscores"
                .into(),
        ));
    }
    if let TargetKind::Ssh(ssh) = &target.kind {
        if ssh.ssh_alias.trim().is_empty()
            || ssh.ssh_alias.starts_with('-')
            || ssh.ssh_alias.chars().any(char::is_whitespace)
        {
            return Err(ApiError::ValidationError(
                "SSH target requires a valid SSH alias".into(),
            ));
        }
        if !ssh.workdir.starts_with('/') {
            return Err(ApiError::ValidationError(
                "SSH target requires an absolute working directory".into(),
            ));
        }
    }
    Ok(())
}

fn ensure_unreferenced(config: &Config, name: &str) -> Result<(), ApiError> {
    if config.launch.target.as_deref() == Some(name)
        || config.delegators.iter().any(|delegator| {
            delegator.launch_config.as_ref().is_some_and(|launch| {
                launch.target.as_deref() == Some(name) || launch.host.as_deref() == Some(name)
            })
        })
    {
        return Err(ApiError::Conflict(format!(
            "Target '{name}' is referenced by launch configuration"
        )));
    }
    ensure_inactive(config, name)
}

fn ensure_inactive(config: &Config, name: &str) -> Result<(), ApiError> {
    let runtime = crate::state::State::load(config).map_err(ApiError::from)?;
    if runtime.agents.iter().any(|agent| {
        (agent.target_name.as_deref() == Some(name) || agent.remote_host.as_deref() == Some(name))
            && matches!(
                agent.status.as_str(),
                "running" | "awaiting_input" | "completing"
            )
    }) {
        return Err(ApiError::Conflict(format!(
            "Target '{name}' has active work"
        )));
    }
    Ok(())
}

#[utoipa::path(get, path = "/api/v1/targets",
    operation_id = "targets_list", tag = "Targets", responses((status = 200, body = TargetsResponse)))]
pub async fn list(State(state): State<ApiState>) -> Result<Json<TargetsResponse>, ApiError> {
    let config = state.config();
    let entitlements = licensing::entitlements(&config);
    let targets = crate::config::targets::launchable_target_names(&config)
        .iter()
        .map(|name| {
            resolve_named_target(&config, name)
                .map(|target| projection(&config, entitlements, target))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ApiError::ValidationError(error.to_string()))?;
    Ok(Json(TargetsResponse {
        total: targets.len(),
        targets,
    }))
}

#[utoipa::path(post, path = "/api/v1/targets",
    operation_id = "targets_create", tag = "Targets", request_body = serde_json::Value,
    responses((status = 200, body = TargetResponse), (status = 402, description = "Premium required")))]
pub async fn create(
    State(state): State<ApiState>,
    Json(target): Json<TargetDef>,
) -> Result<Json<TargetResponse>, ApiError> {
    validate(&target)?;
    let result = state
        .mutate_config(move |config| {
            licensing::require_target(config, &target)?;
            if crate::config::targets::known_target_name(config, &target.name) {
                return Err(ApiError::Conflict(format!(
                    "Target '{}' already exists",
                    target.name
                )));
            }
            config.targets.push(target.clone());
            crate::config::targets::validate_targets(config)
                .map_err(|error| ApiError::ValidationError(error.to_string()))?;
            Ok(response(config, target))
        })
        .await?;
    Ok(Json(result))
}

#[utoipa::path(put, path = "/api/v1/targets/{name}",
    operation_id = "targets_update", tag = "Targets", request_body = serde_json::Value,
    params(("name" = String, Path)), responses((status = 200, body = TargetResponse), (status = 402, description = "Premium required")))]
pub async fn update(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Json(target): Json<TargetDef>,
) -> Result<Json<TargetResponse>, ApiError> {
    validate(&target)?;
    if target.name != name {
        return Err(ApiError::ValidationError(
            "Target renaming is not supported; create a new target instead".into(),
        ));
    }
    let result = state
        .mutate_config(move |config| {
            licensing::require_target(config, &target)?;
            let index = config
                .targets
                .iter()
                .position(|entry| entry.name == name)
                .ok_or_else(|| ApiError::NotFound(format!("Declared target '{name}' not found")))?;
            licensing::require_target(config, &config.targets[index])?;
            ensure_inactive(config, &name)?;
            config.targets[index] = target.clone();
            crate::config::targets::validate_targets(config)
                .map_err(|error| ApiError::ValidationError(error.to_string()))?;
            Ok(response(config, target))
        })
        .await?;
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/api/v1/targets/{name}",
    operation_id = "targets_remove", tag = "Targets", params(("name" = String, Path)),
    responses((status = 200, body = TargetResponse), (status = 409, description = "Target is referenced")))]
pub async fn remove(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<TargetResponse>, ApiError> {
    let result = state
        .mutate_config(move |config| {
            let index = config
                .targets
                .iter()
                .position(|entry| entry.name == name)
                .ok_or_else(|| ApiError::NotFound(format!("Declared target '{name}' not found")))?;
            ensure_unreferenced(config, &name)?;
            let target = config.targets.remove(index);
            Ok(response(config, target))
        })
        .await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/targets/{name}/probe",
    operation_id = "targets_probe", tag = "Targets", params(("name" = String, Path)),
    responses((status = 200, body = TargetProbeResponse), (status = 402, description = "Premium required")))]
pub async fn probe(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<TargetProbeResponse>, ApiError> {
    let config = state.config();
    let target = resolve_named_target(&config, &name)
        .map_err(|error| ApiError::NotFound(error.to_string()))?;
    licensing::require_target(&config, &target)?;
    let reachable = match &target.kind {
        TargetKind::Ssh(ssh) => {
            validate(&target)?;
            let mut command = tokio::process::Command::new("ssh");
            command.kill_on_drop(true);
            command.args(["-o", "BatchMode=yes", "-o", SSH_CONNECT_TIMEOUT]);
            if let Some(path) = &ssh.ssh_config_path {
                command.args(["-F", path]);
            }
            command.args(["--", &ssh.ssh_alias, "true"]);
            command
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            tokio::time::timeout(PROBE_TIMEOUT, command.status())
                .await
                .is_ok_and(|result| result.is_ok_and(|status| status.success()))
        }
        TargetKind::Coder(coder) => {
            licensing::require_premium(&config, PremiumFeature::RemoteTargets)?;
            let session = crate::agents::launcher::coder::resolve_session(coder)
                .map_err(|error| ApiError::ValidationError(error.to_string()))?;
            let client = reqwest::Client::builder()
                .timeout(PROBE_TIMEOUT)
                .build()
                .map_err(|error| ApiError::InternalError(error.to_string()))?;
            client
                .get(format!(
                    "{}/api/v2/users/me",
                    session.url.trim_end_matches('/')
                ))
                .header("Coder-Session-Token", session.token)
                .send()
                .await
                .is_ok_and(|response| response.status().is_success())
        }
        TargetKind::Local => true,
        TargetKind::Docker(_) => {
            let mut command = tokio::process::Command::new("docker");
            command
                .kill_on_drop(true)
                .arg("info")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            tokio::time::timeout(PROBE_TIMEOUT, command.status())
                .await
                .is_ok_and(|result| result.is_ok_and(|status| status.success()))
        }
    };
    Ok(Json(TargetProbeResponse {
        reachable,
        message: if reachable {
            "Connection succeeded"
        } else {
            "Connection failed; check the target configuration and credentials"
        }
        .into(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_alias_cannot_inject_options() {
        let target = TargetDef {
            name: "test".into(),
            display_name: None,
            kind: TargetKind::Ssh(crate::config::SshTarget {
                ssh_alias: "-oProxyCommand=bad".into(),
                workdir: "/project".into(),
                ssh_config_path: None,
            }),
        };
        assert!(validate(&target).is_err());
    }
}
