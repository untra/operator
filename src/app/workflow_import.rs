use std::io::{self, Write};

use anyhow::{ensure, Context, Result};

use super::{App, AppTerminal};
use crate::campaigns::batch::{self, BatchRequest};
use crate::campaigns::board::{BoardScope, BoardSelection};
use crate::campaigns::recipe::{self, Record, WorkflowDataset, WorkflowRecipe};
use crate::config::Config;
use crate::issuetypes::IssueTypeRegistry;

fn ask(label: &str) -> Result<String> {
    print!("{label} ");
    io::stdout().flush()?;
    let mut text = String::new();
    ensure!(io::stdin().read_line(&mut text)? > 0, "Input closed");
    let value = text.trim();
    ensure!(value != "/cancel", "Cancelled");
    Ok(value.to_owned())
}

fn pick(options: &[String], title: &str) -> Result<usize> {
    ensure!(!options.is_empty(), "No {title} available");
    let mut filter = String::new();
    loop {
        for (index, label) in options
            .iter()
            .enumerate()
            .filter(|(_, label)| label.to_lowercase().contains(&filter))
        {
            println!("{}  {label}", index + 1);
        }
        let choice = ask(&format!("{title} / enter a number or search"))?;
        if let Ok(number) = choice.parse::<usize>() {
            if (1..=options.len()).contains(&number) {
                return Ok(number - 1);
            }
        }
        filter = choice.to_lowercase();
    }
}

