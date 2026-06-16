//! Embedded collection bundles
//!
//! This module provides compile-time embedded collections with their
//! issuetype definitions. Each collection is self-contained with its own
//! copies of JSON schemas and markdown templates.

pub mod fetch;
pub mod manifest;

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

impl EmbeddedCollection {
    /// Parse the embedded `collection.json` manifest.
    pub fn manifest_parsed(&self) -> Result<manifest::CollectionManifest, serde_json::Error> {
        manifest::CollectionManifest::from_json(self.manifest)
    }
}

/// All embedded collections
pub static EMBEDDED_COLLECTIONS: &[EmbeddedCollection] = &[
    // Simple collection: TASK only
    EmbeddedCollection {
        name: "simple",
        manifest: include_str!("simple/collection.json"),
        issuetypes: &[EmbeddedIssueType {
            key: "TASK",
            schema_json: include_str!("simple/TASK.json"),
            template_md: include_str!("simple/TASK.md"),
        }],
    },
    // Dev Kanban collection: TASK, FEAT, FIX
    EmbeddedCollection {
        name: "dev_kanban",
        manifest: include_str!("dev_kanban/collection.json"),
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
        manifest: include_str!("devops_kanban/collection.json"),
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
        manifest: include_str!("operator/collection.json"),
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
    // Full collection: All 8 issuetypes
    EmbeddedCollection {
        name: "full",
        manifest: include_str!("full/collection.json"),
        issuetypes: &[
            EmbeddedIssueType {
                key: "TASK",
                schema_json: include_str!("full/TASK.json"),
                template_md: include_str!("full/TASK.md"),
            },
            EmbeddedIssueType {
                key: "FEAT",
                schema_json: include_str!("full/FEAT.json"),
                template_md: include_str!("full/FEAT.md"),
            },
            EmbeddedIssueType {
                key: "FIX",
                schema_json: include_str!("full/FIX.json"),
                template_md: include_str!("full/FIX.md"),
            },
            EmbeddedIssueType {
                key: "SPIKE",
                schema_json: include_str!("full/SPIKE.json"),
                template_md: include_str!("full/SPIKE.md"),
            },
            EmbeddedIssueType {
                key: "INV",
                schema_json: include_str!("full/INV.json"),
                template_md: include_str!("full/INV.md"),
            },
            EmbeddedIssueType {
                key: "ASSESS",
                schema_json: include_str!("full/ASSESS.json"),
                template_md: include_str!("full/ASSESS.md"),
            },
            EmbeddedIssueType {
                key: "SYNC",
                schema_json: include_str!("full/SYNC.json"),
                template_md: include_str!("full/SYNC.md"),
            },
            EmbeddedIssueType {
                key: "INIT",
                schema_json: include_str!("full/INIT.json"),
                template_md: include_str!("full/INIT.md"),
            },
        ],
    },
    // Ralph Loop collection: PRD, STORY, RLOOP
    EmbeddedCollection {
        name: "ralph_loop",
        manifest: include_str!("ralph_loop/collection.json"),
        issuetypes: &[
            EmbeddedIssueType {
                key: "PRD",
                schema_json: include_str!("ralph_loop/PRD.json"),
                template_md: include_str!("ralph_loop/PRD.md"),
            },
            EmbeddedIssueType {
                key: "STORY",
                schema_json: include_str!("ralph_loop/STORY.json"),
                template_md: include_str!("ralph_loop/STORY.md"),
            },
            EmbeddedIssueType {
                key: "RLOOP",
                schema_json: include_str!("ralph_loop/RLOOP.json"),
                template_md: include_str!("ralph_loop/RLOOP.md"),
            },
        ],
    },
    // JR Orchestration collection: feature/task/review/rebase workflows
    EmbeddedCollection {
        name: "jr_orchestration",
        manifest: include_str!("jr_orchestration/collection.json"),
        issuetypes: &[
            EmbeddedIssueType {
                key: "JRPLAN",
                schema_json: include_str!("jr_orchestration/JRPLAN.json"),
                template_md: include_str!("jr_orchestration/JRPLAN.md"),
            },
            EmbeddedIssueType {
                key: "JRFEAT",
                schema_json: include_str!("jr_orchestration/JRFEAT.json"),
                template_md: include_str!("jr_orchestration/JRFEAT.md"),
            },
            EmbeddedIssueType {
                key: "JRTASK",
                schema_json: include_str!("jr_orchestration/JRTASK.json"),
                template_md: include_str!("jr_orchestration/JRTASK.md"),
            },
            EmbeddedIssueType {
                key: "JRREV",
                schema_json: include_str!("jr_orchestration/JRREV.json"),
                template_md: include_str!("jr_orchestration/JRREV.md"),
            },
            EmbeddedIssueType {
                key: "JRREBASE",
                schema_json: include_str!("jr_orchestration/JRREBASE.json"),
                template_md: include_str!("jr_orchestration/JRREBASE.md"),
            },
        ],
    },
    // Elves Overnight collection: staged batch, PR landing, reporting
    EmbeddedCollection {
        name: "elves_overnight",
        manifest: include_str!("elves_overnight/collection.json"),
        issuetypes: &[
            EmbeddedIssueType {
                key: "ELVSTAGE",
                schema_json: include_str!("elves_overnight/ELVSTAGE.json"),
                template_md: include_str!("elves_overnight/ELVSTAGE.md"),
            },
            EmbeddedIssueType {
                key: "ELVBATCH",
                schema_json: include_str!("elves_overnight/ELVBATCH.json"),
                template_md: include_str!("elves_overnight/ELVBATCH.md"),
            },
            EmbeddedIssueType {
                key: "LANDPR",
                schema_json: include_str!("elves_overnight/LANDPR.json"),
                template_md: include_str!("elves_overnight/LANDPR.md"),
            },
            EmbeddedIssueType {
                key: "ELVRPT",
                schema_json: include_str!("elves_overnight/ELVRPT.json"),
                template_md: include_str!("elves_overnight/ELVRPT.md"),
            },
        ],
    },
];

/// Embedded schema files for issue types that need structured output
#[derive(Debug, Clone)]
pub struct EmbeddedSchema {
    pub name: &'static str,
    pub content: &'static str,
}

/// All embedded schema files
pub static EMBEDDED_SCHEMAS: &[EmbeddedSchema] = &[EmbeddedSchema {
    name: "project_analysis.schema.json",
    content: include_str!("../schemas/project_analysis.schema.json"),
}];

/// Get an embedded schema by name
#[allow(dead_code)]
pub fn get_embedded_schema(name: &str) -> Option<&'static str> {
    EMBEDDED_SCHEMAS
        .iter()
        .find(|s| s.name == name)
        .map(|s| s.content)
}

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
    use crate::templates::schema::TemplateSchema;

    #[test]
    fn test_embedded_collections_count() {
        assert_eq!(EMBEDDED_COLLECTIONS.len(), 8);
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

        let full = get_embedded_collection("full").unwrap();
        assert_eq!(full.name, "full");
        assert_eq!(full.issuetypes.len(), 8);

        let ralph = get_embedded_collection("ralph_loop").unwrap();
        assert_eq!(ralph.name, "ralph_loop");
        assert_eq!(ralph.issuetypes.len(), 3);

        let jr = get_embedded_collection("jr_orchestration").unwrap();
        assert_eq!(jr.name, "jr_orchestration");
        assert_eq!(jr.issuetypes.len(), 5);

        let elves = get_embedded_collection("elves_overnight").unwrap();
        assert_eq!(elves.name, "elves_overnight");
        assert_eq!(elves.issuetypes.len(), 4);
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
        assert!(names.contains(&"full"));
    }

    #[test]
    fn test_embedded_manifests_parse_and_match_issuetypes() {
        for collection in EMBEDDED_COLLECTIONS {
            let manifest = collection
                .manifest_parsed()
                .unwrap_or_else(|e| panic!("{} manifest must parse: {e}", collection.name));
            assert_eq!(manifest.id, collection.name, "id == name");
            assert_eq!(manifest.schema_version, 1);
            // The manifest's issue_types must list exactly the embedded files,
            // in the same order, so the docs producer can emit them all.
            let manifest_keys: Vec<&str> = manifest
                .issue_types
                .iter()
                .map(|e| e.key.as_str())
                .collect();
            let embedded_keys: Vec<&str> = collection.issuetypes.iter().map(|it| it.key).collect();
            assert_eq!(
                manifest_keys, embedded_keys,
                "{} manifest issue_types must match embedded files",
                collection.name
            );
            // Embedded manifests omit checksums (bytes are compiled in/trusted).
            for entry in &manifest.issue_types {
                assert!(entry.schema_checksum.is_empty());
            }
        }
    }

    #[test]
    fn test_agentic_loop_collections_have_valid_issuetypes() {
        for name in ["ralph_loop", "jr_orchestration", "elves_overnight"] {
            let collection = get_embedded_collection(name).unwrap();
            for issue_type in collection.issuetypes {
                let schema = TemplateSchema::from_json(issue_type.schema_json)
                    .unwrap_or_else(|e| panic!("{name}/{} schema must parse: {e}", issue_type.key));
                schema.validate().unwrap_or_else(|errors| {
                    panic!("{name}/{} schema invalid: {errors:?}", issue_type.key)
                });
            }
        }
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
