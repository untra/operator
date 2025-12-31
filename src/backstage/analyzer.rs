#![allow(dead_code)]

//! Project analysis engine for structured codebase assessment.
//!
//! This module provides types for representing project analysis results,
//! including detected languages, frameworks, databases, Docker configuration,
//! ports, and test frameworks. The analysis output conforms to the
//! `project_analysis.schema.json` JSON schema for use with Claude's
//! structured output mode.
//!
//! ## Usage
//!
//! The ASSESS issuetype's analyze step uses the schema file to produce
//! structured JSON output. Detection logic is stubbed for future implementation.
//!
//! ## Example
//!
//! ```ignore
//! let analysis = ProjectAnalysis {
//!     project_name: "my-service".to_string(),
//!     project_path: "/path/to/my-service".to_string(),
//!     analyzed_at: "2024-12-25T00:00:00Z".to_string(),
//!     kind_assessment: KindAssessment { ... },
//!     languages: vec![...],
//!     frameworks: vec![...],
//!     databases: vec![...],
//!     docker: DockerDetection { ... },
//!     ports: vec![...],
//!     testing: vec![...],
//!     file_stats: FileStats { ... },
//! };
//!
//! let json = serde_json::to_string_pretty(&analysis)?;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Complete project analysis result.
///
/// This is the top-level structure that conforms to `project_analysis.schema.json`.
/// Claude fills this structure during the ASSESS issuetype's analyze step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectAnalysis {
    /// Project directory name
    pub project_name: String,

    /// Absolute path to project root
    pub project_path: String,

    /// ISO 8601 timestamp of analysis
    pub analyzed_at: String,

    /// Detected project Kind from taxonomy
    pub kind_assessment: KindAssessment,

    /// Detected programming languages
    pub languages: Vec<LanguageDetection>,

    /// Detected frameworks and libraries
    pub frameworks: Vec<FrameworkDetection>,

    /// Detected database systems
    pub databases: Vec<DatabaseDetection>,

    /// Docker configuration detection
    pub docker: DockerDetection,

    /// Detected port configurations
    pub ports: Vec<PortDetection>,

    /// Detected test frameworks
    pub testing: Vec<TestFrameworkDetection>,

    /// File statistics for context
    pub file_stats: FileStats,

    /// Executable commands for common operations
    pub commands: Commands,

    /// Key entry points into the codebase
    pub entry_points: Vec<EntryPoint>,

    /// Environment variables used by the project
    pub environment: Vec<EnvVar>,
}

/// Kind assessment from the 25-Kind taxonomy.
///
/// Maps to one of the Kinds defined in `taxonomy.toml`:
/// - Foundation (1-4): infrastructure, identity-access, config-policy, monorepo-meta
/// - Standards (5-10): design-system, software-library, proto-sdk, blueprint, security-tooling, compliance-audit
/// - Engines (11-16): ml-model, data-etl, microservice, api-gateway, ui-frontend, internal-tool
/// - Ecosystem (17-21): build-tool, e2e-test, docs-site, playbook, cli-devtool
/// - Noncurrent (22-25): reference-example, experiment-sandbox, archival-fork, test-data-fixtures
///
/// ## Tier-Based Assessment Scoping
///
/// Not all assessment types apply to all tiers:
/// - **Frameworks**: Assessed for Standards, Engines, Ecosystem (not Foundation, Noncurrent)
/// - **Databases**: Assessed for Engines, Ecosystem (not Foundation, Standards, Noncurrent)
/// - **Testing**: Assessed for Standards, Engines, Ecosystem (not Foundation, Noncurrent)
///
/// Foundation tier projects are pure infrastructure with no application-level code.
/// Noncurrent tier projects are low-importance repos where detailed analysis is skipped.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KindAssessment {
    /// Primary detected Kind key (e.g., "microservice", "ui-frontend")
    pub primary_kind: String,

    /// Confidence score 0.0-1.0
    pub confidence: f32,

    /// Taxonomy tier: foundation, standards, engines, or ecosystem
    pub tier: String,

    /// Files that matched Kind patterns
    pub matching_files: Vec<String>,

    /// Alternative Kind candidates with their scores
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<KindCandidate>,
}

/// Alternative Kind candidate with confidence score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KindCandidate {
    /// Kind key
    pub kind: String,

    /// Confidence score 0.0-1.0
    pub confidence: f32,

    /// Number of file pattern matches
    pub match_count: usize,
}

