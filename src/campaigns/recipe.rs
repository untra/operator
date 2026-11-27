use std::collections::{BTreeMap, BTreeSet};

use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use super::batch::BatchItem;
use crate::collections::fetch::sha256_hex;

pub type Record = BTreeMap<String, String>;
pub(super) const MAX_EXPANDED_TICKETS: usize = 10_000;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum LabelPolicy {
    Review,
    Skip,
    UseSelected,
}

/// Reusable mapping from a named source to one collection-qualified workflow.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct WorkflowRecipe {
    pub source: String,
    /// Resolve board labels when set. Unmatched or ambiguous issues follow this policy.
    #[serde(default)]
    #[ts(optional)]
    pub unresolved_labels: Option<LabelPolicy>,
    /// Include new variants when adopting an issue already imported by legacy sync.
    #[serde(default)]
    #[ts(optional)]
    pub include_legacy: Option<bool>,
    pub collection: String,
    pub workflow: String,
    pub key_columns: Vec<String>,
    pub columns: Record,
    pub shared: Record,
    pub repeat: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct WorkflowDataset {
    pub recipe: WorkflowRecipe,
    pub rows: Vec<Record>,
}

impl WorkflowDataset {
    pub fn expand(&self) -> Result<Vec<BatchItem>> {
        let recipe = &self.recipe;
        ensure!(
            !recipe.collection.is_empty(),
            "Choose a workflow collection"
        );
        ensure!(!self.rows.is_empty(), "Supply at least one row");
        if !recipe.key_columns.is_empty() {
            ensure!(
                !recipe.source.trim().is_empty(),
                "Name the dataset for repeat imports"
            );
        }
        let mut fields = BTreeSet::new();
        for name in recipe
            .columns
            .keys()
            .chain(recipe.shared.keys())
            .chain(recipe.repeat.keys())
        {
            ensure!(
                fields.insert(name),
                "Input {name} has more than one mapping"
            );
        }
        let mut count = self.rows.len();
        for (field, values) in &recipe.repeat {
            ensure!(
                !values.is_empty() && values.iter().all(|s| !s.trim().is_empty()),
                "Supply repeat values for {field}"
            );
            ensure!(
                values.iter().collect::<BTreeSet<_>>().len() == values.len(),
                "Duplicate repeat value for {field}"
            );
            count = count
                .checked_mul(values.len())
                .context("Too many combinations")?;
        }
        ensure!(
            count <= MAX_EXPANDED_TICKETS,
            "A batch supports up to {MAX_EXPANDED_TICKETS} tickets"
        );
        let mut keys = BTreeSet::new();
        let mut items = Vec::with_capacity(count);
        for (index, row) in self.rows.iter().enumerate() {
            let number = index + 1;
            let mut key = Record::new();
            for name in &recipe.key_columns {
                let value = row
                    .get(name)
                    .filter(|v| !v.trim().is_empty())
                    .with_context(|| format!("Row {number} needs key column {name}"))?;
                key.insert(name.clone(), value.clone());
            }
            ensure!(
                key.is_empty() || keys.insert(key.clone()),
                "Duplicate source key in row {number}"
            );
            let mut mapped = recipe.shared.clone();
            for (field, column) in &recipe.columns {
                let value = row
                    .get(column)
                    .with_context(|| format!("Row {number} is missing mapped column {column}"))?;
                mapped.insert(field.clone(), value.clone());
            }
            let mut combinations = vec![mapped];
            for (field, choices) in &recipe.repeat {
                combinations = combinations
                    .into_iter()
                    .flat_map(|values| {
                        choices.iter().map(move |choice| {
                            let mut next = values.clone();
                            next.insert(field.clone(), choice.clone());
                            next
                        })
                    })
                    .collect();
            }
            for (variant, values) in combinations.into_iter().enumerate() {
                let identity = if key.is_empty() {
                    None
                } else {
                    Some(sha256_hex(&serde_json::to_vec(&(
                        &recipe.source,
                        &key,
                        &recipe.collection,
                        &recipe.workflow,
                        &values,
                    ))?))
                };
                items.push(BatchItem {
                    reference: format!("row-{number}-{}", variant + 1),
                    template: recipe.workflow.clone(),
                    collection: Some(recipe.collection.clone()),
                    record_identity: identity,
                    project: values.get("project").cloned(),
                    summary: values.get("summary").cloned(),
                    values: values.into_iter().collect(),
                    ..BatchItem::default()
                });
            }
        }
        Ok(items)
    }
}

