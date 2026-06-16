use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod api;
mod app;
mod collections;
mod config;
mod editors;
mod git;
mod issuetypes;
mod llm;
mod logging;
mod permissions;
mod pr_config;
mod projects;
mod services;
mod state;
mod steps;
#[allow(dead_code)]
mod taxonomy;
mod templates;
mod types;

mod acp;
mod agents;
mod docs_gen;
pub mod env_vars;
mod mcp;
mod notifications;
mod queue;
mod relay;
mod rest;
mod setup;
mod startup;
mod ui;
mod version;
mod workflow_gen;

use agents::tmux::{SystemTmuxClient, TmuxClient, TmuxError};
use app::App;
use config::Config;
use templates::glyph_for_key;

/// Detect installed LLM tools in PATH
fn detect_llm_tools() -> Vec<String> {
    let tools = ["claude", "codex", "gemini"];
    tools
        .iter()
        .filter(|tool| which::which(tool).is_ok())
        .map(|s| (*s).to_string())
        .collect()
}

/// Check kanban environment variables
#[allow(dead_code)]
fn check_kanban_env_vars() -> (bool, bool) {
    let jira =
        std::env::var("OPERATOR_JIRA_API_KEY").is_ok() || std::env::var("JIRA_API_TOKEN").is_ok();
    let linear =
        std::env::var("OPERATOR_LINEAR_API_KEY").is_ok() || std::env::var("LINEAR_API_KEY").is_ok();
    (jira, linear)
}

/// Check if tmux is available and meets version requirements
fn check_tmux_available() -> Result<(), TmuxError> {
    let client = SystemTmuxClient::new();
    let version = client.check_available()?;

    // Minimum version 2.1 for the features we use
    const MIN_MAJOR: u32 = 2;
    const MIN_MINOR: u32 = 1;

    if !version.meets_minimum(MIN_MAJOR, MIN_MINOR) {
        return Err(TmuxError::VersionTooOld(
            version.raw,
            format!("{MIN_MAJOR}.{MIN_MINOR}"),
        ));
    }

    tracing::debug!(version = %version.raw, "tmux available");
    Ok(())
}

/// Print a helpful error message for tmux issues
fn print_tmux_error(err: &TmuxError) {
    eprintln!("Error: {err}");
    eprintln!();

    match err {
        TmuxError::NotInstalled => {
            eprintln!("tmux is required to run operator.");
            eprintln!();
            eprintln!("Install tmux:");
            eprintln!("  macOS:         brew install tmux");
            eprintln!("  Ubuntu/Debian: sudo apt install tmux");
            eprintln!("  Fedora/RHEL:   sudo dnf install tmux");
            eprintln!("  Arch:          sudo pacman -S tmux");
        }
        TmuxError::VersionTooOld(current, required) => {
            eprintln!(
                "Your tmux version ({current}) is older than the minimum required ({required})."
            );
            eprintln!();
            eprintln!("Please upgrade tmux to continue.");
        }
        _ => {
            eprintln!("Please ensure tmux is properly installed and working.");
        }
    }
}