fn compose(
    config: &Config,
    registry: &IssueTypeRegistry,
    selected: Option<(&str, &str)>,
) -> Result<String> {
    println!("Create tickets / type /cancel to return");
    let saved = recipe::saved(config)?;
    let mut choices: Vec<_> = saved
        .keys()
        .map(|name| format!("Recipe / {name}"))
        .collect();
    let workflows: Vec<_> = registry
        .all_collections()
        .flat_map(|collection| {
            collection.types.iter().filter_map(|key| {
                registry
                    .get_in(&collection.name, key)
                    .map(|definition| (&collection.name, definition))
            })
        })
        .collect();
    choices.extend(
        workflows
            .iter()
            .map(|(owner, definition)| format!("{} / {owner}/{}", definition.name, definition.key)),
    );
    let index = if let Some((collection, key)) = selected {
        saved.len()
            + workflows
                .iter()
                .position(|(owner, definition)| {
                    owner.as_str() == collection && definition.key == key
                })
                .context("Workflow is unavailable")?
    } else {
        pick(&choices, "Workflow")?
    };
    let mut recipe = if let Some((_, saved)) = saved.iter().nth(index) {
        saved.clone()
    } else {
        let (owner, definition) = workflows[index - saved.len()];
        WorkflowRecipe {
            source: String::new(),
            unresolved_labels: None,
            include_legacy: None,
            collection: owner.clone(),
            workflow: definition.key.clone(),
            key_columns: Vec::new(),
            columns: Record::new(),
            shared: Record::new(),
            repeat: std::collections::BTreeMap::new(),
        }
    };
    let definition = registry
        .get_in(&recipe.collection, &recipe.workflow)
        .context("Workflow no longer exists")?;
    let source = ask("Enter a CSV or JSON file path, type /board for provider selection, or press Enter for one ticket")?;
    let mut board_data = None;
    let rows = if source == "/board" {
        let scopes = BoardScope::configured(config);
        let options = scopes
            .iter()
            .map(|b| format!("{} / {} / {}", b.provider, b.instance, b.project))
            .collect::<Vec<_>>();
        let scope = &scopes[pick(&options, "Board")?];
        let issues = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(scope.issues(config))
        })?;
        for issue in &issues {
            println!("{} / {}", issue.key, issue.summary);
        }
        let selection = ask("Issue keys separated by commas (Enter for all)")?;
        let wanted: std::collections::BTreeSet<_> = selection.split(',').map(str::trim).collect();
        let settings = scope.config(config).context("Board no longer configured")?;
        let rows: Vec<_> = issues
            .iter()
            .filter(|issue| selection.is_empty() || wanted.contains(issue.key.as_str()))
            .map(|issue| crate::campaigns::board::record(issue, &settings.type_mappings))
            .collect();
        ensure!(!rows.is_empty(), "No matching issues");
        board_data = Some((scope.clone(), issues));
        rows
    } else if source.is_empty() {
        vec![Record::new()]
    } else {
        recipe::parse_records(&std::fs::read_to_string(&source)?)?
    };
    if index >= saved.len() {
        let headers = rows
            .iter()
            .flat_map(Record::keys)
            .collect::<std::collections::BTreeSet<_>>();
        if !source.is_empty() && board_data.is_none() {
            println!(
                "Columns / {}",
                headers
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            recipe.source = ask("Dataset name")?;
            recipe.key_columns = ask("Key columns, comma-separated (Enter for one-time use)")?
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
        let mut fields: Vec<_> = definition
            .fields
            .iter()
            .filter(|f| f.user_editable)
            .map(|f| f.name.as_str())
            .collect();
        if definition.project_required && !fields.contains(&"project") {
            fields.push("project");
        }
        for field in fields {
            let value=ask(&format!("{field} / enter a shared value, @column, or repeat:one|two (Enter for matching column or default)"))?;
            if let Some(column) = value.strip_prefix('@') {
                recipe.columns.insert(field.to_owned(), column.to_owned());
            } else if let Some(list) = value.strip_prefix("repeat:") {
                recipe.repeat.insert(
                    field.to_owned(),
                    list.split('|').map(String::from).collect(),
                );
            } else if !value.is_empty() {
                recipe.shared.insert(field.to_owned(), value);
            } else if headers.iter().any(|name| name.as_str() == field) {
                recipe.columns.insert(field.to_owned(), field.to_owned());
            }
        }
        if !recipe.key_columns.is_empty() {
            let name = ask("Save recipe as (Enter to skip)")?;
            if !name.is_empty() {
                recipe::save(config, &name, &recipe)?;
            }
        }
    }
    let mut board_selection = None;
    let mut request = if let Some((scope, issues)) = board_data {
        println!("Use the selected workflow for all issues, or use mapped labels.");
        let policy =
            ask("For label matching type review, skip, or default (Enter for selected workflow)")?;
        recipe.unresolved_labels = match policy.as_str() {
            "review" => Some(recipe::LabelPolicy::Review),
            "skip" => Some(recipe::LabelPolicy::Skip),
            "default" => Some(recipe::LabelPolicy::UseSelected),
            "" => None,
            _ => anyhow::bail!("Unknown label policy"),
        };
        recipe.include_legacy = Some(
            ask("Include new variants of previously imported issues? [y/N]")?
                .eq_ignore_ascii_case("y"),
        );
        let selection = BoardSelection {
            scope,
            recipe,
            issue_ids: rows.iter().filter_map(|r| r.get("id").cloned()).collect(),
            status_writers: std::collections::BTreeMap::new(),
        };
        let tickets = selection.expand(
            &issues,
            &selection
                .scope
                .config(config)
                .context("Board no longer configured")?
                .type_mappings,
        )?;
        board_selection = Some(selection);
        BatchRequest {
            tickets,
            ..BatchRequest::default()
        }
    } else {
        BatchRequest {
            dataset: Some(WorkflowDataset { recipe, rows }),
            ..BatchRequest::default()
        }
        .expanded()?
    };
    let tickets = batch::all_tickets(config)?;
    let mut preview = batch::preview_with_registry(config, registry, &tickets, &request, false);
    println!(
        "{} planned / {} previously created",
        preview.tickets.len(),
        preview.previously_created.len()
    );
    for row in &preview.tickets {
        for message in &row.errors {
            println!("{} / {message}", row.reference);
        }
    }
    for message in &preview.errors {
        println!("{message}");
    }
    if let Some(first) = preview.tickets.first() {
        println!("{}", first.markdown);
    }
    if let Some(selection) = &mut board_selection {
        if selection
            .scope
            .config(config)
            .is_some_and(|settings| settings.bidirectional)
        {
            let owners =
                ask("Ticket refs to own upstream status, comma-separated (Enter for local-only)")?;
            for reference in owners
                .split(',')
                .map(str::trim)
                .filter(|reference| !reference.is_empty())
            {
                let source = request
                    .tickets
                    .iter_mut()
                    .find(|item| item.reference == reference)
                    .and_then(|item| item.source_issue.as_mut())
                    .context("Unknown source ticket ref")?;
                ensure!(
                    selection
                        .status_writers
                        .insert(source.id.clone(), reference.to_owned())
                        .is_none(),
                    "Choose only one status writer per issue"
                );
                source.status_writer = true;
            }
            preview = batch::preview_with_registry(config, registry, &tickets, &request, false);
            for error in &preview.errors {
                println!("{error}");
            }
        }
    }
    if let Some(selection) = &board_selection {
        if preview.valid && ask("Save mapping for future syncs? [y/N]")?.eq_ignore_ascii_case("y") {
            selection.scope.save_mapping(config, &selection.recipe)?;
        }
    }
    loop {
        let action = ask("Inspect row number, type create to confirm, or Enter to return")?;
        if action.is_empty() {
            return Ok("Returned to queue".into());
        }
        if action == "create" {
            ensure!(
                preview.valid,
                "Resolve preview errors before creating tickets"
            );
            if let Some(selection) = &board_selection {
                let issues = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(selection.scope.issues(config))
                })?;
                request.tickets = selection.expand(
                    &issues,
                    &selection
                        .scope
                        .config(config)
                        .context("Board no longer configured")?
                        .type_mappings,
                )?;
            }
            request.confirmation = Some(preview.confirmation);
            let created = batch::create_with_registry(config, registry, &request)?;
            return Ok(format!("Created {} tickets", created.tickets.len()));
        }
        if let Some(row) = action
            .parse::<usize>()
            .ok()
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| preview.tickets.get(i))
        {
            println!("{}", row.markdown);
        }
    }
}

