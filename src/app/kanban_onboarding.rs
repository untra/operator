//! App-side async dispatch for the kanban onboarding dialog.
//!
//! The dialog is purely UI state; this module reacts to actions emitted
//! by the dialog and calls `services::kanban_onboarding` directly.

use anyhow::Result;

use crate::rest::dto::{
    JiraCredentials, JiraSessionEnv, KanbanProviderKind, LinearCredentials, LinearSessionEnv,
    ListKanbanProjectsRequest, SetKanbanSessionEnvRequest, ValidateKanbanCredentialsRequest,
    WriteJiraConfigBody, WriteKanbanConfigRequest, WriteLinearConfigBody,
};
use crate::services::kanban_onboarding;
use crate::ui::{KanbanOnboardingAction, KanbanOnboardingProject, KanbanOnboardingProvider};

use super::App;

/// Stash credentials between the validate and writeConfig stages so the
/// dialog doesn't have to expose them.
#[derive(Debug, Clone, Default)]
pub(crate) struct KanbanOnboardingCreds {
    pub jira: Option<JiraCredsInflight>,
    pub linear: Option<LinearCredsInflight>,
}

#[derive(Debug, Clone)]
pub(crate) struct JiraCredsInflight {
    pub domain: String,
    pub email: String,
    pub api_token: String,
}

#[derive(Debug, Clone)]
pub(crate) struct LinearCredsInflight {
    pub api_key: String,
}

impl App {
    /// Show the kanban onboarding dialog (entry point from the kanban view).
    pub(super) fn show_kanban_onboarding_dialog(&mut self) {
        self.kanban_onboarding_dialog.show();
        self.kanban_onboarding_creds = KanbanOnboardingCreds::default();
    }

