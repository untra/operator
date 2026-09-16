//! TUI bridge for installing a Premium licence from the setup wizard.
//!
//! Mirrors [`crate::app::git_onboarding`]: the wizard records the request, the
//! app performs it against `Config`, and the outcome is written back.

impl crate::app::App {
    pub(super) fn install_license_from_setup(&mut self, key: &str) {
        let outcome = crate::licensing::install(&self.config, key).map_err(|e| e.to_string());
        if let Some(setup) = self.setup_screen.as_mut() {
            setup.set_license_outcome(outcome);
        }
    }
}
