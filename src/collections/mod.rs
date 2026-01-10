//! Embedded collection bundles
//!
//! This module provides compile-time embedded collections with their
//! issuetype definitions. Each collection is self-contained with its own
//! copies of JSON schemas and markdown templates.

/// A single embedded issuetype with JSON schema and markdown template
#[derive(Debug, Clone)]
pub struct EmbeddedIssueType {
    pub key: &'static str,
    pub schema_json: &'static str,
    pub template_md: &'static str,
}

/// An embedded collection with manifest and issuetypes
#[derive(Debug, Clone)]
pub struct EmbeddedCollection {
    pub name: &'static str,
    pub manifest: &'static str,
    pub issuetypes: &'static [EmbeddedIssueType],
}

/// All embedded collections
pub static EMBEDDED_COLLECTIONS: &[EmbeddedCollection] = &[
    // Simple collection: TASK only
    EmbeddedCollection {
        name: "simple",
        manifest: include_str!("simple/collection.toml"),
        issuetypes: &[EmbeddedIssueType {
            key: "TASK",
            schema_json: include_str!("simple/TASK.json"),
            template_md: include_str!("simple/TASK.md"),
        }],
    },
    // Dev Kanban collection: TASK, FEAT, FIX
    EmbeddedCollection {
        name: "dev_kanban",
        manifest: include_str!("dev_kanban/collection.toml"),
        issuetypes: &[
            EmbeddedIssueType {
                key: "TASK",
                schema_json: include_str!("dev_kanban/TASK.json"),
                template_md: include_str!("dev_kanban/TASK.md"),
            },
            EmbeddedIssueType {
                key: "FEAT",
                schema_json: include_str!("dev_kanban/FEAT.json"),
                template_md: include_str!("dev_kanban/FEAT.md"),
            },
            EmbeddedIssueType {
                key: "FIX",
                schema_json: include_str!("dev_kanban/FIX.json"),
                template_md: include_str!("dev_kanban/FIX.md"),
            },
        ],
    },
    // DevOps Kanban collection: TASK, FEAT, FIX, SPIKE, INV
    EmbeddedCollection {
        name: "devops_kanban",
        manifest: include_str!("devops_kanban/collection.toml"),
        issuetypes: &[
            EmbeddedIssueType {
                key: "TASK",
                schema_json: include_str!("devops_kanban/TASK.json"),
                template_md: include_str!("devops_kanban/TASK.md"),
            },
            EmbeddedIssueType {
                key: "FEAT",
                schema_json: include_str!("devops_kanban/FEAT.json"),
                template_md: include_str!("devops_kanban/FEAT.md"),
            },
            EmbeddedIssueType {
                key: "FIX",
                schema_json: include_str!("devops_kanban/FIX.json"),
                template_md: include_str!("devops_kanban/FIX.md"),
            },
            EmbeddedIssueType {
                key: "SPIKE",
                schema_json: include_str!("devops_kanban/SPIKE.json"),
                template_md: include_str!("devops_kanban/SPIKE.md"),
            },
            EmbeddedIssueType {
                key: "INV",
                schema_json: include_str!("devops_kanban/INV.json"),
                template_md: include_str!("devops_kanban/INV.md"),
            },
        ],
    },
    // Operator collection: ASSESS, SYNC, INIT, AGENT-SETUP, PROJECT-INIT
    EmbeddedCollection {
        name: "operator",
        manifest: include_str!("operator/collection.toml"),
        issuetypes: &[
            EmbeddedIssueType {
                key: "ASSESS",
                schema_json: include_str!("operator/ASSESS.json"),
                template_md: include_str!("operator/ASSESS.md"),
            },
            EmbeddedIssueType {
                key: "SYNC",
                schema_json: include_str!("operator/SYNC.json"),
                template_md: include_str!("operator/SYNC.md"),
            },
            EmbeddedIssueType {
                key: "INIT",
                schema_json: include_str!("operator/INIT.json"),
                template_md: include_str!("operator/INIT.md"),
            },
            EmbeddedIssueType {
                key: "AGENT-SETUP",
                schema_json: include_str!("operator/AGENT-SETUP.json"),
                template_md: include_str!("operator/AGENT-SETUP.md"),
            },
            EmbeddedIssueType {
                key: "PROJECT-INIT",
                schema_json: include_str!("operator/PROJECT-INIT.json"),
                template_md: include_str!("operator/PROJECT-INIT.md"),
            },
        ],
    },
    // Backstage Full collection: All 8 issuetypes
    EmbeddedCollection {
        name: "backstage_full",
        manifest: include_str!("backstage_full/collection.toml"),
        issuetypes: &[
            EmbeddedIssueType {
                key: "TASK",
                schema_json: include_str!("backstage_full/TASK.json"),
                template_md: include_str!("backstage_full/TASK.md"),
            },
            EmbeddedIssueType {
                key: "FEAT",
                schema_json: include_str!("backstage_full/FEAT.json"),
                template_md: include_str!("backstage_full/FEAT.md"),
            },
            EmbeddedIssueType {
                key: "FIX",
                schema_json: include_str!("backstage_full/FIX.json"),
                template_md: include_str!("backstage_full/FIX.md"),
            },
            EmbeddedIssueType {
                key: "SPIKE",
                schema_json: include_str!("backstage_full/SPIKE.json"),
                template_md: include_str!("backstage_full/SPIKE.md"),
            },
            EmbeddedIssueType {
                key: "INV",
                schema_json: include_str!("backstage_full/INV.json"),
                template_md: include_str!("backstage_full/INV.md"),
            },
            EmbeddedIssueType {
                key: "ASSESS",
                schema_json: include_str!("backstage_full/ASSESS.json"),
                template_md: include_str!("backstage_full/ASSESS.md"),
            },
            EmbeddedIssueType {
                key: "SYNC",
                schema_json: include_str!("backstage_full/SYNC.json"),
                template_md: include_str!("backstage_full/SYNC.md"),
            },
            EmbeddedIssueType {
                key: "INIT",
                schema_json: include_str!("backstage_full/INIT.json"),
                template_md: include_str!("backstage_full/INIT.md"),
            },
        ],
    },
];

