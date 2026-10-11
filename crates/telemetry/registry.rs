use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

type Result<T> = std::result::Result<T, String>;

/// Identifies the Operator telemetry registry and its upstream semantic-convention dependencies.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Names the registry that owns these telemetry definitions.
    pub name: String,
    /// Specifies the version of this registry's semantic conventions.
    pub semconv_version: String,
    /// Provides the base URL for this registry's versioned telemetry schemas.
    pub schema_base_url: String,
    /// Summarizes the registry's purpose for readers and documentation tools.
    pub description: String,
    /// Lists the upstream registries available to Weaver during dependency resolution.
    pub dependencies: Vec<Dependency>,
}

/// Identifies an upstream semantic-convention registry pinned for Weaver validation.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    /// Names the upstream registry within the dependency manifest.
    pub name: String,
    /// Locates the upstream registry using a repository URL, version reference, and model subdirectory.
    pub registry_path: String,
}

/// Contains the semantic-convention groups declared in one model YAML file.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    /// Lists the attribute groups and telemetry signals defined by this model file.
    pub groups: Vec<Group>,
}

/// Describes an attribute group, event, span, or metric and its associated attributes.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// Uniquely identifies the semantic-convention group and supplies the signal name for spans.
    pub id: String,
    /// Selects the attribute-group, event, span, or metric definition format from the YAML `type` field.
    #[serde(rename = "type")]
    pub kind: String,
    /// Describes the group's purpose in generated catalog metadata and documentation.
    pub brief: String,
    /// Declares the convention's maturity, with development stability required for signal definitions.
    pub stability: Option<String>,
    /// Specifies the emitted event name for an event group.
    pub name: Option<String>,
    /// Specifies the emitted metric name for a metric group.
    pub metric_name: Option<String>,
    /// Selects the metric instrument, such as a counter, gauge, or histogram.
    pub instrument: Option<String>,
    /// Specifies the measurement unit recorded by a metric.
    pub unit: Option<String>,
    /// Identifies a span's role, such as internal work or a server operation.
    pub span_kind: Option<String>,
    /// Lists attribute definitions or references associated with this group.
    pub attributes: Vec<Attribute>,
}

/// Defines a classified telemetry attribute or references an existing definition without overrides.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    /// Defines the globally unique attribute key when this entry is not a reference.
    pub id: Option<String>,
    /// Identifies an existing attribute whose definition and classification this entry inherits.
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    /// Declares the attribute's value type for a definition rather than a reference.
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// Explains the attribute's meaning in generated catalog metadata and documentation.
    pub brief: Option<String>,
    /// Declares development stability for an attribute definition.
    pub stability: Option<String>,
    /// Provides illustrative values for an attribute definition in Weaver's YAML format.
    pub examples: Option<serde_yaml::Value>,
    /// Supplies the required Operator redaction classification for an attribute definition.
    pub annotations: Option<Annotations>,
}

/// Holds the Operator-specific annotations attached to a telemetry attribute definition.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annotations {
    /// Contains the attribute's privacy classification under the Operator annotation namespace.
    pub operator: Classification,
}

/// Assigns an attribute to one of the registry's privacy classifications.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Classification {
    /// Names the public, internal, personal, content, or secret classification governing redaction.
    pub redaction_class: String,
}

/// Defines the redaction classifications and their fixed actions.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Classes {
    /// Lists the five classifications and their actions in each redaction mode.
    pub classes: Vec<Class>,
}

/// Defines how one classification behaves with optional redaction enabled or disabled.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Class {
    /// Names the class referenced by attribute definitions and generated policy types.
    pub name: String,
    /// Selects whether to keep, drop, or hash a value when optional redaction is enabled.
    pub when_enabled: String,
    /// Selects the action when optional redaction is disabled, with secrets still dropped.
    pub when_disabled: String,
}

/// Contains the reviewed secret-detection patterns used by the redactor.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Patterns {
    /// Lists credential patterns in their configured matching order.
    pub patterns: Vec<Pattern>,
}

