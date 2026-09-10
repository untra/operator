//! Deliberate public projection of operational configuration.

use axum::extract::State;
use axum::Json;

use crate::config::{Config, TargetKind, TARGET_DOCKER, TARGET_LOCAL};
use crate::rest::dto::{
    AgentsConfiguration, ConfigurationResponse, ExecutionTargetKind, ExecutionTargetSummary,
    ExecutionTargetsResponse, LaunchConfiguration, PanelNamesConfiguration, QueueConfiguration,
    UiConfiguration, UpdateConfigurationRequest,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

fn response(config: &Config) -> ConfigurationResponse {
    ConfigurationResponse {
        agents: AgentsConfiguration {
            max_parallel: config.agents.max_parallel,
            cores_reserved: config.agents.cores_reserved,
            max_agents_per_repo: config.agents.max_agents_per_repo,
            health_check_interval: config.agents.health_check_interval,
            generation_timeout_secs: config.agents.generation_timeout_secs,
            sync_interval: config.agents.sync_interval,
            step_timeout: config.agents.step_timeout,
            silence_threshold: config.agents.silence_threshold,
        },
        queue: QueueConfiguration {
            auto_assign: config.queue.auto_assign,
            priority_order: config.queue.priority_order.clone(),
            poll_interval_ms: config.queue.poll_interval_ms,
        },
        ui: UiConfiguration {
            refresh_rate_ms: config.ui.refresh_rate_ms,
            completed_history_hours: config.ui.completed_history_hours,
            summary_max_length: config.ui.summary_max_length,
            panel_names: PanelNamesConfiguration {
                status: config.ui.panel_names.status.clone(),
                queue: config.ui.panel_names.queue.clone(),
                in_progress: config.ui.panel_names.in_progress.clone(),
                completed: config.ui.panel_names.completed.clone(),
            },
        },
        launch: LaunchConfiguration {
            confirm_autonomous: config.launch.confirm_autonomous,
            confirm_paired: config.launch.confirm_paired,
            launch_delay_ms: config.launch.launch_delay_ms,
            docker_enabled: config.launch.docker.enabled,
            docker_image: config.launch.docker.image.clone(),
            yolo_enabled: config.launch.yolo.enabled,
            session_wrapper: config.sessions.wrapper.into(),
        },
    }
}

fn contains_null(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::Array(values) => values.iter().any(contains_null),
        serde_json::Value::Object(values) => values.values().any(contains_null),
        _ => false,
    }
}

