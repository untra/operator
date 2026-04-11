//! Kanban Issue Type Sync Service
//!
//! Syncs issue types from external kanban providers into a local catalog,
//! and resolves kanban issue type refs to operator issuetype keys.

#![allow(dead_code)] // Infrastructure for kanban sync integration

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::api::providers::kanban::{ExternalIssueType, KanbanProvider};
use crate::issuetypes::kanban_type::{KanbanIssueType, KanbanIssueTypeRef};

/// Service for syncing and managing kanban issue types.
pub struct KanbanIssueTypeService {
    /// Root path for kanban catalog (e.g., `.tickets/operator/kanban`)
    catalog_root: PathBuf,
}

impl KanbanIssueTypeService {
    /// Create a new service with the given catalog root path.
    pub fn new(catalog_root: PathBuf) -> Self {
        Self { catalog_root }
    }

    /// Create from a tickets path (e.g., `.tickets`).
    pub fn from_tickets_path(tickets_path: &Path) -> Self {
        Self {
            catalog_root: tickets_path.join("operator/kanban"),
        }
    }

    /// Get the catalog file path for a provider/project.
    fn catalog_path(&self, provider: &str, project: &str) -> PathBuf {
        self.catalog_root
            .join(provider)
            .join(project)
            .join("issuetypes.json")
    }

    /// Sync issue types from a provider for a specific project.
    ///
    /// Fetches issue types from the provider API and writes them to the local catalog.
    /// Returns the synced types.
    pub async fn sync_issue_types(
        &self,
        provider: &dyn KanbanProvider,
        project_key: &str,
    ) -> Result<Vec<KanbanIssueType>> {
        let provider_name = provider.name();
        info!(
            "Syncing kanban issue types from {}/{}",
            provider_name, project_key
        );

        let external_types = provider
            .get_issue_types(project_key)
            .await
            .context("Failed to fetch issue types from provider")?;

        let source_kind = match provider_name {
            "linear" => "label",
            _ => "issuetype",
        };

        let now = chrono::Utc::now().to_rfc3339();
        let kanban_types: Vec<KanbanIssueType> = external_types
            .iter()
            .map(|et| {
                KanbanIssueType::from_external(et, provider_name, project_key, source_kind, &now)
            })
            .collect();

        self.write_catalog(provider_name, project_key, &kanban_types)?;

        info!(
            "Synced {} kanban issue types for {}/{}",
            kanban_types.len(),
            provider_name,
            project_key
        );

        Ok(kanban_types)
    }

    /// List kanban types from the persisted catalog for a provider/project.
    pub fn list_kanban_types(&self, provider: &str, project: &str) -> Result<Vec<KanbanIssueType>> {
        self.read_catalog(provider, project)
    }

    /// List all kanban types across all providers and projects.
    pub fn list_all_kanban_types(&self) -> Result<Vec<KanbanIssueType>> {
        let mut all = Vec::new();

        if !self.catalog_root.exists() {
            return Ok(all);
        }

        // Iterate provider directories
        for provider_entry in fs::read_dir(&self.catalog_root)? {
            let provider_entry = provider_entry?;
            if !provider_entry.file_type()?.is_dir() {
                continue;
            }
            let provider_name = provider_entry.file_name().to_string_lossy().to_string();

            // Iterate project directories
            for project_entry in fs::read_dir(provider_entry.path())? {
                let project_entry = project_entry?;
                if !project_entry.file_type()?.is_dir() {
                    continue;
                }
                let project_name = project_entry.file_name().to_string_lossy().to_string();

                match self.read_catalog(&provider_name, &project_name) {
                    Ok(types) => all.extend(types),
                    Err(e) => {
                        warn!(
                            "Failed to read kanban catalog for {}/{}: {}",
                            provider_name, project_name, e
                        );
                    }
                }
            }
        }

        Ok(all)
    }

    /// Resolve a kanban issue type ref to an operator issuetype key.
    ///
    /// Looks up the ref's ID in `type_mappings`. Returns `None` if unmapped.
    pub fn resolve_operator_key(
        kanban_ref: &KanbanIssueTypeRef,
        type_mappings: &HashMap<String, String>,
    ) -> Option<String> {
        type_mappings.get(&kanban_ref.id).cloned()
    }

    /// Resolve operator key from multiple kanban refs (e.g., Linear labels).
    ///
    /// Sorts refs by name for deterministic resolution, picks first mapped ref.
    /// Returns `None` if no refs are mapped.
    pub fn resolve_operator_key_from_refs(
        kanban_refs: &[KanbanIssueTypeRef],
        type_mappings: &HashMap<String, String>,
    ) -> Option<String> {
        let mut sorted: Vec<_> = kanban_refs.iter().collect();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        for r in sorted {
            if let Some(key) = type_mappings.get(&r.id) {
                return Some(key.clone());
            }
        }

        None
    }

