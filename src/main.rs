use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod api;
mod app;
mod backstage;
mod config;
mod issuetypes;
mod llm;
mod logging;
mod permissions;
mod pr_config;
mod projects;
mod state;
mod steps;
mod templates;

mod agents;
mod docs_gen;
pub mod env_vars;
mod notifications;
mod queue;
mod rest;
mod ui;

use agents::tmux::{SystemTmuxClient, TmuxClient, TmuxError};
use app::App;
use config::Config;
use templates::glyph_for_key;

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
            format!("{}.{}", MIN_MAJOR, MIN_MINOR),
        ));
    }

    tracing::debug!(version = %version.raw, "tmux available");
    Ok(())
}

/// Print a helpful error message for tmux issues
fn print_tmux_error(err: &TmuxError) {
    eprintln!("Error: {}", err);
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
                "Your tmux version ({}) is older than the minimum required ({}).",
                current, required
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

    match cli.command {
        Some(Commands::Queue { all }) => {
            cmd_queue(&config, all).await?;
        }
        Some(Commands::Launch { ticket, yes }) => {
            cmd_launch(&config, ticket, yes).await?;
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
        Some(Commands::Api { port }) => {
            cmd_api(&config, port).await?;
        }
        None => {
            // No subcommand = launch TUI dashboard
            run_tui(config, logging_handle.log_file_path).await?;
        }
    }

    Ok(())
}

async fn run_tui(config: Config, log_file_path: Option<PathBuf>) -> Result<()> {
    // Check tmux availability before starting TUI
    if let Err(err) = check_tmux_available() {
        print_tmux_error(&err);
        std::process::exit(1);
    }

    let mut app = App::new(config)?;
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

async fn cmd_launch(config: &Config, ticket: Option<String>, skip_confirm: bool) -> Result<()> {
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
        println!(
            "Cannot launch: {} agents running (max {})",
            running_count, max_agents
        );
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
    launcher.launch(&ticket).await?;

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
                println!("    Last: {}", msg);
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
            println!("    Question: {}", msg);
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

    let ticket = queue.create_investigation(source, message, severity, project)?;

    println!("Created investigation ticket: {}", ticket.filename);

    // Send notification
    if config.notifications.enabled && config.notifications.on_investigation_created {
        notifications::send(
            "Investigation Created",
            &format!("{}-{}", ticket.ticket_type, ticket.id),
            &ticket.summary,
            config.notifications.sound,
        )?;
    }

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
        Some("feature") | Some("feat") => TemplateType::Feature,
        Some("fix") => TemplateType::Fix,
        Some("spike") => TemplateType::Spike,
        Some("inv") | Some("investigation") => TemplateType::Investigation,
        Some(other) => {
            println!(
                "Unknown template type: {}. Use: feature, fix, spike, investigation",
                other
            );
            return Ok(());
        }
        None => {
            println!("Please specify template type with --template");
            println!("Options: feature (feat), fix, spike, investigation (inv)");
            return Ok(());
        }
    };

    let project = project.unwrap_or_else(|| "global".to_string());

    let creator = TicketCreator::new(config);
    let filepath = creator.create_ticket(template_type, &project)?;

    println!("Created ticket: {}", filepath.display());

    Ok(())
}

fn cmd_docs(_config: &Config, output: Option<String>, only: Option<String>) -> Result<()> {
    use docs_gen::{cli, config, issuetype, metadata, openapi, shortcuts, taxonomy, DocGenerator};
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
        Some(other) => {
            println!(
                "Unknown generator: {}. Use: taxonomy, issuetype, metadata, shortcuts, cli, config, openapi",
                other
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

async fn cmd_api(config: &Config, port: Option<u16>) -> Result<()> {
    let port = port.unwrap_or(config.rest_api.port);

    println!("Starting REST API server...");
    println!("  Port: {}", port);
    println!("  Endpoints:");
    println!("    GET  /api/v1/health           Health check");
    println!("    GET  /api/v1/status           Server status");
    println!("    GET  /api/v1/issuetypes       List issue types");
    println!("    GET  /api/v1/issuetypes/:key  Get issue type");
    println!("    POST /api/v1/issuetypes       Create issue type");
    println!("    PUT  /api/v1/issuetypes/:key  Update issue type");
    println!("    GET  /api/v1/collections      List collections");
    println!();

    let state = rest::ApiState::new(config.clone(), config.tickets_path());
    rest::serve(state, port).await?;

    Ok(())
}