/// Language detection result.
///
/// Supports both known languages (rust, typescript, python, etc.) and
/// unknown/emerging languages via free-form string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanguageDetection {
    /// Language identifier (e.g., "rust", "typescript", "python")
    pub language: String,

    /// Human-readable name (e.g., "Rust", "TypeScript", "Python")
    pub display_name: String,

    /// Confidence score 0.0-1.0
    pub confidence: f32,

    /// Whether this is the primary/dominant language
    pub is_primary: bool,

    /// Number of files in this language
    pub file_count: usize,

    /// Evidence supporting this detection
    pub evidence: Vec<Evidence>,
}

/// Framework/library detection result.
///
/// Supports both known frameworks and unknown/custom frameworks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameworkDetection {
    /// Framework identifier (e.g., "axum", "react", "django")
    pub framework: String,

    /// Human-readable name (e.g., "Axum", "React", "Django")
    pub display_name: String,

    /// Framework category
    pub category: FrameworkCategory,

    /// Confidence score 0.0-1.0
    pub confidence: f32,

    /// Version if detected (e.g., "0.7.5", "18.2.0")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Evidence supporting this detection
    pub evidence: Vec<Evidence>,
}

/// Framework categories for classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkCategory {
    /// Web frameworks (Axum, Express, Django, etc.)
    Web,
    /// Object-Relational Mappers (Diesel, SQLAlchemy, Prisma)
    Orm,
    /// Testing frameworks (Jest, Pytest)
    Testing,
    /// Build tools (Webpack, Vite, esbuild)
    Build,
    /// Logging frameworks (tracing, winston)
    Logging,
    /// Serialization libraries (serde, Jackson)
    Serialization,
    /// CLI frameworks (clap, commander)
    Cli,
    /// Async runtimes (Tokio, asyncio)
    Async,
    /// API frameworks (tonic, GraphQL)
    Api,
    /// UI frameworks (React, Vue, Yew)
    Ui,
    /// Other/uncategorized
    Other,
}

/// Database detection result.
///
/// Supports both known databases and unknown/custom databases.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseDetection {
    /// Database identifier (e.g., "postgres", "mongodb", "redis")
    pub database: String,

    /// Human-readable name (e.g., "PostgreSQL", "MongoDB", "Redis")
    pub display_name: String,

    /// Database category
    pub category: DatabaseCategory,

    /// Confidence score 0.0-1.0
    pub confidence: f32,

    /// Default or detected port
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Evidence supporting this detection
    pub evidence: Vec<Evidence>,
}

/// Database categories for classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseCategory {
    /// SQL databases (PostgreSQL, MySQL, SQLite)
    Relational,
    /// Document stores (MongoDB, CouchDB)
    Document,
    /// Key-value stores (Redis as KV, etcd)
    KeyValue,
    /// Graph databases (Neo4j, Neptune)
    Graph,
    /// Time-series databases (InfluxDB, TimescaleDB)
    TimeSeries,
    /// Message queues (RabbitMQ, Kafka)
    MessageQueue,
    /// Search engines (Elasticsearch, Meilisearch)
    Search,
    /// Caching systems (Redis as cache, Memcached)
    Cache,
}

/// Docker configuration detection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DockerDetection {
    /// Whether Dockerfile exists
    pub has_dockerfile: bool,

    /// Whether docker-compose.yml/yaml exists
    pub has_compose: bool,

    /// Base images detected from Dockerfile(s)
    pub base_images: Vec<DockerImage>,

    /// Service names from docker-compose
    pub compose_services: Vec<String>,

    /// Evidence supporting this detection
    pub evidence: Vec<Evidence>,
}

/// Docker base image information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DockerImage {
    /// Image name (e.g., "rust", "node", "postgres")
    pub image: String,

    /// Image tag if specified (e.g., "1.75", "20-alpine")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Build stage name if multi-stage (e.g., "builder", "runtime")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
}

/// Port detection result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortDetection {
    /// Port type category
    pub port_type: PortType,

    /// Actual port number if detected
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port_number: Option<u16>,

    /// Environment variable name if port is configured via env
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env_var: Option<String>,

    /// Confidence score 0.0-1.0
    pub confidence: f32,

    /// Evidence supporting this detection
    pub evidence: Vec<Evidence>,
}

