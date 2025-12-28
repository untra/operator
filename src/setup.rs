//! Non-interactive setup for operator workspace initialization
//!
//! This module provides CLI-based setup functionality that mirrors
//! the TUI setup wizard but can be run non-interactively with flags.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::agents::{generate_status_script, generate_tmux_conf};
use crate::backstage::scaffold::{BackstageScaffold, ScaffoldOptions};
use crate::config::{CollectionPreset, Config};
use crate::templates::TemplateType;

/// Common optional fields that can be configured for TASK and propagated to other types
pub const COMMON_OPTIONAL_FIELDS: &[&str] = &["priority", "points", "user_story"];

/// Options for workspace setup
#[derive(Debug, Clone)]
pub struct SetupOptions {
    /// Collection preset to use
    pub preset: CollectionPreset,
    /// Whether to enable backstage
    pub backstage_enabled: bool,
    /// Overwrite existing files
    pub force: bool,
    /// Optional fields to include (propagated to all types)
    /// Only common fields (priority, points, user_story) are filtered
    pub task_fields: Vec<String>,
}

impl Default for SetupOptions {
    fn default() -> Self {
        Self {
            preset: CollectionPreset::Simple,
            backstage_enabled: false,
            force: false,
            // Default: all optional fields enabled
            task_fields: COMMON_OPTIONAL_FIELDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

/// Result of setup operation
#[derive(Debug)]
pub struct SetupResult {
    pub directories_created: Vec<PathBuf>,
    pub files_created: Vec<PathBuf>,
    pub files_skipped: Vec<PathBuf>,
    pub config_path: PathBuf,
}

/// Parse collection preset from string
pub fn parse_collection_preset(s: &str) -> Result<CollectionPreset> {
    match s.to_lowercase().as_str() {
        "simple" => Ok(CollectionPreset::Simple),
        "dev-kanban" | "devkanban" | "dev_kanban" => Ok(CollectionPreset::DevKanban),
        "devops-kanban" | "devopskanban" | "devops_kanban" => Ok(CollectionPreset::DevopsKanban),
        other => bail!(
            "Unknown collection preset: '{}'. Valid options: simple, dev-kanban, devops-kanban",
            other
        ),
    }
}

/// Initialize workspace with the given options
pub fn initialize_workspace(config: &mut Config, options: &SetupOptions) -> Result<SetupResult> {
    let tickets_path = config.tickets_path();
    let mut result = SetupResult {
        directories_created: Vec::new(),
        files_created: Vec::new(),
        files_skipped: Vec::new(),
        config_path: tickets_path.join("operator").join("config.toml"),
    };

    // Create directories
    let dirs = [
        tickets_path.join("queue"),
        tickets_path.join("in-progress"),
        tickets_path.join("completed"),
        tickets_path.join("templates"),
        tickets_path.join("operator"),
        tickets_path.join("operator").join("templates"),
    ];

    for dir in &dirs {
        if !dir.exists() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
            result.directories_created.push(dir.clone());
        }
    }

    // Get effective issue types from preset
    let issue_types = if options.preset == CollectionPreset::Custom {
        config.templates.collection.clone()
    } else {
        options.preset.issue_types()
    };

    // Write template files
    for template_type in TemplateType::all() {
        let type_str = template_type.as_str();
        if !issue_types.contains(&type_str.to_string()) {
            continue;
        }

        // Write markdown template
        let md_filename = template_filename(*template_type);
        let md_path = tickets_path.join("templates").join(md_filename);
        write_file_if_allowed(
            &md_path,
            template_type.template_content(),
            options.force,
            &mut result,
        )?;

        // Write JSON schema (with field filtering applied)
        let json_filename = schema_filename(*template_type);
        let json_path = tickets_path.join("templates").join(json_filename);
        let filtered_schema = filter_schema_fields(template_type.schema(), &options.task_fields)?;
        write_file_if_allowed(&json_path, &filtered_schema, options.force, &mut result)?;
    }

    // Write operator template files for interpolation
    let operator_templates = tickets_path.join("operator").join("templates");
    write_file_if_allowed(
        &operator_templates.join("ACCEPTANCE_CRITERIA.md"),
        include_str!("templates/ACCEPTANCE_CRITERIA.md"),
        options.force,
        &mut result,
    )?;
    write_file_if_allowed(
        &operator_templates.join("DEFINITION_OF_DONE.md"),
        include_str!("templates/DEFINITION_OF_DONE.md"),
        options.force,
        &mut result,
    )?;
    write_file_if_allowed(
        &operator_templates.join("DEFINITION_OF_READY.md"),
        include_str!("templates/DEFINITION_OF_READY.md"),
        options.force,
        &mut result,
    )?;

    // Update config with preset
    config.templates.preset = options.preset;
    if options.preset == CollectionPreset::Custom {
        // Keep existing collection
    } else {
        config.templates.collection = issue_types;
    }

    // Configure backstage if enabled
    config.backstage.enabled = options.backstage_enabled;

    // Generate tmux config
    generate_tmux_config(config)?;

    // Generate backstage scaffold if enabled
    if options.backstage_enabled {
        let backstage_path = config.backstage_path();
        if !BackstageScaffold::exists(&backstage_path) || options.force {
            let scaffold_options = ScaffoldOptions::from_config(config);
            let scaffold = BackstageScaffold::new(backstage_path, scaffold_options);
            scaffold.generate()?;
        }
    }

    // Save config (must be after directories are created)
    config.save()?;

    Ok(result)
}

/// Get template filename for a template type
fn template_filename(template_type: TemplateType) -> &'static str {
    match template_type {
        TemplateType::Feature => "feature.md",
        TemplateType::Fix => "fix.md",
        TemplateType::Task => "task.md",
        TemplateType::Spike => "spike.md",
        TemplateType::Investigation => "investigation.md",
        TemplateType::Assess => "assess.md",
        TemplateType::Sync => "sync.md",
        TemplateType::Init => "init.md",
    }
}

/// Get schema filename for a template type
fn schema_filename(template_type: TemplateType) -> &'static str {
    match template_type {
        TemplateType::Feature => "feature.json",
        TemplateType::Fix => "fix.json",
        TemplateType::Task => "task.json",
        TemplateType::Spike => "spike.json",
        TemplateType::Investigation => "investigation.json",
        TemplateType::Assess => "assess.json",
        TemplateType::Sync => "sync.json",
        TemplateType::Init => "init.json",
    }
}