fn apply_patch(config: &mut Config, patch: UpdateConfigurationRequest) {
    if let Some(agents) = patch.agents {
        if let Some(value) = agents.max_parallel {
            config.agents.max_parallel = value;
        }
        if let Some(value) = agents.cores_reserved {
            config.agents.cores_reserved = value;
        }
        if let Some(value) = agents.max_agents_per_repo {
            config.agents.max_agents_per_repo = value;
        }
        if let Some(value) = agents.health_check_interval {
            config.agents.health_check_interval = value;
        }
        if let Some(value) = agents.generation_timeout_secs {
            config.agents.generation_timeout_secs = value;
        }
        if let Some(value) = agents.sync_interval {
            config.agents.sync_interval = value;
        }
        if let Some(value) = agents.step_timeout {
            config.agents.step_timeout = value;
        }
        if let Some(value) = agents.silence_threshold {
            config.agents.silence_threshold = value;
        }
    }
    if let Some(queue) = patch.queue {
        if let Some(value) = queue.auto_assign {
            config.queue.auto_assign = value;
        }
        if let Some(value) = queue.priority_order {
            config.queue.priority_order = value;
        }
        if let Some(value) = queue.poll_interval_ms {
            config.queue.poll_interval_ms = value;
        }
    }
    if let Some(ui) = patch.ui {
        if let Some(value) = ui.refresh_rate_ms {
            config.ui.refresh_rate_ms = value;
        }
        if let Some(value) = ui.completed_history_hours {
            config.ui.completed_history_hours = value;
        }
        if let Some(value) = ui.summary_max_length {
            config.ui.summary_max_length = value;
        }
        if let Some(names) = ui.panel_names {
            if let Some(value) = names.status {
                config.ui.panel_names.status = value;
            }
            if let Some(value) = names.queue {
                config.ui.panel_names.queue = value;
            }
            if let Some(value) = names.in_progress {
                config.ui.panel_names.in_progress = value;
            }
            if let Some(value) = names.completed {
                config.ui.panel_names.completed = value;
            }
        }
    }
    if let Some(launch) = patch.launch {
        if let Some(value) = launch.confirm_autonomous {
            config.launch.confirm_autonomous = value;
        }
        if let Some(value) = launch.confirm_paired {
            config.launch.confirm_paired = value;
        }
        if let Some(value) = launch.launch_delay_ms {
            config.launch.launch_delay_ms = value;
        }
        if let Some(value) = launch.docker_enabled {
            config.launch.docker.enabled = value;
        }
        if let Some(value) = launch.docker_image {
            config.launch.docker.image = value;
        }
        if let Some(value) = launch.yolo_enabled {
            config.launch.yolo.enabled = value;
        }
        if let Some(value) = launch.session_wrapper {
            config.sessions.wrapper = value.into();
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/configuration",
    tag = "Configuration",
    operation_id = "configuration_get",
    responses((status = 200, description = "Supported operational configuration", body = ConfigurationResponse))
)]
pub async fn get_config(State(state): State<ApiState>) -> Json<ConfigurationResponse> {
    Json(response(&state.config()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/configuration",
    tag = "Configuration",
    operation_id = "configuration_patch",
    request_body = UpdateConfigurationRequest,
    responses(
        (status = 200, description = "Updated operational configuration", body = ConfigurationResponse),
        (status = 400, description = "Invalid, empty, or null-valued patch")
    )
)]
pub async fn patch_config(
    State(state): State<ApiState>,
    Json(raw): Json<serde_json::Value>,
) -> Result<Json<ConfigurationResponse>, ApiError> {
    if contains_null(&raw) {
        return Err(ApiError::ValidationError(
            "configuration patch values cannot be null".to_string(),
        ));
    }
    let patch: UpdateConfigurationRequest = serde_json::from_value(raw)
        .map_err(|error| ApiError::ValidationError(error.to_string()))?;
    if patch.is_empty() {
        return Err(ApiError::ValidationError(
            "configuration patch must change at least one section".to_string(),
        ));
    }

    let updated = state
        .mutate_config(move |config| {
            apply_patch(config, patch);
            Ok(response(config))
        })
        .await?;
    Ok(Json(updated))
}

#[utoipa::path(
    get,
    path = "/api/v1/execution-targets",
    tag = "Configuration",
    operation_id = "configuration_execution_targets",
    responses((status = 200, description = "Safe execution-target summaries", body = ExecutionTargetsResponse))
)]
pub async fn execution_targets(State(state): State<ApiState>) -> Json<ExecutionTargetsResponse> {
    let config = state.config();
    let mut targets = vec![
        ExecutionTargetSummary {
            name: TARGET_LOCAL.to_string(),
            display_name: Some("Local".to_string()),
            kind: ExecutionTargetKind::Local,
            available: true,
        },
        ExecutionTargetSummary {
            name: TARGET_DOCKER.to_string(),
            display_name: Some("Docker".to_string()),
            kind: ExecutionTargetKind::Docker,
            available: config.launch.docker.enabled,
        },
    ];
    targets.extend(config.targets.iter().map(|target| ExecutionTargetSummary {
        name: target.name.clone(),
        display_name: target.display_name.clone(),
        kind: match &target.kind {
            TargetKind::Local => ExecutionTargetKind::Local,
            TargetKind::Docker(_) => ExecutionTargetKind::Docker,
            TargetKind::Coder(_) => ExecutionTargetKind::Coder,
            TargetKind::Ssh(_) => ExecutionTargetKind::Ssh,
        },
        available: true,
    }));
    targets.extend(config.hosts.iter().map(|host| ExecutionTargetSummary {
        name: host.name.clone(),
        display_name: host.display_name.clone(),
        kind: ExecutionTargetKind::Ssh,
        available: true,
    }));
    let total = targets.len();
    Json(ExecutionTargetsResponse { targets, total })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_omits_internal_configuration() {
        let json = serde_json::to_value(response(&Config::default())).unwrap();
        for internal in [
            "paths",
            "rest_api",
            "notifications",
            "mcp",
            "hosts",
            "targets",
        ] {
            assert!(
                json.get(internal).is_none(),
                "{internal} must not be public"
            );
        }
    }

    #[test]
    fn patch_preserves_unmentioned_fields_and_replaces_arrays() {
        let mut config = Config::default();
        let original_timeout = config.agents.step_timeout;
        let patch: UpdateConfigurationRequest = serde_json::from_value(serde_json::json!({
            "queue": { "priority_order": ["high", "normal"] }
        }))
        .unwrap();
        apply_patch(&mut config, patch);
        assert_eq!(config.queue.priority_order, ["high", "normal"]);
        assert_eq!(config.agents.step_timeout, original_timeout);
    }

    #[test]
    fn patch_rejects_unknown_and_null_fields() {
        assert!(
            serde_json::from_value::<UpdateConfigurationRequest>(serde_json::json!({
                "paths": {}
            }))
            .is_err()
        );
        assert!(contains_null(
            &serde_json::json!({"queue": {"auto_assign": null}})
        ));
        let empty_nested: UpdateConfigurationRequest =
            serde_json::from_value(serde_json::json!({"ui": {"panel_names": {}}})).unwrap();
        assert!(empty_nested.is_empty());
    }
}