pub fn saved(config: &crate::config::Config) -> Result<BTreeMap<String, WorkflowRecipe>> {
    let dir = config.state_path().join("workflow-recipes");
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error.into()),
    };
    let mut recipes = BTreeMap::new();
    for entry in entries {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let (name, recipe) = serde_json::from_slice(&std::fs::read(path)?)?;
            recipes.insert(name, recipe);
        }
    }
    Ok(recipes)
}

pub fn save(config: &crate::config::Config, name: &str, recipe: &WorkflowRecipe) -> Result<()> {
    ensure!(!name.trim().is_empty(), "Name the recipe");
    ensure!(!recipe.key_columns.is_empty(), "Choose a stable key column");
    ensure!(!recipe.source.trim().is_empty(), "Name the source");
    let dir = config.state_path().join("workflow-recipes");
    std::fs::create_dir_all(&dir)?;
    let temp = dir.join(uuid::Uuid::new_v4().to_string());
    std::fs::write(&temp, serde_json::to_vec_pretty(&(name, recipe))?)?;
    std::fs::rename(
        temp,
        dir.join(format!("{}.json", sha256_hex(name.as_bytes()))),
    )?;
    Ok(())
}

/// Decode pasted records or file contents into workflow input columns.
pub fn parse_records(input: &str) -> Result<Vec<Record>> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }
    if input.trim_start().starts_with('[') {
        let records: Vec<BTreeMap<String, serde_json::Value>> =
            serde_json::from_str(input).context("Expected a JSON array of records")?;
        ensure!(
            records.len() <= MAX_EXPANDED_TICKETS,
            "Too many source records"
        );
        return records
            .into_iter()
            .enumerate()
            .map(|(index, row)| {
                row.into_iter()
                    .map(|(name, value)| {
                        let text = match value {
                            serde_json::Value::Null => String::new(),
                            serde_json::Value::String(text) => text,
                            serde_json::Value::Number(_) | serde_json::Value::Bool(_) => {
                                value.to_string()
                            }
                            _ => anyhow::bail!(
                                "Row {}, column {name} needs a scalar value",
                                index + 1
                            ),
                        };
                        Ok((name, text))
                    })
                    .collect()
            })
            .collect();
    }
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::Headers)
        .from_reader(input.as_bytes());
    let headers = reader.headers()?.clone();
    let unique: BTreeSet<_> = headers.iter().collect();
    ensure!(
        unique.len() == headers.len() && !unique.contains(""),
        "CSV columns need distinct, nonempty names"
    );
    reader
        .records()
        .enumerate()
        .map(|(index, record)| {
            ensure!(index < MAX_EXPANDED_TICKETS, "Too many source records");
            let record = record.with_context(|| format!("Invalid CSV in row {}", index + 1))?;
            Ok(headers
                .iter()
                .zip(record.iter())
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_records_quoted_csv_matches_json() {
        let csv = "\u{feff}id,text\r\n1,\"line one, \"\"quoted\"\"\nline two\"\r\n";
        let json = r#"[{"id":1,"text":"line one, \"quoted\"\nline two"}]"#;
        let rows = parse_records(csv).unwrap();
        assert_eq!(rows[0]["text"], "line one, \"quoted\"\nline two");
        assert_eq!(rows, parse_records(json).unwrap());
    }

    #[test]
    fn test_parse_records_rejects_ambiguous_columns_and_nested_json() {
        for bad in ["id,id\n1,2", " ,id\nx,1", "id,x\n1,2,3", r#"[{"id":{}}]"#] {
            assert!(parse_records(bad).is_err(), "{bad}");
        }
    }
}