fn author(config: &Config, registry: &mut IssueTypeRegistry, editor: &str) -> Result<String> {
    use crate::rest::routes::issuetypes::{persist_definition, render_sample, WorkflowSample};
    use serde_json::json;
    println!("New workflow / type /cancel to return");
    let name = ask("Workflow name")?;
    let suggested: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .take(16)
        .collect();
    let key = ask(&format!("Key (Enter for {suggested})"))?;
    let outcome = ask("Expected outcome")?;
    let mut fields = vec![json!({"name":"summary","description":"Ticket summary","required":true})];
    let mut values = std::collections::HashMap::new();
    values.insert("summary".into(), ask("Sample summary")?);
    loop {
        let field = ask("New input name (Enter to finish inputs)")?;
        if field.is_empty() {
            break;
        }
        fields.push(json!({"name":field,"description":field,"required":true}));
        values.insert(field.clone(), ask(&format!("Sample {field}"))?);
    }
    let mut steps = Vec::new();
    loop {
        let step = ask("Step name (Enter to finish steps)")?;
        if step.is_empty() {
            break;
        }
        let file = tempfile::Builder::new().suffix(".md").tempfile()?;
        let (program, args) = crate::editors::EditorConfig::split_command(editor);
        ensure!(
            std::process::Command::new(program)
                .args(args)
                .arg(file.path())
                .status()?
                .success(),
            "Editor exited before saving"
        );
        let prompt = std::fs::read_to_string(file.path())?;
        ensure!(!prompt.trim().is_empty(), "Step instructions are empty");
        if let Some(previous) = steps.last_mut() {
            let previous: &mut serde_json::Value = previous;
            previous["next_step"] = json!(step);
        }
        steps.push(json!({"name":step,"prompt":prompt,"outputs":[]}));
    }
    let project_required = ask("Require a local project? [y/N]")?.eq_ignore_ascii_case("y");
    if project_required {
        values.insert("project".into(), ask("Sample project")?);
    }
    let collection = registry.active_collection_name().to_owned();
    let definition = json!({"key":if key.is_empty(){suggested}else{key},"name":name,"description":outcome,"glyph":">","fields":fields,"steps":steps,"collection":collection,"project_required":project_required});
    let sample = WorkflowSample {
        definition: serde_json::from_value(definition.clone())?,
        values,
    };
    println!(
        "{}",
        render_sample(config, sample).map_err(|error| anyhow::anyhow!("{error:?}"))?
    );
    if ask("Type save to register the workflow")? != "save" {
        return Ok("Draft discarded".into());
    }
    let saved = persist_definition(
        registry,
        &config.tickets_path().join("templates"),
        serde_json::from_value(definition)?,
    )
    .map_err(|error| anyhow::anyhow!("{error:?}"))?;
    if ask("Create tickets with this workflow? [y/N]")?.eq_ignore_ascii_case("y") {
        return compose(config, registry, Some((&collection, &saved.key)));
    }
    Ok(format!("Saved workflow {collection}/{}", saved.key))
}

impl App {
    pub(super) async fn new_workflow(&mut self, terminal: &mut AppTerminal) -> Result<()> {
        let outcome = crate::ui::with_suspended_tui(terminal, || {
            author(
                &self.config,
                &mut self.issue_type_registry,
                self.dashboard.editor_config.file_editor(),
            )
        });
        if outcome.is_ok() {
            if let Some(state) = self.rest_api_server.api_state() {
                *state.registry.write().await = crate::startup::templates::load_active_registry(
                    &self.config.tickets_path(),
                    self.config.templates.active_collection.as_deref(),
                );
            }
        }
        self.dashboard
            .set_status(&outcome.unwrap_or_else(|e| e.to_string()));
        Ok(())
    }

    pub(super) fn import_records(&mut self, terminal: &mut AppTerminal) -> Result<()> {
        let registry = crate::startup::templates::load_active_registry(
            &self.config.tickets_path(),
            self.config.templates.active_collection.as_deref(),
        );
        let outcome =
            crate::ui::with_suspended_tui(terminal, || compose(&self.config, &registry, None));
        self.dashboard
            .set_status(&outcome.unwrap_or_else(|e| e.to_string()));
        self.refresh_data()
    }
}