/// Get an embedded collection by name
pub fn get_embedded_collection(name: &str) -> Option<&'static EmbeddedCollection> {
    EMBEDDED_COLLECTIONS.iter().find(|c| c.name == name)
}

/// List all embedded collection names
#[allow(dead_code)]
pub fn embedded_collection_names() -> Vec<&'static str> {
    EMBEDDED_COLLECTIONS.iter().map(|c| c.name).collect()
}

/// Get an embedded issuetype by key from any collection
#[allow(dead_code)]
pub fn get_embedded_issuetype(key: &str) -> Option<&'static EmbeddedIssueType> {
    for collection in EMBEDDED_COLLECTIONS {
        if let Some(it) = collection.issuetypes.iter().find(|it| it.key == key) {
            return Some(it);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_collections_count() {
        assert_eq!(EMBEDDED_COLLECTIONS.len(), 5);
    }

    #[test]
    fn test_get_embedded_collection() {
        let simple = get_embedded_collection("simple").unwrap();
        assert_eq!(simple.name, "simple");
        assert_eq!(simple.issuetypes.len(), 1);

        let dev = get_embedded_collection("dev_kanban").unwrap();
        assert_eq!(dev.name, "dev_kanban");
        assert_eq!(dev.issuetypes.len(), 3);

        let devops = get_embedded_collection("devops_kanban").unwrap();
        assert_eq!(devops.name, "devops_kanban");
        assert_eq!(devops.issuetypes.len(), 5);

        let operator = get_embedded_collection("operator").unwrap();
        assert_eq!(operator.name, "operator");
        assert_eq!(operator.issuetypes.len(), 5);

        let full = get_embedded_collection("backstage_full").unwrap();
        assert_eq!(full.name, "backstage_full");
        assert_eq!(full.issuetypes.len(), 8);
    }

    #[test]
    fn test_get_nonexistent_collection() {
        assert!(get_embedded_collection("nonexistent").is_none());
    }

    #[test]
    fn test_embedded_collection_names() {
        let names = embedded_collection_names();
        assert!(names.contains(&"simple"));
        assert!(names.contains(&"dev_kanban"));
        assert!(names.contains(&"devops_kanban"));
        assert!(names.contains(&"operator"));
        assert!(names.contains(&"backstage_full"));
    }

    #[test]
    fn test_get_embedded_issuetype() {
        let task = get_embedded_issuetype("TASK").unwrap();
        assert_eq!(task.key, "TASK");
        assert!(!task.schema_json.is_empty());
        assert!(!task.template_md.is_empty());

        let feat = get_embedded_issuetype("FEAT").unwrap();
        assert_eq!(feat.key, "FEAT");

        assert!(get_embedded_issuetype("NONEXISTENT").is_none());
    }
}
