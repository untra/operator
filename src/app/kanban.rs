use anyhow::Result;

use crate::state::State;

use super::App;

impl App {
    /// Sync all configured kanban collections
    pub(super) async fn run_kanban_sync_all(&mut self) -> Result<()> {
        let collections = self.kanban_sync_service.configured_collections();
        let total = collections.len();

        if total == 0 {
            self.sync_confirm_dialog.hide();
            self.sync_status_message = Some("No kanban providers configured".to_string());
            return Ok(());
        }

        let mut created_total = 0;
        let mut skipped_total = 0;
        let mut error_count = 0;

        for (i, collection) in collections.iter().enumerate() {
            self.sync_confirm_dialog.set_syncing(i, total);

            match self
                .kanban_sync_service
                .sync_collection(&collection.provider, &collection.project_key)
                .await
            {
                Ok(result) => {
                    created_total += result.created.len();
                    skipped_total += result.skipped.len();
                    if !result.is_success() {
                        error_count += result.errors.len();
                    }
                    tracing::info!(
                        provider = %collection.provider,
                        project = %collection.project_key,
                        "Synced collection: {}",
                        result.summary()
                    );
                }
                Err(e) => {
                    error_count += 1;
                    tracing::warn!(
                        provider = %collection.provider,
                        project = %collection.project_key,
                        "Failed to sync collection: {}",
                        e
                    );
                }
            }
        }

        // Build summary message
        let summary = if error_count > 0 {
            format!(
                "Sync complete: {created_total} created, {skipped_total} skipped, {error_count} errors"
            )
        } else {
            format!("Sync complete: {created_total} tickets created, {skipped_total} skipped")
        };

        self.sync_confirm_dialog.set_complete(&summary);
        self.sync_status_message = Some(summary);
        self.sync_confirm_dialog.hide();

        // Trigger queue refresh
        self.run_manual_sync()?;

        Ok(())
    }

    /// Show the collection switch dialog
    pub(super) fn show_collection_dialog(&mut self) {
        // Get project context from selected queue item if any
        let project_context = self.dashboard.selected_ticket().map(|t| t.project.as_str());
        self.collection_dialog.show(
            &self.issue_type_registry,
            self.issue_type_registry.active_collection_name(),
            project_context,
        );
    }

    /// Show the kanban providers view
    pub(super) fn show_kanban_view(&mut self) {
        let collections = self.kanban_sync_service.configured_collections();
        if collections.is_empty() {
            // No kanban providers configured, show a message
            self.sync_status_message = Some(
                "No kanban providers configured. Add [kanban] section to config.toml".to_string(),
            );
            return;
        }
        self.kanban_view.show(collections);
    }

    /// Handle collection switch result
    pub(super) fn handle_collection_switch(
        &mut self,
        result: crate::ui::CollectionSwitchResult,
    ) -> Result<()> {
        // Activate the collection in the registry
        if let Err(e) = self
            .issue_type_registry
            .activate_collection(&result.collection_name)
        {
            tracing::warn!(
                "Failed to activate collection '{}': {}",
                result.collection_name,
                e
            );
            return Ok(());
        }

        // Persist the preference
        if let Some(project) = result.project_scope {
            // Per-project preference
            let mut state = State::load(&self.config)?;
            state.set_project_collection(&project, &result.collection_name)?;
            tracing::info!(
                "Set collection '{}' for project '{}'",
                result.collection_name,
                project
            );
        } else {
            // Global preference - update config
            self.config.templates.active_collection = Some(result.collection_name.clone());
            if let Err(e) = self.config.save() {
                tracing::warn!("Failed to save config: {}", e);
            }
            tracing::info!("Set global collection to '{}'", result.collection_name);
        }

        Ok(())
    }
}