/// Port type categories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PortType {
    /// HTTP server port (typically 80, 8080, 3000)
    Http,
    /// HTTPS server port (typically 443, 8443)
    Https,
    /// gRPC server port (typically 50051)
    Grpc,
    /// Database connection port (5432, 3306, 27017)
    Database,
    /// Redis port (typically 6379)
    Redis,
    /// RabbitMQ port (typically 5672)
    Rabbitmq,
    /// WebSocket port
    Websocket,
    /// Metrics/observability port (Prometheus 9090, etc.)
    Metrics,
    /// Debug port (Node 9229, etc.)
    Debug,
    /// Other/unknown port type
    Other,
}

/// Test framework detection result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestFrameworkDetection {
    /// Framework identifier (e.g., "cargo_test", "jest", "pytest")
    pub framework: String,

    /// Human-readable name (e.g., "Cargo Test", "Jest", "Pytest")
    pub display_name: String,

    /// Test category
    pub category: TestCategory,

    /// Confidence score 0.0-1.0
    pub confidence: f32,

    /// Evidence supporting this detection
    pub evidence: Vec<Evidence>,
}

/// Test categories for classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TestCategory {
    /// Unit tests
    Unit,
    /// Integration tests
    Integration,
    /// End-to-end tests
    E2e,
    /// Performance/load tests
    Performance,
    /// Security tests
    Security,
    /// Mixed/general testing framework
    Mixed,
}

/// Evidence supporting a detection.
///
/// Provides explainability for why something was detected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    /// Type of evidence
    pub evidence_type: EvidenceType,

    /// File path relative to project root
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,

    /// Pattern that matched (glob or regex)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// Matched content excerpt (max ~200 chars)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_content: Option<String>,

    /// Line number if applicable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,
}

/// Types of evidence for detections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// File exists at expected path
    FileExists,
    /// File name matches a pattern
    FilePattern,
    /// Content within file matches a pattern
    ContentMatch,
    /// Configuration key found
    ConfigKey,
    /// Listed as dependency in manifest
    Dependency,
    /// Import/require statement found
    Import,
    /// File extension indicates language
    Extension,
}

/// File statistics providing context for the analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileStats {
    /// Total number of files analyzed
    pub total_files: usize,

    /// File count by extension (e.g., {"rs": 42, "toml": 3})
    pub by_extension: HashMap<String, usize>,

    /// Number of directories traversed
    pub directories: usize,

    /// Number of files excluded (node_modules, target, etc.)
    pub excluded_files: usize,
}

/// Executable commands for common project operations.
///
/// These commands are detected from package.json scripts, Makefile, Cargo.toml,
/// or other project configuration files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Commands {
    /// Command to start the application (e.g., "cargo run", "npm start")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,

    /// Command to start in development mode (e.g., "npm run dev")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dev: Option<String>,

    /// Command to run tests (e.g., "cargo test", "npm test")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,

    /// Command to build for production (e.g., "cargo build --release")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,

    /// Command to run linter (e.g., "cargo clippy", "npm run lint")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lint: Option<String>,

    /// Command to format code (e.g., "cargo fmt", "npm run fmt")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fmt: Option<String>,

    /// Command to run type checker (e.g., "tsc --noEmit")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typecheck: Option<String>,
}

/// Purpose categories for entry points.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntryPointPurpose {
    /// Main binary entry point (e.g., src/main.rs, index.js)
    BinaryEntry,
    /// Library entry point (e.g., src/lib.rs, lib/index.js)
    LibraryEntry,
    /// Test entry point (e.g., tests/main.rs)
    TestEntry,
    /// Configuration file (e.g., config/default.toml)
    Config,
    /// Route definitions (e.g., src/routes.rs, routes/index.js)
    Routes,
    /// Main UI component (e.g., src/App.tsx)
    MainComponent,
}

/// A key entry point into the codebase.
///
/// Entry points help AI agents understand where to start when exploring
/// or modifying specific aspects of the project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntryPoint {
    /// Relative path from project root
    pub file: String,

    /// Purpose of this entry point
    pub purpose: EntryPointPurpose,
}

/// An environment variable used by the project.
///
/// Detected from .env.example, docker-compose.yml, config files, or code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvVar {
    /// Environment variable name (e.g., "DATABASE_URL")
    pub name: String,

    /// Whether this variable is required for the app to run
    pub required: bool,

    /// What this variable is used for
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,

    /// Default value if not set
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Example value for documentation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

/// Project analyzer for detecting project attributes.
///
/// Currently provides stub implementations. Future versions will implement
/// actual file scanning and pattern matching.
pub struct ProjectAnalyzer;