/// Describes a credential pattern for dropping attributes or scrubbing permitted bodies.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pattern {
    /// Uniquely names the credential pattern for validation diagnostics.
    pub name: String,
    /// Defines a Rust regular expression that matches secret text without matching empty strings.
    pub regex: String,
}

/// Combines registry metadata, model definitions, and redaction policies for validation and generation.
#[derive(Clone)]
pub struct Registry {
    /// Identifies this registry, its version, and its upstream dependencies.
    pub manifest: Manifest,
    /// Pairs each group with its registry-relative source path, sorted by group identifier when loaded.
    pub groups: Vec<(String, Group)>,
    /// Defines the classification actions used by generated redaction policies.
    pub classes: Classes,
    /// Supplies the secret-detection expressions embedded in the generated catalog.
    pub patterns: Patterns,
}

fn read_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_yaml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn model_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            files.extend(model_files(&path)?);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("yaml" | "yml")
        ) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

impl Registry {
    pub fn load(root: &Path) -> Result<Self> {
        let mut groups = Vec::new();
        for path in model_files(&root.join("model"))? {
            let model: Model = read_yaml(&path)?;
            let source = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            groups.extend(model.groups.into_iter().map(|g| (source.clone(), g)));
        }
        groups.sort_by(|a, b| a.1.id.cmp(&b.1.id));
        let registry = Self {
            manifest: read_yaml(&root.join("registry_manifest.yaml"))?,
            groups,
            classes: read_yaml(&root.join("redaction/classes.yaml"))?,
            patterns: read_yaml(&root.join("redaction/secret_patterns.yaml"))?,
        };
        registry.validate()?;
        Ok(registry)
    }

    pub fn attributes(&self) -> BTreeMap<&str, (&str, &Attribute)> {
        self.groups
            .iter()
            .flat_map(|(source, group)| {
                group
                    .attributes
                    .iter()
                    .filter_map(move |a| a.id.as_deref().map(|id| (id, (source.as_str(), a))))
            })
            .collect()
    }