/// Write file if it doesn't exist or force is true
fn write_file_if_allowed(
    path: &PathBuf,
    content: &str,
    force: bool,
    result: &mut SetupResult,
) -> Result<()> {
    if path.exists() && !force {
        result.files_skipped.push(path.clone());
        return Ok(());
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
    result.files_created.push(path.clone());
    Ok(())
}

/// Filter out disabled common fields from a JSON schema
/// Only filters common optional fields (priority, context)
/// Type-specific fields (severity, scope, etc.) are preserved
pub fn filter_schema_fields(schema_json: &str, enabled_fields: &[String]) -> Result<String> {
    let mut schema: serde_json::Value =
        serde_json::from_str(schema_json).context("Failed to parse schema JSON")?;

    if let Some(fields) = schema.get_mut("fields").and_then(|f| f.as_array_mut()) {
        fields.retain(|field| {
            if let Some(name) = field.get("name").and_then(|n| n.as_str()) {
                // Keep if not a common optional field, or if it's enabled
                if COMMON_OPTIONAL_FIELDS.contains(&name) {
                    enabled_fields.contains(&name.to_string())
                } else {
                    true // Keep type-specific fields
                }
            } else {
                true // Keep if no name (shouldn't happen)
            }
        });

        // Renumber display_order to be consecutive
        for (i, field) in fields.iter_mut().enumerate() {
            if let Some(obj) = field.as_object_mut() {
                obj.insert("display_order".to_string(), serde_json::json!(i));
            }
        }
    }

    serde_json::to_string_pretty(&schema).context("Failed to serialize filtered schema")
}

/// Generate tmux configuration files
fn generate_tmux_config(config: &mut Config) -> Result<()> {
    let state_path = config.state_path();
    let tmux_conf_path = config.tmux_config_path();
    let status_script_path = config.tmux_status_script_path();

    // Ensure parent directory exists
    if let Some(parent) = tmux_conf_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmux_conf_content = generate_tmux_conf(&status_script_path, &state_path);
    fs::write(&tmux_conf_path, tmux_conf_content)?;

    let status_script_content = generate_status_script();
    fs::write(&status_script_path, status_script_content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&status_script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&status_script_path, perms)?;
    }

    config.tmux.config_generated = true;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_collection_preset_simple() {
        assert_eq!(
            parse_collection_preset("simple").unwrap(),
            CollectionPreset::Simple
        );
        assert_eq!(
            parse_collection_preset("SIMPLE").unwrap(),
            CollectionPreset::Simple
        );
    }

    #[test]
    fn test_parse_collection_preset_dev_kanban() {
        assert_eq!(
            parse_collection_preset("dev-kanban").unwrap(),
            CollectionPreset::DevKanban
        );
        assert_eq!(
            parse_collection_preset("devkanban").unwrap(),
            CollectionPreset::DevKanban
        );
        assert_eq!(
            parse_collection_preset("dev_kanban").unwrap(),
            CollectionPreset::DevKanban
        );
    }

    #[test]
    fn test_parse_collection_preset_devops_kanban() {
        assert_eq!(
            parse_collection_preset("devops-kanban").unwrap(),
            CollectionPreset::DevopsKanban
        );
        assert_eq!(
            parse_collection_preset("devopskanban").unwrap(),
            CollectionPreset::DevopsKanban
        );
    }

    #[test]
    fn test_parse_collection_preset_invalid() {
        assert!(parse_collection_preset("invalid").is_err());
    }

    #[test]
    fn test_setup_options_default() {
        let options = SetupOptions::default();
        assert_eq!(options.preset, CollectionPreset::Simple);
        assert!(!options.backstage_enabled);
        assert!(!options.force);
    }

    #[test]
    fn test_template_filename() {
        assert_eq!(template_filename(TemplateType::Feature), "feature.md");
        assert_eq!(template_filename(TemplateType::Fix), "fix.md");
        assert_eq!(template_filename(TemplateType::Task), "task.md");
        assert_eq!(template_filename(TemplateType::Spike), "spike.md");
        assert_eq!(
            template_filename(TemplateType::Investigation),
            "investigation.md"
        );
    }

    #[test]
    fn test_schema_filename() {
        assert_eq!(schema_filename(TemplateType::Feature), "feature.json");
        assert_eq!(schema_filename(TemplateType::Fix), "fix.json");
        assert_eq!(schema_filename(TemplateType::Task), "task.json");
    }

    #[test]
    fn test_initialize_workspace_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path().join(".tickets");

        let mut config = Config::default();
        config.paths.tickets = tickets_path.to_string_lossy().to_string();
        config.paths.state = tickets_path.join("operator").to_string_lossy().to_string();

        let options = SetupOptions::default();
        let result = initialize_workspace(&mut config, &options).unwrap();

        assert!(!result.directories_created.is_empty());
        assert!(tickets_path.join("queue").exists());
        assert!(tickets_path.join("in-progress").exists());
        assert!(tickets_path.join("completed").exists());
        assert!(tickets_path.join("templates").exists());
        assert!(tickets_path.join("operator").exists());
    }

    #[test]
    fn test_initialize_workspace_creates_task_template() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path().join(".tickets");

        let mut config = Config::default();
        config.paths.tickets = tickets_path.to_string_lossy().to_string();
        config.paths.state = tickets_path.join("operator").to_string_lossy().to_string();

        let options = SetupOptions {
            preset: CollectionPreset::Simple,
            ..Default::default()
        };
        initialize_workspace(&mut config, &options).unwrap();

        // Simple preset should create TASK template
        assert!(tickets_path.join("templates/task.md").exists());
        assert!(tickets_path.join("templates/task.json").exists());
    }

    #[test]
    fn test_initialize_workspace_skips_existing_without_force() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path().join(".tickets");
        fs::create_dir_all(tickets_path.join("templates")).unwrap();

        // Create an existing file
        let existing_file = tickets_path.join("templates/task.md");
        fs::write(&existing_file, "existing content").unwrap();

        let mut config = Config::default();
        config.paths.tickets = tickets_path.to_string_lossy().to_string();
        config.paths.state = tickets_path.join("operator").to_string_lossy().to_string();

        let options = SetupOptions {
            force: false,
            ..Default::default()
        };
        let result = initialize_workspace(&mut config, &options).unwrap();

        // File should be in skipped list
        assert!(result.files_skipped.iter().any(|p| p.ends_with("task.md")));

        // Content should be unchanged
        let content = fs::read_to_string(&existing_file).unwrap();
        assert_eq!(content, "existing content");
    }

    #[test]
    fn test_initialize_workspace_overwrites_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path().join(".tickets");
        fs::create_dir_all(tickets_path.join("templates")).unwrap();

        // Create an existing file
        let existing_file = tickets_path.join("templates/task.md");
        fs::write(&existing_file, "existing content").unwrap();

        let mut config = Config::default();
        config.paths.tickets = tickets_path.to_string_lossy().to_string();
        config.paths.state = tickets_path.join("operator").to_string_lossy().to_string();

        let options = SetupOptions {
            force: true,
            ..Default::default()
        };
        let result = initialize_workspace(&mut config, &options).unwrap();

        // File should be in created list
        assert!(result.files_created.iter().any(|p| p.ends_with("task.md")));

        // Content should be overwritten
        let content = fs::read_to_string(&existing_file).unwrap();
        assert_ne!(content, "existing content");
    }

    #[test]
    fn test_initialize_workspace_with_backstage() {
        let temp_dir = TempDir::new().unwrap();
        let tickets_path = temp_dir.path().join(".tickets");

        let mut config = Config::default();
        config.paths.tickets = tickets_path.to_string_lossy().to_string();
        config.paths.state = tickets_path.join("operator").to_string_lossy().to_string();

        let options = SetupOptions {
            backstage_enabled: true,
            ..Default::default()
        };
        initialize_workspace(&mut config, &options).unwrap();

        assert!(config.backstage.enabled);
    }
}