#[derive(Parser)]
#[command(name = "operator")]
#[command(about = "Multi-agent orchestration dashboard for gbqr.us")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Config file path
    #[arg(short, long)]
    config: Option<String>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Start with web view enabled
    #[arg(short = 'w', long)]
    web: bool,

    /// Open the embedded web UI in a browser on launch
    #[arg(long)]
    ui: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Show queue status
    Queue {
        /// Show all tickets, not just summary
        #[arg(short, long)]
        all: bool,
    },

    /// Launch agent for next available ticket
    Launch {
        /// Specific ticket to launch (optional)
        ticket: Option<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,

        /// Use a named delegator from config (mutually exclusive with --llm-tool/--model/--model-server)
        #[arg(long)]
        delegator: Option<String>,

        /// LLM tool override: claude, codex, gemini
        #[arg(long = "llm-tool")]
        llm_tool: Option<String>,

        /// Model override (e.g., opus, gpt-4o, qwen2.5-coder)
        #[arg(long)]
        model: Option<String>,

        /// Named model server reference (e.g., ollama-local) — overrides the delegator's default.
        /// Pairs with --llm-tool/--model for ad-hoc ollama-backed launches. v1 accepts the flag
        /// and validates the name; env-var injection on spawn ships in v2.
        #[arg(long = "model-server")]
        model_server: Option<String>,
    },

    /// List active agents
    Agents {
        /// Show detailed agent info
        #[arg(short, long)]
        verbose: bool,
    },

    /// Pause queue processing
    Pause,

    /// Resume queue processing
    Resume,

    /// Show stalled agents awaiting input
    Stalled,

    /// Create investigation from external alert
    Alert {
        /// Alert source (e.g., pagerduty, datadog)
        #[arg(long)]
        source: String,

        /// Alert message
        #[arg(long)]
        message: String,

        /// Severity (S0, S1, S2)
        #[arg(long, default_value = "S1")]
        severity: String,

        /// Affected project (optional)
        #[arg(long)]
        project: Option<String>,
    },

    /// Create a new ticket from template
    Create {
        /// Template type (feature, fix, spike, investigation)
        #[arg(short, long)]
        template: Option<String>,

        /// Target project
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Generate documentation from source-of-truth files
    Docs {
        /// Output directory (default: docs/)
        #[arg(short, long)]
        output: Option<String>,

        /// Only generate specific docs (taxonomy, issuetype, metadata)
        #[arg(short = 'g', long)]
        only: Option<String>,
    },

    /// Start the REST API server for issue type management
    Api {
        /// Port to listen on (default: 7008)
        #[arg(short, long)]
        port: Option<u16>,

        /// Open the web UI in browser after server starts
        #[arg(long)]
        open: bool,
    },

    /// Run as an MCP stdio server (for use by Claude Code, Cursor, Zed, `JetBrains`, etc.).
    ///
    /// Reads line-delimited JSON-RPC from stdin and writes responses to stdout.
    /// Log output goes to stderr. Intended to be spawned by an MCP-capable client.
    Mcp,

    /// Run as an ACP agent over stdio (for use by Zed, `JetBrains`, Emacs `agent-shell`, etc.).
    ///
    /// Implements the Agent Client Protocol. Reads line-delimited JSON-RPC
    /// from stdin and writes responses/notifications to stdout. Log output
    /// goes to stderr. Intended to be spawned by an ACP-capable editor.
    Acp,

    /// Initialize operator workspace (non-interactive by default)
    Setup {
        /// Launch TUI setup wizard instead of non-interactive setup
        #[arg(short, long)]
        interactive: bool,

        /// Collection preset: simple, dev-kanban, devops-kanban
        #[arg(short = 'C', long, default_value = "simple")]
        collection: String,

        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,

        /// Working directory (parent of .tickets/)
        #[arg(short = 'w', long)]
        working_dir: Option<PathBuf>,

        /// Kanban provider to configure: jira, linear
        #[arg(short = 'k', long)]
        kanban_provider: Option<String>,

        /// Preferred LLM tool: claude, codex, gemini
        #[arg(short = 'l', long)]
        llm_tool: Option<String>,

        /// Skip LLM tool detection
        #[arg(long)]
        skip_llm_detection: bool,
    },

    /// Convert between operator issuetypes and other orchestration formats
    Workflow {
        #[command(subcommand)]
        action: WorkflowAction,
    },
}