    pub fn validate(&self) -> Result<()> {
        if self.manifest.name != "operator"
            || self.manifest.semconv_version.is_empty()
            || !self.manifest.schema_base_url.starts_with("https://")
            || self.manifest.description.is_empty()
            || self.manifest.dependencies.is_empty()
            || self
                .manifest
                .dependencies
                .iter()
                .any(|d| d.name.is_empty() || !d.registry_path.contains("@v"))
        {
            return Err("invalid or unpinned registry manifest".into());
        }
        let names: BTreeSet<_> = self
            .classes
            .classes
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        if names != BTreeSet::from(["public", "internal", "personal", "content", "secret"])
            || self.classes.classes.len() != names.len()
            || self.classes.classes.iter().any(|c| {
                !["keep", "drop", "hash"].contains(&c.when_enabled.as_str())
                    || !["keep", "drop", "hash"].contains(&c.when_disabled.as_str())
                    || (c.name == "secret"
                        && (c.when_enabled != "drop" || c.when_disabled != "drop"))
            })
        {
            return Err("invalid redaction classes or actions".into());
        }
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut symbols = BTreeSet::new();
        let mut attrs = BTreeSet::new();
        for (source, g) in &self.groups {
            if !valid_name(&g.id) || !ids.insert(&g.id) || g.brief.trim().is_empty() {
                return Err(format!("{source}: invalid or duplicate group {}", g.id));
            }
            if g.kind != "attribute_group" && g.stability.as_deref() != Some("development") {
                return Err(format!(
                    "{source}: missing development stability for {}",
                    g.id
                ));
            }
            match g.kind.as_str() {
                "attribute_group"
                    if g.name.is_none()
                        && g.metric_name.is_none()
                        && g.instrument.is_none()
                        && g.unit.is_none()
                        && g.span_kind.is_none() => {}
                "event"
                    if g.name.is_some()
                        && g.metric_name.is_none()
                        && g.instrument.is_none()
                        && g.unit.is_none()
                        && g.span_kind.is_none() => {}
                "span"
                    if g.name.is_none()
                        && g.metric_name.is_none()
                        && g.instrument.is_none()
                        && g.unit.is_none()
                        && matches!(
                            g.span_kind.as_deref(),
                            Some("internal" | "server" | "client" | "producer" | "consumer")
                        ) => {}
                "metric"
                    if g.name.is_none()
                        && g.metric_name.is_some()
                        && g.unit.as_ref().is_some_and(|u| !u.is_empty())
                        && g.span_kind.is_none()
                        && matches!(
                            g.instrument.as_deref(),
                            Some("counter" | "gauge" | "histogram" | "updowncounter")
                        ) => {}
                _ => return Err(format!("{source}: unsupported or invalid group {}", g.id)),
            }
            if g.kind != "attribute_group" {
                let name = g.signal_name();
                if !valid_name(name)
                    || !names.insert(name)
                    || !symbols.insert(format!("{}:{}", g.kind, symbol(name)))
                {
                    return Err(format!("{source}: duplicate or invalid signal {name}"));
                }
            }
            let mut references = BTreeSet::new();
            for a in &g.attributes {
                match (&a.id, &a.reference) {
                    (Some(id), None) => {
                        if !valid_name(id)
                            || !attrs.insert(id)
                            || !symbols.insert(format!("attr:{}", symbol(id)))
                        {
                            return Err(format!("{source}: duplicate or invalid attribute {id}"));
                        }
                        if !matches!(
                            a.kind.as_deref(),
                            Some(
                                "string"
                                    | "int"
                                    | "double"
                                    | "boolean"
                                    | "any"
                                    | "string[]"
                                    | "int[]"
                                    | "double[]"
                                    | "boolean[]"
                            )
                        ) || a.brief.as_ref().is_none_or(|b| b.trim().is_empty())
                            || a.stability.as_deref() != Some("development")
                            || a.examples.is_none()
                            || !self.classes.classes.iter().any(|c| c.name == a.class())
                        {
                            return Err(format!(
                                "{source}: missing or invalid definition/class for {id}"
                            ));
                        }
                    }
                    (None, Some(reference))
                        if a.kind.is_none()
                            && a.brief.is_none()
                            && a.stability.is_none()
                            && a.examples.is_none()
                            && a.annotations.is_none()
                            && references.insert(reference) => {}
                    _ => {
                        return Err(format!(
                            "{source}: invalid attribute or reference in {}",
                            g.id
                        ))
                    }
                }
            }
        }
        let definitions = self.attributes();
        for (source, g) in &self.groups {
            for a in &g.attributes {
                let definition = if let Some(reference) = &a.reference {
                    definitions
                        .get(reference.as_str())
                        .ok_or_else(|| format!("{source}: unresolved reference {reference}"))?
                        .1
                } else {
                    a
                };
                if g.kind == "event" && definition.class() == "secret" {
                    return Err(format!("{source}: secret attribute on event {}", g.id));
                }
            }
        }
        let mut patterns = BTreeSet::new();
        if self.patterns.patterns.is_empty() {
            return Err("secret patterns must not be empty".into());
        }
        for p in &self.patterns.patterns {
            let regex =
                regex::Regex::new(&p.regex).map_err(|e| format!("pattern {}: {e}", p.name))?;
            if p.name.is_empty() || !patterns.insert(&p.name) || regex.is_match("") {
                return Err(format!("invalid or duplicate secret pattern {}", p.name));
            }
        }
        Ok(())
    }