impl ProjectAnalyzer {
    /// Create a new project analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyze a project and return structured analysis.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Path to the project root directory
    ///
    /// # Returns
    ///
    /// A `ProjectAnalysis` struct with all detected attributes.
    ///
    /// # Note
    ///
    /// This is currently a stub that returns a minimal placeholder.
    /// Actual detection logic will be implemented in future versions.
    #[allow(unused_variables)]
    pub fn analyze(project_path: &Path) -> anyhow::Result<ProjectAnalysis> {
        // TODO: Implement actual detection logic
        // For now, return a minimal placeholder
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(ProjectAnalysis {
            project_name,
            project_path: project_path.to_string_lossy().to_string(),
            analyzed_at: chrono::Utc::now().to_rfc3339(),
            kind_assessment: KindAssessment {
                primary_kind: "experiment-sandbox".to_string(),
                confidence: 0.1,
                tier: "noncurrent".to_string(),
                matching_files: vec![],
                alternatives: vec![],
            },
            languages: vec![],
            frameworks: vec![],
            databases: vec![],
            docker: DockerDetection {
                has_dockerfile: false,
                has_compose: false,
                base_images: vec![],
                compose_services: vec![],
                evidence: vec![],
            },
            ports: vec![],
            testing: vec![],
            file_stats: FileStats {
                total_files: 0,
                by_extension: HashMap::new(),
                directories: 0,
                excluded_files: 0,
            },
            commands: Commands::default(),
            entry_points: vec![],
            environment: vec![],
        })
    }
}

impl Default for ProjectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_analysis_serializes_to_json() {
        let analysis = ProjectAnalysis {
            project_name: "test-project".to_string(),
            project_path: "/path/to/test-project".to_string(),
            analyzed_at: "2024-12-25T00:00:00Z".to_string(),
            kind_assessment: KindAssessment {
                primary_kind: "microservice".to_string(),
                confidence: 0.85,
                tier: "engines".to_string(),
                matching_files: vec!["src/main.rs".to_string(), "Dockerfile".to_string()],
                alternatives: vec![KindCandidate {
                    kind: "cli-devtool".to_string(),
                    confidence: 0.4,
                    match_count: 2,
                }],
            },
            languages: vec![LanguageDetection {
                language: "rust".to_string(),
                display_name: "Rust".to_string(),
                confidence: 0.95,
                is_primary: true,
                file_count: 42,
                evidence: vec![Evidence {
                    evidence_type: EvidenceType::FileExists,
                    file_path: Some("Cargo.toml".to_string()),
                    pattern: None,
                    matched_content: None,
                    line_number: None,
                }],
            }],
            frameworks: vec![FrameworkDetection {
                framework: "axum".to_string(),
                display_name: "Axum".to_string(),
                category: FrameworkCategory::Web,
                confidence: 0.9,
                version: Some("0.7".to_string()),
                evidence: vec![Evidence {
                    evidence_type: EvidenceType::Dependency,
                    file_path: Some("Cargo.toml".to_string()),
                    pattern: Some(r#"axum = "#.to_string()),
                    matched_content: Some(r#"axum = "0.7""#.to_string()),
                    line_number: Some(15),
                }],
            }],
            databases: vec![DatabaseDetection {
                database: "postgres".to_string(),
                display_name: "PostgreSQL".to_string(),
                category: DatabaseCategory::Relational,
                confidence: 0.85,
                port: Some(5432),
                evidence: vec![Evidence {
                    evidence_type: EvidenceType::ConfigKey,
                    file_path: Some(".env.example".to_string()),
                    pattern: Some("DATABASE_URL".to_string()),
                    matched_content: Some(
                        "DATABASE_URL=postgres://localhost:5432/mydb".to_string(),
                    ),
                    line_number: Some(1),
                }],
            }],
            docker: DockerDetection {
                has_dockerfile: true,
                has_compose: true,
                base_images: vec![DockerImage {
                    image: "rust".to_string(),
                    tag: Some("1.75-alpine".to_string()),
                    stage: Some("builder".to_string()),
                }],
                compose_services: vec!["app".to_string(), "db".to_string()],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::FileExists,
                        file_path: Some("Dockerfile".to_string()),
                        pattern: None,
                        matched_content: None,
                        line_number: None,
                    },
                    Evidence {
                        evidence_type: EvidenceType::FileExists,
                        file_path: Some("docker-compose.yml".to_string()),
                        pattern: None,
                        matched_content: None,
                        line_number: None,
                    },
                ],
            },
            ports: vec![PortDetection {
                port_type: PortType::Http,
                port_number: Some(8080),
                env_var: Some("PORT".to_string()),
                confidence: 0.8,
                evidence: vec![Evidence {
                    evidence_type: EvidenceType::ConfigKey,
                    file_path: Some("config/default.toml".to_string()),
                    pattern: Some("port".to_string()),
                    matched_content: Some("port = 8080".to_string()),
                    line_number: Some(5),
                }],
            }],
            testing: vec![TestFrameworkDetection {
                framework: "cargo_test".to_string(),
                display_name: "Cargo Test".to_string(),
                category: TestCategory::Mixed,
                confidence: 0.95,
                evidence: vec![Evidence {
                    evidence_type: EvidenceType::FilePattern,
                    file_path: Some("tests/".to_string()),
                    pattern: Some("tests/**/*.rs".to_string()),
                    matched_content: None,
                    line_number: None,
                }],
            }],
            file_stats: FileStats {
                total_files: 150,
                by_extension: HashMap::from([
                    ("rs".to_string(), 42),
                    ("toml".to_string(), 5),
                    ("md".to_string(), 3),
                ]),
                directories: 20,
                excluded_files: 1200,
            },
            commands: Commands {
                start: Some("cargo run".to_string()),
                dev: None,
                test: Some("cargo test".to_string()),
                build: Some("cargo build --release".to_string()),
                lint: Some("cargo clippy -- -D warnings".to_string()),
                fmt: Some("cargo fmt".to_string()),
                typecheck: None,
            },
            entry_points: vec![
                EntryPoint {
                    file: "src/main.rs".to_string(),
                    purpose: EntryPointPurpose::BinaryEntry,
                },
                EntryPoint {
                    file: "src/lib.rs".to_string(),
                    purpose: EntryPointPurpose::LibraryEntry,
                },
            ],
            environment: vec![EnvVar {
                name: "DATABASE_URL".to_string(),
                required: true,
                purpose: Some("PostgreSQL connection string".to_string()),
                default: None,
                example: Some("postgres://localhost:5432/mydb".to_string()),
            }],
        };