#[derive(Subcommand)]
enum WorkflowAction {
    /// Export a ticket, rendered against its issuetype, to an orchestration workflow
    Export {
        /// Ticket id (e.g. FEAT-1234) or path to a ticket markdown file
        ticket: String,

        /// Output path (default: derived from the ticket id + format; "-" for stdout)
        #[arg(short, long)]
        out: Option<PathBuf>,

        /// Output format: claude (Claude Code .js) or agnt (AGNT.gg workflow JSON)
        #[arg(long, value_enum, default_value_t)]
        format: workflow_gen::WorkflowFormat,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration first (needed for logging setup)
    let config = Config::load(cli.config.as_deref())?;

    // Determine if we're running in TUI mode (no subcommand)
    let is_tui_mode = cli.command.is_none();

    // Initialize logging (file-based for TUI, stderr for CLI)
    let logging_handle = logging::init_logging(&config, is_tui_mode, cli.debug)?;

    // Inject the status-section provider into the REST layer. The section logic
    // lives in `ui` (which `rest` can't depend on — see rest::dto::sections), so
    // the binary registers it here, before any server starts. Covers all serving
    // paths (TUI app, `operator rest`, embedded UI) since they share one process.
    rest::dto::register_section_provider(std::sync::Arc::new(|config, registry, live| {
        let issue_types = registry
            .all_types()
            .map(|it| ui::status_panel::IssueTypeInfo {
                key: it.key.clone(),
                name: it.name.clone(),
                mode: if it.is_autonomous() {
                    "autonomous".to_string()
                } else {
                    "paired".to_string()
                },
            })
            .collect();
        let mut snapshot = ui::status_panel::StatusSnapshot::from_config(config, issue_types);
        snapshot.apply_api_connection(live);
        ui::status_panel::build_section_dtos(&snapshot)
    }));

    match cli.command {
        Some(Commands::Queue { all }) => {
            cmd_queue(&config, all).await?;
        }
        Some(Commands::Launch {
            ticket,
            yes,
            delegator,
            llm_tool,
            model,
            model_server,
        }) => {
            cmd_launch(
                &config,
                ticket,
                yes,
                LaunchOverrides {
                    delegator,
                    llm_tool,
                    model,
                    model_server,
                },
            )
            .await?;
        }
        Some(Commands::Agents { verbose }) => {
            cmd_agents(&config, verbose).await?;
        }
        Some(Commands::Pause) => {
            cmd_pause(&config).await?;
        }
        Some(Commands::Resume) => {
            cmd_resume(&config).await?;
        }
        Some(Commands::Stalled) => {
            cmd_stalled(&config).await?;
        }
        Some(Commands::Alert {
            source,
            message,
            severity,
            project,
        }) => {
            cmd_alert(&config, source, message, severity, project).await?;
        }
        Some(Commands::Create { template, project }) => {
            cmd_create(&config, template, project).await?;
        }
        Some(Commands::Docs { output, only }) => {
            cmd_docs(&config, output, only)?;
        }
        Some(Commands::Api { port, open }) => {
            cmd_api(&config, port, open).await?;
        }
        Some(Commands::Mcp) => {
            cmd_mcp(&config).await?;
        }
        Some(Commands::Acp) => {
            cmd_acp(&config).await?;
        }
        Some(Commands::Setup {
            interactive,
            collection,
            force,
            working_dir,
            kanban_provider,
            llm_tool,
            skip_llm_detection,
        }) => {
            cmd_setup(
                config,
                interactive,
                collection,
                force,
                working_dir,
                kanban_provider,
                llm_tool,
                skip_llm_detection,
            )?;
        }
        Some(Commands::Workflow { action }) => {
            cmd_workflow(&config, action)?;
        }
        None => {
            // No subcommand = launch TUI dashboard
            #[allow(clippy::large_futures)] // TUI state is inherently large
            run_tui(config, logging_handle.log_file_path, cli.web, cli.ui).await?;
        }
    }

    Ok(())
}

async fn run_tui(
    config: Config,
    log_file_path: Option<PathBuf>,
    start_web: bool,
    open_ui: bool,
) -> Result<()> {
    // Install panic hook before any terminal operations
    // This ensures terminal is restored even on panic
    crate::ui::install_panic_hook();

    // Note: tmux availability is now checked in the setup wizard (TmuxOnboarding step)
    // when the user selects tmux as their session wrapper
    let mut app = App::new(config, start_web, open_ui).await?;
    let result = app.run().await;

    // Print log file path on exit if logs were written
    if let Some(log_path) = log_file_path {
        if log_path.exists() {
            if let Ok(metadata) = log_path.metadata() {
                if metadata.len() > 0 {
                    eprintln!("Session log: {}", log_path.display());
                }
            }
        }
    }

    result
}

async fn cmd_queue(config: &Config, all: bool) -> Result<()> {
    let queue = queue::Queue::new(config)?;
    let tickets = queue.list_by_priority()?;

    if tickets.is_empty() {
        println!("Queue is empty");
        return Ok(());
    }

    println!("Ticket Queue ({} tickets)", tickets.len());
    println!("{}", "─".repeat(60));

    let display_count = if all {
        tickets.len()
    } else {
        10.min(tickets.len())
    };

    for ticket in tickets.iter().take(display_count) {
        let glyph = glyph_for_key(&ticket.ticket_type);
        println!("{} {}", glyph, ticket.summary);
    }

    if !all && tickets.len() > 10 {
        println!("... and {} more (use --all to see all)", tickets.len() - 10);
    }

    Ok(())
}

/// Ad-hoc launch overrides parsed from CLI flags.
///
/// Flags are validated up front, then resolved through `resolve_launch_options`,
/// which injects the chosen `model_server`'s env vars into the spawned agent.
#[derive(Debug, Default)]
struct LaunchOverrides {
    delegator: Option<String>,
    llm_tool: Option<String>,
    model: Option<String>,
    model_server: Option<String>,
}

async fn cmd_launch(
    config: &Config,
    ticket: Option<String>,
    skip_confirm: bool,
    overrides: LaunchOverrides,
) -> Result<()> {
    // Validate CLI overrides up front so bad input doesn't get swallowed later.
    // Named model_server must exist among declared servers or implicit builtins.
    if let Some(ref name) = overrides.model_server {
        let declared = config.model_servers.iter().any(|s| &s.name == name);
        let implicit = ["claude", "codex", "gemini"]
            .iter()
            .any(|t| &config::implicit_model_server_for_tool(t).name == name);
        if !declared && !implicit {
            anyhow::bail!(
                "Unknown model-server '{name}'. Declare it under [[model_servers]] in your config."
            );
        }
    }
    // --delegator and ad-hoc (--llm-tool / --model / --model-server) are mutually exclusive.
    let has_adhoc = overrides.llm_tool.is_some()
        || overrides.model.is_some()
        || overrides.model_server.is_some();
    if overrides.delegator.is_some() && has_adhoc {
        anyhow::bail!(
            "--delegator is mutually exclusive with --llm-tool / --model / --model-server"
        );
    }

    // Resolve launch options from the CLI overrides (a named delegator, or the
    // ad-hoc --llm-tool/--model/--model-server trio). The chosen model server's
    // env vars (OPENAI_BASE_URL, ANTHROPIC_BASE_URL, …) are threaded into
    // LaunchOptions.provider.env and exported before the agent CLI spawns.
    let launch_options = crate::agents::delegator_resolution::resolve_launch_options(
        config,
        overrides.delegator.as_deref(),
        overrides.llm_tool.as_deref(),
        overrides.model.as_deref(),
        overrides.model_server.as_deref(),
        false,
        None,
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Check tmux availability before launching
    if let Err(err) = check_tmux_available() {
        print_tmux_error(&err);
        std::process::exit(1);
    }

    let queue = queue::Queue::new(config)?;
    let state = state::State::load(config)?;

    // Check if we can launch more agents
    let running_count = state.running_agents().len();
    let max_agents = config.effective_max_agents();

    if running_count >= max_agents {
        println!("Cannot launch: {running_count} agents running (max {max_agents})");
        println!("Use 'operator agents' to see running agents");
        return Ok(());
    }

    // Get ticket to launch
    let ticket = match ticket {
        Some(id) => queue.find_ticket(&id)?,
        None => queue.next_ticket()?,
    };

    let Some(ticket) = ticket else {
        println!("No tickets available to launch");
        return Ok(());
    };

    // Confirmation
    if !skip_confirm {
        println!("Launch agent for ticket?");
        println!();
        println!("  Type:    {}", ticket.ticket_type);
        println!("  ID:      {}", ticket.id);
        println!("  Project: {}", ticket.project);
        println!("  Summary: {}", ticket.summary);
        println!();
        print!("Confirm? [y/N] ");

        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled");
            return Ok(());
        }
    }

    // Launch agent
    let launcher = agents::Launcher::new(config)?;
    launcher
        .launch_with_options(&ticket, launch_options)
        .await?;

    println!("Launched agent for {}-{}", ticket.ticket_type, ticket.id);

    Ok(())
}

async fn cmd_agents(config: &Config, verbose: bool) -> Result<()> {
    let state = state::State::load(config)?;
    let agents = state.running_agents();

    if agents.is_empty() {
        println!("No agents running");
        return Ok(());
    }

    println!(
        "Running Agents ({}/{})",
        agents.len(),
        config.effective_max_agents()
    );
    println!("{}", "─".repeat(60));

    for agent in &agents {
        let status_icon = match agent.status.as_str() {
            "running" => "▶",
            "awaiting_input" => "⏸",
            "completing" => "✓",
            _ => "?",
        };

        println!(
            "{} {} [{}] {}-{}",
            status_icon, agent.project, agent.status, agent.ticket_type, agent.ticket_id
        );

        if verbose {
            println!("    Started: {}", agent.started_at);
            if let Some(ref msg) = agent.last_message {
                println!("    Last: {msg}");
            }
            println!();
        }
    }

    Ok(())
}

async fn cmd_pause(config: &Config) -> Result<()> {
    let mut state = state::State::load(config)?;
    state.set_paused(true)?;
    println!("Queue processing paused");
    Ok(())
}

async fn cmd_resume(config: &Config) -> Result<()> {
    let mut state = state::State::load(config)?;
    state.set_paused(false)?;
    println!("Queue processing resumed");
    Ok(())
}

async fn cmd_stalled(config: &Config) -> Result<()> {
    let state = state::State::load(config)?;
    let stalled = state.stalled_agents();

    if stalled.is_empty() {
        println!("No agents awaiting input");
        return Ok(());
    }

    println!("Agents Awaiting Input ({})", stalled.len());
    println!("{}", "─".repeat(60));

    for agent in &stalled {
        println!(
            "⏸ {} [{}-{}]",
            agent.project, agent.ticket_type, agent.ticket_id
        );
        if let Some(ref msg) = agent.last_message {
            println!("    Question: {msg}");
        }
        println!();
    }

    Ok(())
}

async fn cmd_alert(
    config: &Config,
    source: String,
    message: String,
    severity: String,
    project: Option<String>,
) -> Result<()> {
    let queue = queue::Queue::new(config)?;

    let ticket = queue.create_investigation(source.clone(), message, severity.clone(), project)?;

    println!("Created investigation ticket: {}", ticket.filename);

    // Send notification
    let notification_service = notifications::NotificationService::from_config(config)?;
    notification_service
        .notify(notifications::NotificationEvent::InvestigationCreated {
            source,
            severity,
            summary: ticket.summary.clone(),
            ticket_id: ticket.id.clone(),
        })
        .await;

    Ok(())
}

async fn cmd_create(
    config: &Config,
    template: Option<String>,
    project: Option<String>,
) -> Result<()> {
    use crate::queue::TicketCreator;
    use crate::templates::TemplateType;

    // Parse template type
    let template_type = match template.as_deref() {
        Some("feature" | "feat") => TemplateType::Feature,
        Some("fix") => TemplateType::Fix,
        Some("spike") => TemplateType::Spike,
        Some("inv" | "investigation") => TemplateType::Investigation,
        Some(other) => {
            println!("Unknown template type: {other}. Use: feature, fix, spike, investigation");
            return Ok(());
        }
        None => {
            println!("Please specify template type with --template");
            println!("Options: feature (feat), fix, spike, investigation (inv)");
            return Ok(());
        }
    };

    let project = project.unwrap_or_else(|| "global".to_string());

    let editor_config = editors::EditorConfig::detect(config.sessions.wrapper);
    let creator = TicketCreator::new(config);
    let filepath = creator.create_ticket(template_type, &project, editor_config.file_editor())?;

    println!("Created ticket: {}", filepath.display());

    Ok(())
}

fn cmd_workflow(config: &Config, action: WorkflowAction) -> Result<()> {
    match action {
        WorkflowAction::Export {
            ticket,
            out,
            format,
        } => {
            // Resolve the ticket (by id via the queue, or as a direct file path).
            // Resolution is the only edge-specific step; the registry build and
            // the export itself go through the same shared path as the REST API.
            let resolved = {
                let path = std::path::Path::new(&ticket);
                if path.is_file() {
                    queue::Ticket::from_file(path)?
                } else {
                    let queue = queue::Queue::new(config)?;
                    queue
                        .find_ticket(&ticket)?
                        .ok_or_else(|| anyhow::anyhow!("Ticket not found: {ticket}"))?
                }
            };

            // Same registry loader the REST API (ApiState::new) uses.
            let registry = startup::templates::load_registry(&config.tickets_path());
            let exported = workflow_gen::export_workflow_for_ticket(
                &resolved, &registry, None, config, format,
            )?;

            match out.as_deref() {
                Some(p) if p == std::path::Path::new("-") => {
                    print!("{}", exported.contents);
                }
                Some(p) => {
                    std::fs::write(p, &exported.contents)?;
                    println!("Wrote workflow to {}", p.display());
                }
                None => {
                    let default = PathBuf::from(&exported.suggested_filename);
                    std::fs::write(&default, &exported.contents)?;
                    println!("Wrote workflow to {}", default.display());
                }
            }
        }
    }
    Ok(())
}

fn cmd_docs(_config: &Config, output: Option<String>, only: Option<String>) -> Result<()> {
    use docs_gen::{
        cli, config, config_schema, issuetype, issuetype_json_schema, jira_api, llms, metadata,
        openapi, operator_output_schema, project_analysis_schema, schema_index, shortcuts, startup,
        state_schema, taxonomy, DocGenerator,
    };
    use std::path::PathBuf;

    // Determine output directory (default: ./docs relative to current directory)
    let docs_dir = match output {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir().unwrap_or_default().join("docs"),
    };

    println!("Generating documentation to: {}", docs_dir.display());

    // Build list of generators based on --only filter
    let generators: Vec<Box<dyn DocGenerator>> = match only.as_deref() {
        Some("taxonomy") => {
            vec![Box::new(taxonomy::TaxonomyDocGenerator)]
        }
        Some("issuetype") => {
            vec![Box::new(issuetype::IssuetypeSchemaDocGenerator)]
        }
        Some("metadata") => {
            vec![Box::new(metadata::MetadataSchemaDocGenerator)]
        }
        Some("shortcuts") => {
            vec![Box::new(shortcuts::ShortcutsDocGenerator)]
        }
        Some("cli") => {
            vec![Box::new(cli::CliDocGenerator)]
        }
        Some("config") => {
            vec![Box::new(config::ConfigDocGenerator)]
        }
        Some("openapi") => {
            vec![Box::new(openapi::OpenApiDocGenerator)]
        }
        Some("startup") => {
            vec![Box::new(startup::StartupDocGenerator)]
        }
        Some("config-schema") => {
            vec![Box::new(config_schema::ConfigSchemaDocGenerator)]
        }
        Some("state-schema") => {
            vec![Box::new(state_schema::StateSchemaDocGenerator)]
        }
        Some("schema-index") => {
            vec![Box::new(schema_index::SchemaIndexDocGenerator)]
        }
        Some("jira-api") => {
            vec![Box::new(jira_api::JiraApiDocGenerator)]
        }
        Some("operator-output-schema") => {
            vec![Box::new(
                operator_output_schema::OperatorOutputSchemaDocGenerator,
            )]
        }
        Some("issuetype-json-schema") => {
            vec![Box::new(
                issuetype_json_schema::IssuetypeJsonSchemaDocGenerator,
            )]
        }
        Some("project-analysis-schema") => {
            vec![Box::new(
                project_analysis_schema::ProjectAnalysisSchemaDocGenerator,
            )]
        }
        Some("llms") => {
            vec![Box::new(llms::LlmsTxtDocGenerator)]
        }
        Some(other) => {
            println!(
                "Unknown generator: {other}. Available: taxonomy, issuetype, metadata, shortcuts, cli, config, openapi, startup, config-schema, state-schema, schema-index, jira-api, operator-output-schema, issuetype-json-schema, project-analysis-schema, llms"
            );
            return Ok(());
        }
        None => {
            // Generate all
            vec![
                Box::new(taxonomy::TaxonomyDocGenerator),
                Box::new(issuetype::IssuetypeSchemaDocGenerator),
                Box::new(metadata::MetadataSchemaDocGenerator),
                Box::new(shortcuts::ShortcutsDocGenerator),
                Box::new(cli::CliDocGenerator),
                Box::new(config::ConfigDocGenerator),
                Box::new(openapi::OpenApiDocGenerator),
                Box::new(startup::StartupDocGenerator),
                Box::new(config_schema::ConfigSchemaDocGenerator),
                Box::new(state_schema::StateSchemaDocGenerator),
                Box::new(schema_index::SchemaIndexDocGenerator),
                Box::new(jira_api::JiraApiDocGenerator),
                Box::new(operator_output_schema::OperatorOutputSchemaDocGenerator),
                Box::new(issuetype_json_schema::IssuetypeJsonSchemaDocGenerator),
                Box::new(project_analysis_schema::ProjectAnalysisSchemaDocGenerator),
                Box::new(llms::LlmsTxtDocGenerator),
            ]
        }
    };

    for generator in generators {
        generator.write(&docs_dir)?;
        println!("  ✓ {} → {}", generator.name(), generator.output_path());
    }

    println!("\nDocumentation generation complete.");
    Ok(())
}

async fn cmd_api(config: &Config, port: Option<u16>, open: bool) -> Result<()> {
    let port = port.unwrap_or(config.rest_api.port);

    println!("Starting REST API server...");
    println!("  Port: {port}");
    println!("  Endpoints:");
    println!("    GET  /api/v1/health           Health check");
    println!("    GET  /api/v1/status           Server status");
    println!("    GET  /api/v1/issuetypes       List issue types");
    println!("    GET  /api/v1/issuetypes/:key  Get issue type");
    println!("    POST /api/v1/issuetypes       Create issue type");
    println!("    PUT  /api/v1/issuetypes/:key  Update issue type");
    println!("    GET  /api/v1/collections      List collections");
    println!();

    if open {
        let url = format!("http://localhost:{port}/");
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let opener = if cfg!(target_os = "macos") {
                "open"
            } else if cfg!(target_os = "windows") {
                "cmd"
            } else {
                "xdg-open"
            };
            if cfg!(target_os = "windows") {
                let _ = std::process::Command::new(opener)
                    .args(["/C", "start", &url])
                    .spawn();
            } else {
                let _ = std::process::Command::new(opener).arg(&url).spawn();
            }
        });
    }

    let state = rest::ApiState::new(config.clone(), config.tickets_path());
    rest::serve(state, port).await?;

    Ok(())
}