    pub fn rust(&self) -> String {
        let mut out = String::from("#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]\n#[serde(rename_all = \"lowercase\")]\n#[cfg_attr(feature = \"schema\", derive(schemars::JsonSchema, ts_rs::TS))]\npub enum RedactionClass {\n");
        for c in &self.classes.classes {
            writeln!(out, "{},", symbol(&c.name)).unwrap();
        }
        out.push_str("}\nimpl RedactionClass {\n/// Returns the fixed action for this classification and redaction mode.\npub const fn action(self, enable_redaction: bool) -> RedactionAction { match (self, enable_redaction) {\n");
        for c in &self.classes.classes {
            writeln!(
                out,
                "(Self::{}, true) => RedactionAction::{},",
                symbol(&c.name),
                symbol(&c.when_enabled)
            )
            .unwrap();
            writeln!(
                out,
                "(Self::{}, false) => RedactionAction::{},",
                symbol(&c.name),
                symbol(&c.when_disabled)
            )
            .unwrap();
        }
        out.push_str("}}}\npub mod attr {\n");
        let attrs = self.attributes();
        for key in attrs.keys() {
            writeln!(out, "pub const {}: &str = {key:?};", constant(key)).unwrap();
        }
        out.push_str("}\npub fn class_of(key: &str) -> RedactionClass { match key {\n");
        for (key, (_, a)) in &attrs {
            writeln!(out, "{key:?} => RedactionClass::{},", symbol(a.class())).unwrap();
        }
        out.push_str(
            "_ => RedactionClass::Content,\n}}\npub static ATTRIBUTES: &[AttributeSpec] = &[\n",
        );
        for (key, (source, a)) in &attrs {
            writeln!(out, "AttributeSpec {{ name: {key:?}, class: RedactionClass::{}, value_type: {:?}, description: {:?}, source: {source:?} }},", symbol(a.class()), a.kind.as_deref().unwrap(), a.brief.as_deref().unwrap()).unwrap();
        }
        out.push_str("];\n");
        for (kind, ty) in [
            ("event", "EventName"),
            ("span", "SpanName"),
            ("metric", "MetricName"),
        ] {
            let groups: Vec<_> = self.groups.iter().filter(|(_, g)| g.kind == kind).collect();
            writeln!(
                out,
                "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum {ty} {{"
            )
            .unwrap();
            for (_, g) in &groups {
                writeln!(out, "{},", symbol(g.signal_name())).unwrap();
            }
            writeln!(out, "}}\nimpl {ty} {{ pub const ALL: &'static [Self] = &[").unwrap();
            for (_, g) in &groups {
                writeln!(out, "Self::{},", symbol(g.signal_name())).unwrap();
            }
            out.push_str("];\npub const fn as_str(self) -> &'static str { match self {\n");
            for (_, g) in &groups {
                writeln!(
                    out,
                    "Self::{} => {:?},",
                    symbol(g.signal_name()),
                    g.signal_name()
                )
                .unwrap();
            }
            out.push_str("}}}\n");
        }
        out.push_str("pub static SIGNALS: &[SignalSpec] = &[\n");
        for (source, g) in self
            .groups
            .iter()
            .filter(|(_, g)| g.kind != "attribute_group")
        {
            let keys: Vec<_> = g
                .attributes
                .iter()
                .map(|a| a.id.as_ref().or(a.reference.as_ref()).unwrap())
                .collect();
            writeln!(out, "SignalSpec {{ name: {:?}, kind: {:?}, description: {:?}, attributes: &{keys:?}, source: {source:?}, unit: {:?} }},", g.signal_name(), g.kind, g.brief, g.unit).unwrap();
        }
        out.push_str("];\npub static SECRET_PATTERNS: &[&str] = &[\n");
        for p in &self.patterns.patterns {
            writeln!(out, "{:?},", p.regex).unwrap();
        }
        out.push_str("];\n");
        out
    }