    /// Handle an action emitted by the kanban onboarding dialog.
    /// Performs async work (validate / list projects / write config / sync)
    /// and updates the dialog state via its setters.
    pub(super) async fn handle_kanban_onboarding_action(
        &mut self,
        action: KanbanOnboardingAction,
    ) -> Result<()> {
        match action {
            KanbanOnboardingAction::None
            | KanbanOnboardingAction::PickedProvider(_)
            | KanbanOnboardingAction::Cancelled
            | KanbanOnboardingAction::Done => {
                // Pure UI transitions — no async work needed.
            }
            KanbanOnboardingAction::SubmitJiraCreds {
                domain,
                email,
                token,
            } => {
                // Stash for later (write_config + set_session_env)
                self.kanban_onboarding_creds.jira = Some(JiraCredsInflight {
                    domain: domain.clone(),
                    email: email.clone(),
                    api_token: token.clone(),
                });

                // Validate
                let req = ValidateKanbanCredentialsRequest {
                    provider: KanbanProviderKind::Jira,
                    jira: Some(JiraCredentials {
                        domain: domain.clone(),
                        email: email.clone(),
                        api_token: token.clone(),
                    }),
                    linear: None,
                    github: None,
                };
                let resp = match kanban_onboarding::validate_credentials(req).await {
                    Ok(r) => r,
                    Err(e) => {
                        self.kanban_onboarding_dialog
                            .set_error(format!("Could not reach provider: {e:?}"));
                        return Ok(());
                    }
                };
                if !resp.valid {
                    self.kanban_onboarding_dialog.set_error(
                        resp.error
                            .unwrap_or_else(|| "Validation failed".to_string()),
                    );
                    return Ok(());
                }
                let Some(jira_details) = resp.jira else {
                    self.kanban_onboarding_dialog
                        .set_error("Validation succeeded but no Jira details returned".to_string());
                    return Ok(());
                };
                self.kanban_onboarding_dialog
                    .set_validation_jira(jira_details.account_id, jira_details.display_name);

                // Now list projects
                let list_req = ListKanbanProjectsRequest {
                    provider: KanbanProviderKind::Jira,
                    jira: Some(JiraCredentials {
                        domain,
                        email,
                        api_token: token,
                    }),
                    linear: None,
                    github: None,
                };
                let projects = match kanban_onboarding::list_projects(list_req).await {
                    Ok(r) => r.projects,
                    Err(e) => {
                        self.kanban_onboarding_dialog
                            .set_error(format!("Failed to list projects: {e:?}"));
                        return Ok(());
                    }
                };
                if projects.is_empty() {
                    self.kanban_onboarding_dialog
                        .set_error("No Jira projects found. Check your permissions.".to_string());
                    return Ok(());
                }
                let dialog_projects: Vec<KanbanOnboardingProject> = projects
                    .into_iter()
                    .map(|p| KanbanOnboardingProject {
                        id: p.id,
                        key: p.key,
                        name: p.name,
                    })
                    .collect();
                self.kanban_onboarding_dialog.set_projects(dialog_projects);
            }
            KanbanOnboardingAction::SubmitLinearCreds { api_key } => {
                // Stash creds; workspace_key gets filled in after we know the team
                self.kanban_onboarding_creds.linear = Some(LinearCredsInflight {
                    api_key: api_key.clone(),
                });

                let req = ValidateKanbanCredentialsRequest {
                    provider: KanbanProviderKind::Linear,
                    jira: None,
                    linear: Some(LinearCredentials {
                        api_key: api_key.clone(),
                    }),
                    github: None,
                };
                let resp = match kanban_onboarding::validate_credentials(req).await {
                    Ok(r) => r,
                    Err(e) => {
                        self.kanban_onboarding_dialog
                            .set_error(format!("Could not reach provider: {e:?}"));
                        return Ok(());
                    }
                };
                if !resp.valid {
                    self.kanban_onboarding_dialog.set_error(
                        resp.error
                            .unwrap_or_else(|| "Validation failed".to_string()),
                    );
                    return Ok(());
                }
                let Some(linear_details) = resp.linear else {
                    self.kanban_onboarding_dialog.set_error(
                        "Validation succeeded but no Linear details returned".to_string(),
                    );
                    return Ok(());
                };
                self.kanban_onboarding_dialog.set_validation_linear(
                    linear_details.user_id,
                    linear_details.user_name,
                    linear_details.org_name,
                );

                // For Linear we already have the team list from validate;
                // turn it into the project picker.
                if linear_details.teams.is_empty() {
                    self.kanban_onboarding_dialog
                        .set_error("No Linear teams found. Check your permissions.".to_string());
                    return Ok(());
                }
                let dialog_projects: Vec<KanbanOnboardingProject> = linear_details
                    .teams
                    .into_iter()
                    .map(|t| KanbanOnboardingProject {
                        id: t.id,
                        key: t.key,
                        name: t.name,
                    })
                    .collect();
                self.kanban_onboarding_dialog.set_projects(dialog_projects);
            }
            KanbanOnboardingAction::PickedProject {
                provider,
                project_key,
                project_name,
            } => {
                // Build write_config + set_session_env requests from stashed creds
                let result = self
                    .finish_kanban_onboarding(provider, project_key, project_name)
                    .await;
                if let Err(e) = result {
                    self.kanban_onboarding_dialog
                        .set_error(format!("Failed to write config: {e}"));
                }
            }
            KanbanOnboardingAction::CopyExportBlock => {
                // No-op on the Rust side — the dialog displays the block;
                // the user can manually copy from the terminal. Future
                // enhancement: integrate with arboard for system clipboard.
                self.sync_status_message = Some(
                    "Export block displayed in dialog — copy manually from the terminal"
                        .to_string(),
                );
            }
        }
        Ok(())
    }

