//! Template initialization for first-time setup.
//!
//! This module handles copying embedded template files to the filesystem
//! when the templates directory doesn't exist.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::info;

use crate::templates::TemplateType;

/// Collection preset for initial template setup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionPreset {
    /// Simple: TASK only
    Simple,
    /// Dev Kanban: TASK, FEAT, FIX
    DevKanban,
    /// DevOps Kanban: TASK, FEAT, FIX, SPIKE, INV
    DevopsKanban,
    /// Operator: ASSESS, SYNC, INIT
    Operator,
}

impl CollectionPreset {
    /// Get the directory name for this preset
    pub fn dir_name(&self) -> &'static str {
        match self {
            CollectionPreset::Simple => "simple",
            CollectionPreset::DevKanban => "dev_kanban",
            CollectionPreset::DevopsKanban => "devops_kanban",
            CollectionPreset::Operator => "operator",
        }
    }

    /// Get the description for this preset
    pub fn description(&self) -> &'static str {
        match self {
            CollectionPreset::Simple => "Simple collection with TASK only",
            CollectionPreset::DevKanban => "Developer kanban with TASK, FEAT, FIX",
            CollectionPreset::DevopsKanban => "DevOps kanban with TASK, FEAT, FIX, SPIKE, INV",
            CollectionPreset::Operator => "Operator workflows: ASSESS, SYNC, INIT",
        }
    }

    /// Get the template types included in this preset
    pub fn template_types(&self) -> &'static [TemplateType] {
        match self {
            CollectionPreset::Simple => &[TemplateType::Task],
            CollectionPreset::DevKanban => {
                &[TemplateType::Task, TemplateType::Feature, TemplateType::Fix]
            }
            CollectionPreset::DevopsKanban => &[
                TemplateType::Task,
                TemplateType::Feature,
                TemplateType::Fix,
                TemplateType::Spike,
                TemplateType::Investigation,
            ],
            CollectionPreset::Operator => {
                &[TemplateType::Assess, TemplateType::Sync, TemplateType::Init]
            }
        }
    }

    /// Get the priority order for this preset
    pub fn priority_order(&self) -> &'static [&'static str] {
        match self {
            CollectionPreset::Simple => &["TASK"],
            CollectionPreset::DevKanban => &["FIX", "FEAT", "TASK"],
            CollectionPreset::DevopsKanban => &["INV", "FIX", "FEAT", "SPIKE", "TASK"],
            CollectionPreset::Operator => &["ASSESS", "SYNC", "INIT"],
        }
    }

    /// Get all default presets
    pub fn defaults() -> &'static [CollectionPreset] {
        &[
            CollectionPreset::DevKanban,
            CollectionPreset::DevopsKanban,
            CollectionPreset::Simple,
            CollectionPreset::Operator,
        ]
    }
}

/// Initialize the templates directory with default collections
///
/// Creates the directory structure:
/// ```text
/// .tickets/templates/
/// ├── dev_kanban/
/// │   ├── collection.toml
/// │   └── issues/
/// │       ├── TASK.json
/// │       ├── FEAT.json
/// │       └── FIX.json
/// └── ...
/// ```
pub fn init_default_templates(templates_path: &Path) -> Result<()> {
    if templates_path.exists() {
        info!(
            "Templates directory already exists: {}",
            templates_path.display()
        );
        return Ok(());
    }

    info!(
        "Initializing default templates at {}",
        templates_path.display()
    );

    // Create templates for each default preset
    for preset in CollectionPreset::defaults() {
        init_collection(templates_path, *preset)?;
    }

    Ok(())
}

/// Initialize a single collection preset
pub fn init_collection(templates_path: &Path, preset: CollectionPreset) -> Result<()> {
    let collection_path = templates_path.join(preset.dir_name());
    let issues_path = collection_path.join("issues");

    // Create directories
    fs::create_dir_all(&issues_path).with_context(|| {
        format!(
            "Failed to create issues directory: {}",
            issues_path.display()
        )
    })?;

    // Write collection.toml
    let collection_toml = format!(
        r#"# {} collection
description = "{}"
priority_order = {:?}
"#,
        preset.dir_name(),
        preset.description(),
        preset.priority_order()
    );
    fs::write(collection_path.join("collection.toml"), collection_toml)?;

    // Copy template files
    for template_type in preset.template_types() {
        let json_content = template_type.schema();
        let filename = format!("{}.json", template_type.as_str());
        fs::write(issues_path.join(&filename), json_content)?;
        info!("Created template: {}/{}", preset.dir_name(), filename);
    }

    info!(
        "Initialized collection '{}' with {} issue types",
        preset.dir_name(),
        preset.template_types().len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_default_templates() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        init_default_templates(&templates_path).unwrap();

        // Check that directories were created
        assert!(templates_path.exists());
        assert!(templates_path.join("dev_kanban/issues").exists());
        assert!(templates_path.join("devops_kanban/issues").exists());
        assert!(templates_path.join("simple/issues").exists());
        assert!(templates_path.join("operator/issues").exists());

        // Check that template files were created
        assert!(templates_path.join("dev_kanban/issues/TASK.json").exists());
        assert!(templates_path.join("dev_kanban/issues/FEAT.json").exists());
        assert!(templates_path.join("dev_kanban/issues/FIX.json").exists());
        assert!(templates_path
            .join("devops_kanban/issues/SPIKE.json")
            .exists());
        assert!(templates_path
            .join("devops_kanban/issues/INV.json")
            .exists());

        // Check collection.toml was created
        assert!(templates_path.join("dev_kanban/collection.toml").exists());
    }

    #[test]
    fn test_init_skips_existing() {
        let temp_dir = TempDir::new().unwrap();
        let templates_path = temp_dir.path().join("templates");

        // Create the directory first
        fs::create_dir_all(&templates_path).unwrap();
        fs::write(templates_path.join("marker.txt"), "existing").unwrap();

        // This should not fail and should not overwrite
        init_default_templates(&templates_path).unwrap();

        // Marker file should still exist
        assert!(templates_path.join("marker.txt").exists());
    }

    #[test]
    fn test_collection_preset_types() {
        assert_eq!(CollectionPreset::Simple.template_types().len(), 1);
        assert_eq!(CollectionPreset::DevKanban.template_types().len(), 3);
        assert_eq!(CollectionPreset::DevopsKanban.template_types().len(), 5);
        assert_eq!(CollectionPreset::Operator.template_types().len(), 3);
    }
}