    pub fn markdown(&self) -> String {
        let mut out = String::from("---\nlayout: doc\ntitle: OTLP Event Catalog\n---\n\n<!-- AUTO-GENERATED FROM telemetry/ - DO NOT EDIT MANUALLY -->\n<!-- Regenerate with: cargo run --bin generate_types -->\n\nThe catalog defines the classifications and approved signal names used by Operator OTLP ingestion.\n\n## Redaction policy\n\nTelemetry and optional redaction are independently disabled by default. Set `telemetry.enable_redaction = true` to hash personal attributes and drop content. Secret attributes and attributes containing detected credentials are dropped in both modes; retained free text is scrubbed. Unknown attributes are content, not a sixth class.\n\n| Class | Redaction disabled | Redaction enabled |\n| --- | --- | --- |\n");
        for c in &self.classes.classes {
            writeln!(
                out,
                "| {} | {} | {} |",
                c.name, c.when_disabled, c.when_enabled
            )
            .unwrap();
        }
        out.push_str("\nHashing uses installation-key HMAC-SHA256 as `hmac:<hex16>`. The policy runs once before queueing; all sinks receive that sanitized payload. Configuration changes require a restart.\n\nWith redaction enabled, unknown span and event names become `operator.redacted.span` and `operator.redacted.event`; unknown metrics are dropped. Absent log event names stay absent. Bodies and auxiliary text are content. Retained metrics use catalog descriptions and units. Attribute types must match the catalog.\n\nIn both modes, opaque non-UTF-8 bytes and unresolved string-table references are removed. Secret detection is pattern-based and cannot recognize every credential or establish that arbitrary metadata strings are safe.\n\n`operator.redacted.count` is server-authored: each removed attribute, body, nested body entry, auxiliary field, or metric counts once, each hash counts once, and each non-overlapping secret substitution counts once. A replaced signal name or metric description/unit counts once per field; a removed entity reference counts once. Removing a metric does not additionally count its fields. Client counts are discarded.\n\nSee [Telemetry](/telemetry/) for configuration and ingest access.\n\n## Attributes\n\n| Attribute | Type | Class | Description | Source |\n| --- | --- | --- | --- | --- |\n");
        for (key, (source, a)) in self.attributes() {
            writeln!(
                out,
                "| `{key}` | {} | {} | {} | `telemetry/{source}` |",
                a.kind.as_deref().unwrap(),
                a.class(),
                cell(a.brief.as_deref().unwrap())
            )
            .unwrap();
        }
        for (kind, title) in [
            ("event", "Events"),
            ("span", "Spans"),
            ("metric", "Metrics"),
        ] {
            writeln!(out, "\n## {title}\n").unwrap();
            for (source, g) in self.groups.iter().filter(|(_, g)| g.kind == kind) {
                writeln!(
                    out,
                    "### `{}`\n\n{}\n\nSource: `telemetry/{source}`.\n",
                    g.signal_name(),
                    g.brief
                )
                .unwrap();
                if let (Some(instrument), Some(unit)) = (&g.instrument, &g.unit) {
                    writeln!(out, "Instrument: `{instrument}`. Unit: `{unit}`.\n").unwrap();
                }
                if let Some(span_kind) = &g.span_kind {
                    writeln!(out, "Span kind: `{span_kind}`.\n").unwrap();
                }
                let keys: Vec<_> = g
                    .attributes
                    .iter()
                    .map(|a| format!("`{}`", a.id.as_ref().or(a.reference.as_ref()).unwrap()))
                    .collect();
                writeln!(out, "Attributes: {}.\n", keys.join(", ")).unwrap();
            }
        }
        out
    }
}

impl Attribute {
    fn class(&self) -> &str {
        self.annotations
            .as_ref()
            .map_or("", |a| a.operator.redaction_class.as_str())
    }
}

impl Group {
    fn signal_name(&self) -> &str {
        self.name
            .as_deref()
            .or(self.metric_name.as_deref())
            .unwrap_or(&self.id)
    }
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.split('.').all(|part| {
            !part.is_empty()
                && part.as_bytes()[0].is_ascii_lowercase()
                && part
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
        })
}

fn symbol(value: &str) -> String {
    value
        .split(['.', '_'])
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            chars.next().unwrap().to_ascii_uppercase().to_string() + chars.as_str()
        })
        .collect()
}

fn constant(value: &str) -> String {
    value.replace('.', "_").to_ascii_uppercase()
}
fn cell(value: &str) -> String {
    value.replace('|', "&#124;").replace('\n', " ")
}
