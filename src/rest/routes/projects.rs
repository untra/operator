//! Project listing and assessment endpoints.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::backstage::analyzer::ProjectAnalysis;
use crate::queue::creator::{render_template, TicketCreator};
use crate::rest::dto::{AssessTicketResponse, ProjectSummary};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::templates::TemplateType;

/// List all configured projects with analysis data
#[utoipa::path(
    get,
    path = "/api/v1/projects",
    tag = "Projects",
    responses(
        (status = 200, description = "List of projects with analysis data", body = Vec<ProjectSummary>)
    )
)]
pub async fn list(State(state): State<ApiState>) -> Json<Vec<ProjectSummary>> {
    let config = &state.config;
    let projects_path = config.projects_path();
    let project_names = &config.projects;

    let mut summaries = Vec::new();

    for name in project_names {
        let project_dir = projects_path.join(name);
        let exists = project_dir.is_dir();
        let has_catalog_info = project_dir.join("catalog-info.yaml").is_file();
        let context_path = project_dir.join("project-context.json");
        let has_project_context = context_path.is_file();

        let mut summary = ProjectSummary {
            project_name: name.clone(),
            project_path: project_dir.to_string_lossy().to_string(),
            exists,
            has_catalog_info,
            has_project_context,
            kind: None,
            kind_confidence: None,
            kind_tier: None,
            languages: Vec::new(),
            frameworks: Vec::new(),
            databases: Vec::new(),
            has_docker: None,
            has_tests: None,
            ports: Vec::new(),
            env_var_count: 0,
            entry_point_count: 0,
            commands: Vec::new(),
        };

        // Try to read project-context.json for analysis data
        if has_project_context {
            if let Ok(content) = std::fs::read_to_string(&context_path) {
                if let Ok(analysis) = serde_json::from_str::<ProjectAnalysis>(&content) {
                    summary.kind = Some(analysis.kind_assessment.primary_kind);
                    summary.kind_confidence = Some(analysis.kind_assessment.confidence as f64);
                    summary.kind_tier = Some(analysis.kind_assessment.tier);
                    summary.languages = analysis
                        .languages
                        .iter()
                        .map(|l| l.display_name.clone())
                        .collect();
                    summary.frameworks = analysis
                        .frameworks
                        .iter()
                        .map(|f| f.display_name.clone())
                        .collect();
                    summary.databases = analysis
                        .databases
                        .iter()
                        .map(|d| d.display_name.clone())
                        .collect();
                    summary.has_docker =
                        Some(analysis.docker.has_dockerfile || analysis.docker.has_compose);
                    summary.has_tests = Some(!analysis.testing.is_empty());
                    summary.ports = analysis
                        .ports
                        .iter()
                        .filter_map(|p| p.port_number)
                        .collect();
                    summary.env_var_count = analysis.environment.len();
                    summary.entry_point_count = analysis.entry_points.len();

                    let cmds = &analysis.commands;
                    let mut cmd_names = Vec::new();
                    if cmds.start.is_some() {
                        cmd_names.push("start".to_string());
                    }
                    if cmds.dev.is_some() {
                        cmd_names.push("dev".to_string());
                    }
                    if cmds.test.is_some() {
                        cmd_names.push("test".to_string());
                    }
                    if cmds.build.is_some() {
                        cmd_names.push("build".to_string());
                    }
                    if cmds.lint.is_some() {
                        cmd_names.push("lint".to_string());
                    }
                    if cmds.fmt.is_some() {
                        cmd_names.push("fmt".to_string());
                    }
                    if cmds.typecheck.is_some() {
                        cmd_names.push("typecheck".to_string());
                    }
                    summary.commands = cmd_names;
                }
            }
        }

        summaries.push(summary);
    }

    Json(summaries)
}

/// Create an ASSESS ticket for a project
#[utoipa::path(
    post,
    path = "/api/v1/projects/{name}/assess",
    tag = "Projects",
    params(
        ("name" = String, Path, description = "Project name")
    ),
    responses(
        (status = 200, description = "ASSESS ticket created", body = AssessTicketResponse),
        (status = 404, description = "Project not found")
    )
)]
pub async fn assess(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<AssessTicketResponse>, ApiError> {
    let config = &state.config;

    // Validate project exists in config
    if !config.projects.contains(&name) {
        return Err(ApiError::NotFound(format!(
            "Project '{}' not found in configuration",
            name
        )));
    }

    let creator = TicketCreator::new(config);
    let template_type = TemplateType::Assess;

    // Generate default values and add summary
    let mut values = creator.generate_default_values(template_type, &name);
    values.insert(
        "summary".to_string(),
        format!("Assess {} for Backstage catalog", name),
    );

    // Render template content
    let template = template_type.template_content();
    let content = render_template(template, &values)?;

    // Write ticket file directly (no editor)
    let ticket_id = values
        .get("id")
        .cloned()
        .unwrap_or_else(|| "ASSESS-0000".to_string());
    let queue_path = config.tickets_path().join("queue");
    std::fs::create_dir_all(&queue_path)
        .map_err(|e| ApiError::InternalError(format!("Failed to create queue directory: {}", e)))?;

    let now = chrono::Utc::now();
    let timestamp = now.format("%Y%m%d-%H%M").to_string();
    let filename = format!("{}-ASSESS-{}-new-ticket.md", timestamp, name);
    let filepath = queue_path.join(&filename);

    std::fs::write(&filepath, &content)
        .map_err(|e| ApiError::InternalError(format!("Failed to write ticket file: {}", e)))?;

    Ok(Json(AssessTicketResponse {
        ticket_id,
        ticket_path: filepath.to_string_lossy().to_string(),
        project_name: name,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_list_empty_projects() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-projects"));

        let resp = list(State(state)).await;
        // Default config has no projects
        assert!(resp.0.is_empty());
    }

    #[tokio::test]
    async fn test_assess_unknown_project() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-projects"));

        let result = assess(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
    }
}