    /// Attempt to resolve a legacy name-based mapping key against the synced catalog.
    ///
    /// If a `type_mappings` key is not a known external ID, looks up by synced name
    /// and returns the resolved ID if found.
    pub fn resolve_legacy_mapping(
        &self,
        mapping_key: &str,
        provider: &str,
        project: &str,
    ) -> Option<String> {
        let types = self.read_catalog(provider, project).ok()?;
        // Check if the key is already a valid ID
        if types.iter().any(|t| t.id == mapping_key) {
            return Some(mapping_key.to_string());
        }
        // Try matching by name (case-insensitive)
        types
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(mapping_key))
            .map(|t| t.id.clone())
    }

    /// Write the kanban catalog to disk.
    fn write_catalog(
        &self,
        provider: &str,
        project: &str,
        types: &[KanbanIssueType],
    ) -> Result<()> {
        let path = self.catalog_path(provider, project);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create catalog dir: {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(types)?;
        fs::write(&path, json)?;
        debug!("Wrote kanban catalog to {}", path.display());
        Ok(())
    }

    /// Read the kanban catalog from disk.
    fn read_catalog(&self, provider: &str, project: &str) -> Result<Vec<KanbanIssueType>> {
        let path = self.catalog_path(provider, project);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read catalog: {}", path.display()))?;
        let types: Vec<KanbanIssueType> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse catalog: {}", path.display()))?;
        Ok(types)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_external_types() -> Vec<ExternalIssueType> {
        vec![
            ExternalIssueType {
                id: "10001".to_string(),
                name: "Bug".to_string(),
                description: Some("A bug report".to_string()),
                icon_url: None,
                custom_fields: vec![],
            },
            ExternalIssueType {
                id: "10002".to_string(),
                name: "Story".to_string(),
                description: Some("A user story".to_string()),
                icon_url: None,
                custom_fields: vec![],
            },
            ExternalIssueType {
                id: "10003".to_string(),
                name: "Task".to_string(),
                description: None,
                icon_url: None,
                custom_fields: vec![],
            },
        ]
    }

    fn create_service_with_catalog(types: &[KanbanIssueType]) -> (KanbanIssueTypeService, TempDir) {
        let tmp = TempDir::new().unwrap();
        let service = KanbanIssueTypeService::new(tmp.path().to_path_buf());

        if !types.is_empty() {
            let provider = &types[0].provider;
            let project = &types[0].project;
            service.write_catalog(provider, project, types).unwrap();
        }

        (service, tmp)
    }

    #[test]
    fn test_catalog_path() {
        let service = KanbanIssueTypeService::new(PathBuf::from("/tmp/kanban"));
        let path = service.catalog_path("jira", "PROJ");
        assert_eq!(path, PathBuf::from("/tmp/kanban/jira/PROJ/issuetypes.json"));
    }

    #[test]
    fn test_from_tickets_path() {
        let service = KanbanIssueTypeService::from_tickets_path(Path::new(".tickets"));
        assert_eq!(
            service.catalog_root,
            PathBuf::from(".tickets/operator/kanban")
        );
    }

    #[test]
    fn test_write_and_read_catalog() {
        let tmp = TempDir::new().unwrap();
        let service = KanbanIssueTypeService::new(tmp.path().to_path_buf());

        let types: Vec<KanbanIssueType> = sample_external_types()
            .iter()
            .map(|et| {
                KanbanIssueType::from_external(
                    et,
                    "jira",
                    "PROJ",
                    "issuetype",
                    "2026-04-05T12:00:00Z",
                )
            })
            .collect();

        service.write_catalog("jira", "PROJ", &types).unwrap();
        let read_types = service.read_catalog("jira", "PROJ").unwrap();

        assert_eq!(types.len(), read_types.len());
        assert_eq!(types[0].id, read_types[0].id);
        assert_eq!(types[1].name, read_types[1].name);
    }

    #[test]
    fn test_read_catalog_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let service = KanbanIssueTypeService::new(tmp.path().to_path_buf());

        let types = service.read_catalog("jira", "NONEXISTENT").unwrap();
        assert!(types.is_empty());
    }

    #[test]
    fn test_list_kanban_types() {
        let types: Vec<KanbanIssueType> = sample_external_types()
            .iter()
            .map(|et| {
                KanbanIssueType::from_external(
                    et,
                    "jira",
                    "PROJ",
                    "issuetype",
                    "2026-04-05T12:00:00Z",
                )
            })
            .collect();

        let (service, _tmp) = create_service_with_catalog(&types);
        let listed = service.list_kanban_types("jira", "PROJ").unwrap();
        assert_eq!(listed.len(), 3);
    }

    #[test]
    fn test_list_all_kanban_types() {
        let tmp = TempDir::new().unwrap();
        let service = KanbanIssueTypeService::new(tmp.path().to_path_buf());

        // Write two catalogs
        let jira_types: Vec<KanbanIssueType> = sample_external_types()[..2]
            .iter()
            .map(|et| {
                KanbanIssueType::from_external(
                    et,
                    "jira",
                    "PROJ",
                    "issuetype",
                    "2026-04-05T12:00:00Z",
                )
            })
            .collect();
        service.write_catalog("jira", "PROJ", &jira_types).unwrap();

        let linear_types = vec![KanbanIssueType::from_external(
            &sample_external_types()[0],
            "linear",
            "TEAM",
            "label",
            "2026-04-05T12:00:00Z",
        )];
        service
            .write_catalog("linear", "TEAM", &linear_types)
            .unwrap();

        let all = service.list_all_kanban_types().unwrap();
        assert_eq!(all.len(), 3); // 2 jira + 1 linear
    }

    #[test]
    fn test_list_all_empty() {
        let tmp = TempDir::new().unwrap();
        let service = KanbanIssueTypeService::new(tmp.path().to_path_buf());
        let all = service.list_all_kanban_types().unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn test_resolve_operator_key() {
        let mut mappings = HashMap::new();
        mappings.insert("10001".to_string(), "FIX".to_string());
        mappings.insert("10002".to_string(), "FEAT".to_string());

        let r = KanbanIssueTypeRef {
            id: "10001".to_string(),
            name: "Bug".to_string(),
        };
        assert_eq!(
            KanbanIssueTypeService::resolve_operator_key(&r, &mappings),
            Some("FIX".to_string())
        );

        let unmapped = KanbanIssueTypeRef {
            id: "99999".to_string(),
            name: "Unknown".to_string(),
        };
        assert_eq!(
            KanbanIssueTypeService::resolve_operator_key(&unmapped, &mappings),
            None
        );
    }

    #[test]
    fn test_resolve_operator_key_from_refs_sorted() {
        let mut mappings = HashMap::new();
        mappings.insert("label-bug".to_string(), "FIX".to_string());
        mappings.insert("label-feat".to_string(), "FEAT".to_string());

        let refs = vec![
            KanbanIssueTypeRef {
                id: "label-feat".to_string(),
                name: "Feature".to_string(),
            },
            KanbanIssueTypeRef {
                id: "label-bug".to_string(),
                name: "Bug".to_string(),
            },
        ];

        // Sorted by name: Bug < Feature, so Bug matches first -> FIX
        let result = KanbanIssueTypeService::resolve_operator_key_from_refs(&refs, &mappings);
        assert_eq!(result, Some("FIX".to_string()));
    }

    #[test]
    fn test_resolve_operator_key_from_refs_empty() {
        let mappings = HashMap::new();
        let result = KanbanIssueTypeService::resolve_operator_key_from_refs(&[], &mappings);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_legacy_mapping_by_id() {
        let types = vec![KanbanIssueType::from_external(
            &sample_external_types()[0],
            "jira",
            "PROJ",
            "issuetype",
            "2026-04-05T12:00:00Z",
        )];

        let (service, _tmp) = create_service_with_catalog(&types);

        // Exact ID match
        let result = service.resolve_legacy_mapping("10001", "jira", "PROJ");
        assert_eq!(result, Some("10001".to_string()));
    }

    #[test]
    fn test_resolve_legacy_mapping_by_name() {
        let types = vec![KanbanIssueType::from_external(
            &sample_external_types()[0],
            "jira",
            "PROJ",
            "issuetype",
            "2026-04-05T12:00:00Z",
        )];

        let (service, _tmp) = create_service_with_catalog(&types);

        // Name-based lookup (case-insensitive)
        let result = service.resolve_legacy_mapping("bug", "jira", "PROJ");
        assert_eq!(result, Some("10001".to_string()));
    }

    #[test]
    fn test_resolve_legacy_mapping_not_found() {
        let types = vec![KanbanIssueType::from_external(
            &sample_external_types()[0],
            "jira",
            "PROJ",
            "issuetype",
            "2026-04-05T12:00:00Z",
        )];

        let (service, _tmp) = create_service_with_catalog(&types);

        let result = service.resolve_legacy_mapping("Nonexistent", "jira", "PROJ");
        assert_eq!(result, None);
    }
}