    /// Final step: write config + set session env + sync issue types.
    async fn finish_kanban_onboarding(
        &mut self,
        provider: KanbanOnboardingProvider,
        project_key: String,
        project_name: String,
    ) -> Result<()> {
        match provider {
            KanbanOnboardingProvider::Jira => {
                let creds = self
                    .kanban_onboarding_creds
                    .jira
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Missing stashed Jira credentials"))?;
                let account_id = self.kanban_onboarding_dialog.jira_account_id.clone();
                let api_key_env = "OPERATOR_JIRA_API_KEY".to_string();

                // Write config
                let write_req = WriteKanbanConfigRequest {
                    provider: KanbanProviderKind::Jira,
                    jira: Some(WriteJiraConfigBody {
                        domain: creds.domain.clone(),
                        email: creds.email.clone(),
                        api_key_env: api_key_env.clone(),
                        project_key: project_key.clone(),
                        sync_user_id: account_id,
                    }),
                    linear: None,
                    github: None,
                };
                kanban_onboarding::write_config(write_req, None)
                    .map_err(|e| anyhow::anyhow!("write_config failed: {e:?}"))?;

                // Set session env
                let env_req = SetKanbanSessionEnvRequest {
                    provider: KanbanProviderKind::Jira,
                    jira: Some(JiraSessionEnv {
                        domain: creds.domain,
                        email: creds.email,
                        api_token: creds.api_token,
                        api_key_env,
                    }),
                    linear: None,
                    github: None,
                };
                let env_resp = kanban_onboarding::set_session_env(env_req);

                // Sync issue types (best effort — non-fatal)
                self.try_sync_kanban_issue_types("jira", &project_key).await;

                self.kanban_onboarding_dialog.set_success(
                    format!("Jira project {project_name} configured!"),
                    env_resp.shell_export_block,
                );
                Ok(())
            }
            KanbanOnboardingProvider::Linear => {
                let creds = self
                    .kanban_onboarding_creds
                    .linear
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Missing stashed Linear credentials"))?;
                let user_id = self.kanban_onboarding_dialog.linear_user_id.clone();
                let api_key_env = "OPERATOR_LINEAR_API_KEY".to_string();
                // Use the project_key (team key) as the workspace key for Linear
                let workspace_key = project_key.clone();

                let write_req = WriteKanbanConfigRequest {
                    provider: KanbanProviderKind::Linear,
                    jira: None,
                    linear: Some(WriteLinearConfigBody {
                        workspace_key: workspace_key.clone(),
                        api_key_env: api_key_env.clone(),
                        project_key: project_key.clone(),
                        sync_user_id: user_id,
                    }),
                    github: None,
                };
                kanban_onboarding::write_config(write_req, None)
                    .map_err(|e| anyhow::anyhow!("write_config failed: {e:?}"))?;

                let env_req = SetKanbanSessionEnvRequest {
                    provider: KanbanProviderKind::Linear,
                    jira: None,
                    linear: Some(LinearSessionEnv {
                        api_key: creds.api_key,
                        api_key_env,
                    }),
                    github: None,
                };
                let env_resp = kanban_onboarding::set_session_env(env_req);

                self.try_sync_kanban_issue_types("linear", &project_key)
                    .await;

                self.kanban_onboarding_dialog.set_success(
                    format!("Linear team {project_name} configured!"),
                    env_resp.shell_export_block,
                );
                Ok(())
            }
        }
    }

    /// Best-effort issue type sync after onboarding completes.
    /// Non-fatal — onboarding succeeds even if the sync fails.
    async fn try_sync_kanban_issue_types(&mut self, provider: &str, project_key: &str) {
        use crate::api::providers::kanban::get_provider_from_config;
        use crate::config::Config;
        use crate::services::kanban_issuetype_service::KanbanIssueTypeService;

        // Reload fresh config from disk so the just-written provider is found.
        let fresh_config = match Config::load(None) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Could not reload config for issue type sync: {}", e);
                return;
            }
        };
        let kanban_provider =
            match get_provider_from_config(&fresh_config.kanban, provider, project_key) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("Could not build provider for sync: {}", e);
                    return;
                }
            };
        let service = KanbanIssueTypeService::from_tickets_path(std::path::Path::new(
            &fresh_config.paths.tickets,
        ));
        match service
            .sync_issue_types(kanban_provider.as_ref(), project_key)
            .await
        {
            Ok(types) => {
                tracing::info!(
                    "Synced {} issue types for {}/{}",
                    types.len(),
                    provider,
                    project_key
                );
            }
            Err(e) => {
                tracing::warn!("Issue type sync failed: {}", e);
            }
        }
    }
}