        let json = serde_json::to_string_pretty(&analysis);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("\"project_name\": \"test-project\""));
        assert!(json_str.contains("\"primary_kind\": \"microservice\""));
        assert!(json_str.contains("\"language\": \"rust\""));
        assert!(json_str.contains("\"framework\": \"axum\""));
        assert!(json_str.contains("\"database\": \"postgres\""));
        assert!(json_str.contains("\"has_dockerfile\": true"));
        assert!(json_str.contains("\"start\": \"cargo run\""));
        assert!(json_str.contains("\"binary_entry\""));
        assert!(json_str.contains("\"DATABASE_URL\""));
    }

    #[test]
    fn test_project_analysis_deserializes_from_json() {
        let json = r#"{
            "project_name": "my-app",
            "project_path": "/home/user/my-app",
            "analyzed_at": "2024-12-25T12:00:00Z",
            "kind_assessment": {
                "primary_kind": "ui-frontend",
                "confidence": 0.9,
                "tier": "engines",
                "matching_files": ["package.json", "src/App.tsx"]
            },
            "languages": [{
                "language": "typescript",
                "display_name": "TypeScript",
                "confidence": 0.95,
                "is_primary": true,
                "file_count": 100,
                "evidence": []
            }],
            "frameworks": [{
                "framework": "react",
                "display_name": "React",
                "category": "ui",
                "confidence": 0.95,
                "evidence": []
            }],
            "databases": [],
            "docker": {
                "has_dockerfile": false,
                "has_compose": false,
                "base_images": [],
                "compose_services": [],
                "evidence": []
            },
            "ports": [{
                "port_type": "http",
                "port_number": 3000,
                "confidence": 0.8,
                "evidence": []
            }],
            "testing": [{
                "framework": "jest",
                "display_name": "Jest",
                "category": "unit",
                "confidence": 0.9,
                "evidence": []
            }],
            "file_stats": {
                "total_files": 200,
                "by_extension": {"ts": 80, "tsx": 100, "json": 20},
                "directories": 30,
                "excluded_files": 5000
            },
            "commands": {
                "start": "npm start",
                "dev": "npm run dev",
                "test": "npm test",
                "build": "npm run build"
            },
            "entry_points": [
                {"file": "src/index.tsx", "purpose": "binary_entry"},
                {"file": "src/App.tsx", "purpose": "main_component"}
            ],
            "environment": [
                {"name": "REACT_APP_API_URL", "required": true, "example": "https://api.example.com"}
            ]
        }"#;

        let analysis: Result<ProjectAnalysis, _> = serde_json::from_str(json);
        assert!(analysis.is_ok(), "Should deserialize from JSON");

        let analysis = analysis.unwrap();
        assert_eq!(analysis.project_name, "my-app");
        assert_eq!(analysis.kind_assessment.primary_kind, "ui-frontend");
        assert_eq!(analysis.languages.len(), 1);
        assert_eq!(analysis.languages[0].language, "typescript");
        assert_eq!(analysis.frameworks.len(), 1);
        assert_eq!(analysis.frameworks[0].category, FrameworkCategory::Ui);
        assert_eq!(analysis.ports[0].port_number, Some(3000));
        assert_eq!(analysis.commands.start, Some("npm start".to_string()));
        assert_eq!(analysis.commands.dev, Some("npm run dev".to_string()));
        assert_eq!(analysis.entry_points.len(), 2);
        assert_eq!(
            analysis.entry_points[0].purpose,
            EntryPointPurpose::BinaryEntry
        );
        assert_eq!(analysis.environment.len(), 1);
        assert_eq!(analysis.environment[0].name, "REACT_APP_API_URL");
        assert!(analysis.environment[0].required);
    }

    #[test]
    fn test_framework_category_serializes_as_snake_case() {
        let detection = FrameworkDetection {
            framework: "tokio".to_string(),
            display_name: "Tokio".to_string(),
            category: FrameworkCategory::Async,
            confidence: 0.9,
            version: None,
            evidence: vec![],
        };

        let json = serde_json::to_string(&detection).unwrap();
        assert!(json.contains("\"category\":\"async\""));
    }

    #[test]
    fn test_database_category_serializes_as_snake_case() {
        let detection = DatabaseDetection {
            database: "rabbitmq".to_string(),
            display_name: "RabbitMQ".to_string(),
            category: DatabaseCategory::MessageQueue,
            confidence: 0.85,
            port: Some(5672),
            evidence: vec![],
        };

        let json = serde_json::to_string(&detection).unwrap();
        assert!(json.contains("\"category\":\"message_queue\""));
    }

    #[test]
    fn test_port_type_serializes_as_snake_case() {
        let detection = PortDetection {
            port_type: PortType::Rabbitmq,
            port_number: Some(5672),
            env_var: None,
            confidence: 0.8,
            evidence: vec![],
        };

        let json = serde_json::to_string(&detection).unwrap();
        assert!(json.contains("\"port_type\":\"rabbitmq\""));
    }

    #[test]
    fn test_evidence_type_serializes_as_snake_case() {
        let evidence = Evidence {
            evidence_type: EvidenceType::ContentMatch,
            file_path: Some("src/main.rs".to_string()),
            pattern: Some("async fn main".to_string()),
            matched_content: Some("async fn main() {".to_string()),
            line_number: Some(1),
        };

        let json = serde_json::to_string(&evidence).unwrap();
        assert!(json.contains("\"evidence_type\":\"content_match\""));
    }

    #[test]
    fn test_optional_fields_omitted_when_none() {
        let detection = FrameworkDetection {
            framework: "serde".to_string(),
            display_name: "Serde".to_string(),
            category: FrameworkCategory::Serialization,
            confidence: 0.95,
            version: None, // Should be omitted
            evidence: vec![],
        };

        let json = serde_json::to_string(&detection).unwrap();
        assert!(!json.contains("version"));
    }

    #[test]
    fn test_project_analyzer_stub_returns_placeholder() {
        use std::path::PathBuf;

        let path = PathBuf::from("/tmp/test-project");
        let result = ProjectAnalyzer::analyze(&path);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.project_name, "test-project");
        assert_eq!(analysis.kind_assessment.primary_kind, "experiment-sandbox");
        assert_eq!(analysis.kind_assessment.tier, "noncurrent");
        assert!(analysis.kind_assessment.confidence < 0.2);
    }

    #[test]
    fn test_file_stats_serialization() {
        let stats = FileStats {
            total_files: 100,
            by_extension: HashMap::from([("rs".to_string(), 50), ("toml".to_string(), 10)]),
            directories: 15,
            excluded_files: 500,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_files\":100"));
        assert!(json.contains("\"directories\":15"));

        let parsed: FileStats = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total_files, 100);
        assert_eq!(parsed.by_extension.get("rs"), Some(&50));
    }
}