async fn cmd_mcp(config: &Config) -> Result<()> {
    let state = rest::ApiState::new(config.clone(), config.tickets_path());
    tracing::info!("Starting MCP stdio server");
    mcp::stdio::run(state, tokio::io::stdin(), tokio::io::stdout()).await?;
    tracing::info!("MCP stdio server stopped (stdin closed)");
    Ok(())
}

async fn cmd_acp(config: &Config) -> Result<()> {
    tracing::info!("Starting ACP stdio agent");
    acp::run_stdio(config.clone())
        .await
        .map_err(|e| anyhow::anyhow!("ACP transport error: {e:?}"))?;
    tracing::info!("ACP agent stopped (stdin closed)");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_setup(
    mut config: Config,
    interactive: bool,
    collection: String,
    force: bool,
    working_dir: Option<PathBuf>,
    kanban_provider: Option<String>,
    llm_tool: Option<String>,
    skip_llm_detection: bool,
) -> Result<()> {
    use setup::{initialize_workspace, parse_collection_preset, SetupOptions};

    if interactive {
        // For interactive mode, just launch the TUI which has its own setup wizard
        println!("Interactive setup not yet implemented.");
        println!("Run 'operator' without arguments to use the TUI setup wizard.");
        return Ok(());
    }

    // Parse collection preset
    let preset = parse_collection_preset(&collection)?;

    // Validate kanban provider if specified
    if let Some(ref provider) = kanban_provider {
        match provider.to_lowercase().as_str() {
            "jira" | "linear" => {}
            other => {
                anyhow::bail!("Unknown kanban provider: {other}. Use 'jira' or 'linear'.");
            }
        }
    }

    // Validate LLM tool if specified
    if let Some(ref tool) = llm_tool {
        match tool.to_lowercase().as_str() {
            "claude" | "codex" | "gemini" => {}
            other => {
                anyhow::bail!("Unknown LLM tool: {other}. Use 'claude', 'codex', or 'gemini'.");
            }
        }
    }

    // Detect LLM tools unless skipped
    if !skip_llm_detection && llm_tool.is_none() {
        let detected = detect_llm_tools();
        if !detected.is_empty() {
            println!("Detected LLM tools: {}", detected.join(", "));
        }
    }

    let options = SetupOptions {
        preset,
        force,
        working_dir,
        kanban_provider,
        llm_tool,
        ..Default::default()
    };

    // Apply working_dir to config paths so tickets_path() resolves correctly
    // (without this, relative ".tickets" resolves against cwd which may be wrong)
    if let Some(ref wd) = options.working_dir {
        config.paths.tickets = wd.join(".tickets").to_string_lossy().to_string();
        config.paths.projects = wd.to_string_lossy().to_string();
        config.paths.state = wd
            .join(".tickets")
            .join("operator")
            .to_string_lossy()
            .to_string();
    }

    println!("Initializing operator workspace...");
    println!("  Collection: {:?}", options.preset);
    println!("  Force:      {}", options.force);
    if let Some(ref dir) = options.working_dir {
        println!("  Working Dir: {}", dir.display());
    }
    if let Some(ref provider) = options.kanban_provider {
        println!("  Kanban:     {provider}");
    }
    if let Some(ref tool) = options.llm_tool {
        println!("  LLM Tool:   {tool}");
    }
    println!();

    let result = initialize_workspace(&mut config, &options)?;

    // Report results
    if !result.directories_created.is_empty() {
        println!("Created directories:");
        for dir in &result.directories_created {
            println!("  {}", dir.display());
        }
    }

    if !result.files_created.is_empty() {
        println!("Created files:");
        for file in &result.files_created {
            println!("  {}", file.display());
        }
    }

    if !result.files_skipped.is_empty() {
        println!("Skipped (already exist):");
        for file in &result.files_skipped {
            println!("  {}", file.display());
        }
    }

    println!();
    println!("Configuration saved to: {}", result.config_path.display());
    println!();
    println!("Workspace initialized successfully!");
    println!("Run 'operator' to launch the TUI dashboard.");

    Ok(())
}

/// Parse a template type string into a `TemplateType`.
/// Used for testing and CLI parsing.
#[allow(dead_code)]
fn parse_template_type(s: &str) -> Option<templates::TemplateType> {
    use templates::TemplateType;
    match s {
        "feature" | "feat" => Some(TemplateType::Feature),
        "fix" => Some(TemplateType::Fix),
        "spike" => Some(TemplateType::Spike),
        "inv" | "investigation" => Some(TemplateType::Investigation),
        "task" => Some(TemplateType::Task),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use templates::TemplateType;

    #[test]
    fn test_cli_parsing_no_args() {
        // Verify the CLI struct is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_parse_template_type_feature() {
        assert_eq!(parse_template_type("feature"), Some(TemplateType::Feature));
        assert_eq!(parse_template_type("feat"), Some(TemplateType::Feature));
    }

    #[test]
    fn test_parse_template_type_fix() {
        assert_eq!(parse_template_type("fix"), Some(TemplateType::Fix));
    }

    #[test]
    fn test_parse_template_type_spike() {
        assert_eq!(parse_template_type("spike"), Some(TemplateType::Spike));
    }

    #[test]
    fn test_parse_template_type_investigation() {
        assert_eq!(
            parse_template_type("inv"),
            Some(TemplateType::Investigation)
        );
        assert_eq!(
            parse_template_type("investigation"),
            Some(TemplateType::Investigation)
        );
    }

    #[test]
    fn test_parse_template_type_task() {
        assert_eq!(parse_template_type("task"), Some(TemplateType::Task));
    }

    #[test]
    fn test_parse_template_type_invalid() {
        assert_eq!(parse_template_type("unknown"), None);
        assert_eq!(parse_template_type(""), None);
        assert_eq!(parse_template_type("FEAT"), None); // Case sensitive
    }

    #[test]
    fn test_glyph_for_key_returns_values() {
        // Test that glyph_for_key returns non-empty strings for known types
        assert!(!glyph_for_key("FEAT").is_empty());
        assert!(!glyph_for_key("FIX").is_empty());
        assert!(!glyph_for_key("SPIKE").is_empty());
        assert!(!glyph_for_key("INV").is_empty());
        assert!(!glyph_for_key("TASK").is_empty());
    }

    #[test]
    fn test_glyph_for_key_unknown_returns_default() {
        // Unknown types should return a default glyph
        let unknown_glyph = glyph_for_key("UNKNOWN");
        assert!(!unknown_glyph.is_empty());
    }

    #[test]
    fn test_cli_setup_with_working_dir() {
        use clap::Parser;
        let result = Cli::try_parse_from(["operator", "setup", "--working-dir", "/tmp/test"]);
        assert!(result.is_ok());
        if let Ok(cli) = result {
            match cli.command {
                Some(Commands::Setup { working_dir, .. }) => {
                    assert_eq!(working_dir, Some(std::path::PathBuf::from("/tmp/test")));
                }
                _ => panic!("Expected Setup command"),
            }
        }
    }

    #[test]
    fn test_cli_setup_with_kanban_provider() {
        use clap::Parser;
        let result = Cli::try_parse_from(["operator", "setup", "--kanban-provider", "jira"]);
        assert!(result.is_ok());
        if let Ok(cli) = result {
            match cli.command {
                Some(Commands::Setup {
                    kanban_provider, ..
                }) => {
                    assert_eq!(kanban_provider, Some("jira".to_string()));
                }
                _ => panic!("Expected Setup command"),
            }
        }
    }

    #[test]
    fn test_cli_setup_with_llm_tool() {
        use clap::Parser;
        let result = Cli::try_parse_from(["operator", "setup", "--llm-tool", "claude"]);
        assert!(result.is_ok());
        if let Ok(cli) = result {
            match cli.command {
                Some(Commands::Setup { llm_tool, .. }) => {
                    assert_eq!(llm_tool, Some("claude".to_string()));
                }
                _ => panic!("Expected Setup command"),
            }
        }
    }

    #[test]
    fn test_cli_setup_with_skip_llm_detection() {
        use clap::Parser;
        let result = Cli::try_parse_from(["operator", "setup", "--skip-llm-detection"]);
        assert!(result.is_ok());
        if let Ok(cli) = result {
            match cli.command {
                Some(Commands::Setup {
                    skip_llm_detection, ..
                }) => {
                    assert!(skip_llm_detection);
                }
                _ => panic!("Expected Setup command"),
            }
        }
    }

    #[test]
    fn test_cli_setup_all_new_flags() {
        use clap::Parser;
        let result = Cli::try_parse_from([
            "operator",
            "setup",
            "-w",
            "/tmp/workspace",
            "-k",
            "linear",
            "-l",
            "gemini",
            "--skip-llm-detection",
        ]);
        assert!(result.is_ok());
        if let Ok(cli) = result {
            match cli.command {
                Some(Commands::Setup {
                    working_dir,
                    kanban_provider,
                    llm_tool,
                    skip_llm_detection,
                    ..
                }) => {
                    assert_eq!(
                        working_dir,
                        Some(std::path::PathBuf::from("/tmp/workspace"))
                    );
                    assert_eq!(kanban_provider, Some("linear".to_string()));
                    assert_eq!(llm_tool, Some("gemini".to_string()));
                    assert!(skip_llm_detection);
                }
                _ => panic!("Expected Setup command"),
            }
        }
    }

    #[test]
    fn test_detect_llm_tools_returns_vec() {
        let tools = detect_llm_tools();
        // Just verify it returns a Vec, actual content depends on environment
        assert!(tools.len() <= 3);
    }

    #[test]
    fn test_check_kanban_env_vars_no_vars_set() {
        // Save original values
        let orig_jira = std::env::var("OPERATOR_JIRA_API_KEY").ok();
        let orig_jira_token = std::env::var("JIRA_API_TOKEN").ok();
        let orig_linear = std::env::var("OPERATOR_LINEAR_API_KEY").ok();
        let orig_linear_key = std::env::var("LINEAR_API_KEY").ok();

        // Clear all
        std::env::remove_var("OPERATOR_JIRA_API_KEY");
        std::env::remove_var("JIRA_API_TOKEN");
        std::env::remove_var("OPERATOR_LINEAR_API_KEY");
        std::env::remove_var("LINEAR_API_KEY");

        let (jira, linear) = check_kanban_env_vars();
        assert!(!jira);
        assert!(!linear);

        // Restore
        if let Some(v) = orig_jira {
            std::env::set_var("OPERATOR_JIRA_API_KEY", v);
        }
        if let Some(v) = orig_jira_token {
            std::env::set_var("JIRA_API_TOKEN", v);
        }
        if let Some(v) = orig_linear {
            std::env::set_var("OPERATOR_LINEAR_API_KEY", v);
        }
        if let Some(v) = orig_linear_key {
            std::env::set_var("LINEAR_API_KEY", v);
        }
    }

    #[test]
    fn test_check_kanban_env_vars_jira_set() {
        // Save original
        let orig = std::env::var("OPERATOR_JIRA_API_KEY").ok();

        std::env::set_var("OPERATOR_JIRA_API_KEY", "test-key");
        let (jira, _) = check_kanban_env_vars();
        assert!(jira);

        // Restore
        if let Some(v) = orig {
            std::env::set_var("OPERATOR_JIRA_API_KEY", v);
        } else {
            std::env::remove_var("OPERATOR_JIRA_API_KEY");
        }
    }

    #[test]
    fn test_check_kanban_env_vars_linear_set() {
        // Save original
        let orig = std::env::var("OPERATOR_LINEAR_API_KEY").ok();

        std::env::set_var("OPERATOR_LINEAR_API_KEY", "lin_test");
        let (_, linear) = check_kanban_env_vars();
        assert!(linear);

        // Restore
        if let Some(v) = orig {
            std::env::set_var("OPERATOR_LINEAR_API_KEY", v);
        } else {
            std::env::remove_var("OPERATOR_LINEAR_API_KEY");
        }
    }
}
