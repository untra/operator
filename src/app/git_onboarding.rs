//! TUI orchestration for shared git provider onboarding.

pub(crate) use crate::services::git_onboarding::{
    apply_git_provider, complete_git_onboarding, resolve_onboarding_with_config,
    shell_export_block, token_env_for, validate_token_with_config, OnboardingStep,
};

impl crate::app::App {
    pub(super) fn connect_git_provider_from_setup(&mut self, slug: &str) {
        let step = resolve_onboarding_with_config(&self.config, slug);
        let status = match step {
            Some(OnboardingStep::InstallCli {
                install_url,
                provider_display,
            }) => {
                let _ = crate::app::status_actions::open_in_browser(&install_url);
                format!("install the {provider_display} CLI, then retry")
            }
            Some(OnboardingStep::CollectToken {
                pat_url,
                provider,
                provider_display,
                placeholder,
            }) => {
                let _ = crate::app::status_actions::open_in_browser(&pat_url);
                self.git_token_dialog
                    .show(&provider, &provider_display, &pat_url, &placeholder);
                return;
            }
            Some(OnboardingStep::AutoConfigured {
                username,
                token,
                provider,
                ..
            }) => match apply_git_provider(&mut self.config, &provider, &token) {
                Ok(()) => format!("connected as {username}"),
                Err(error) => format!("failed: {error}"),
            },
            None => "unsupported provider".to_string(),
        };
        let export_hint = token_env_for(&self.config, slug).map(|env| shell_export_block(&env));
        if let Some(setup) = self.setup_screen.as_mut() {
            setup.set_git_provider_status(slug, status);
            setup.git_export_hint = export_hint;
        }
    }
}
