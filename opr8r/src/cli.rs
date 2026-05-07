use clap::Parser;

/// Minimal CLI wrapper for LLM commands in multi-step ticket workflows.
///
/// Wraps LLM commands (claude, gemini, codex), passes through output,
/// and orchestrates step transitions via the Operator API.
///
/// Run `opr8r relay` to start as an MCP stdio relay server.
#[derive(Parser, Debug)]
#[command(name = "opr8r")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Subcommand (e.g., relay). If absent, runs in step-wrapper mode.
    #[command(subcommand)]
    pub subcommand: Option<Cmd>,

    /// Ticket ID being worked on (e.g., FEAT-123) [required in step-wrapper mode]
    #[arg(long)]
    pub ticket_id: Option<String>,

    /// Current step name (e.g., "plan", "build", "test") [required in step-wrapper mode]
    #[arg(long)]
    pub step: Option<String>,

    /// Operator API URL. If not provided, auto-discovers from
    /// .tickets/operator/api-session.json
    #[arg(long)]
    pub api_url: Option<String>,

    /// Session ID for LLM session tracking (passed to claude --session-id)
    #[arg(long)]
    pub session_id: Option<String>,

    /// Disable automatic step transition even for review_type=none
    #[arg(long, default_value = "false")]
    pub no_auto_proceed: bool,

    /// Enable verbose logging to stderr
    #[arg(long, short, default_value = "false")]
    pub verbose: bool,

    /// Show what would happen without executing
    #[arg(long, default_value = "false")]
    pub dry_run: bool,

    /// The LLM command and its arguments to execute [required in step-wrapper mode]
    #[arg(last = true)]
    pub command: Vec<String>,
}

/// Available subcommands for opr8r.
#[derive(clap::Subcommand, Debug, PartialEq)]
pub enum Cmd {
    /// Run as an MCP stdio relay server.
    ///
    /// Connects to the relay hub and exposes relay tools (relay_peers,
    /// relay_ask, relay_reply, relay_broadcast, relay_rename) to LLM agents
    /// via the MCP stdio protocol.
    Relay,
}

impl Args {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Args::parse()
    }

    /// Validate that all required step-wrapper fields are present.
    /// Returns Err with a descriptive message if any are missing.
    pub fn validate_step_wrapper(&self) -> Result<(), String> {
        if self.ticket_id.is_none() {
            return Err("--ticket-id is required in step-wrapper mode".to_string());
        }
        if self.step.is_none() {
            return Err("--step is required in step-wrapper mode".to_string());
        }
        if self.command.is_empty() {
            return Err("command is required in step-wrapper mode (pass after --)".to_string());
        }
        Ok(())
    }

    /// Get the LLM command as program and arguments
    pub fn command_parts(&self) -> Option<(&str, &[String])> {
        if self.command.is_empty() {
            None
        } else {
            Some((&self.command[0], &self.command[1..]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_subcommand_parses() {
        let result = Args::try_parse_from(["opr8r", "relay"]);
        assert!(result.is_ok(), "relay subcommand should parse successfully");
        let args = result.unwrap();
        assert!(matches!(args.subcommand, Some(Cmd::Relay)));
    }

    #[test]
    fn test_step_wrapper_mode_still_requires_ticket_id() {
        let args = Args::try_parse_from(["opr8r", "--step=plan", "--", "claude"]).unwrap();
        let err = args.validate_step_wrapper().unwrap_err();
        assert!(
            err.contains("ticket-id"),
            "error should mention ticket-id, got: {err}"
        );
    }

    #[test]
    fn test_step_wrapper_mode_still_requires_step() {
        let args = Args::try_parse_from(["opr8r", "--ticket-id=FEAT-1", "--", "claude"]).unwrap();
        let err = args.validate_step_wrapper().unwrap_err();
        assert!(
            err.contains("step"),
            "error should mention step, got: {err}"
        );
    }

    #[test]
    fn test_step_wrapper_mode_still_requires_command() {
        let args = Args::try_parse_from(["opr8r", "--ticket-id=FEAT-1", "--step=plan"]).unwrap();
        let err = args.validate_step_wrapper().unwrap_err();
        assert!(
            err.contains("command"),
            "error should mention command, got: {err}"
        );
    }

    #[test]
    fn test_parse_minimal_args() {
        let args = Args::try_parse_from([
            "opr8r",
            "--ticket-id=FEAT-123",
            "--step=plan",
            "--",
            "claude",
            "--prompt",
            "test",
        ])
        .unwrap();

        assert_eq!(args.ticket_id, Some("FEAT-123".to_string()));
        assert_eq!(args.step, Some("plan".to_string()));
        assert!(args.api_url.is_none());
        assert!(!args.no_auto_proceed);
        assert!(!args.verbose);
        assert_eq!(args.command, vec!["claude", "--prompt", "test"]);
        assert!(args.subcommand.is_none());
    }

    #[test]
    fn test_parse_full_args() {
        let args = Args::try_parse_from([
            "opr8r",
            "--ticket-id=FIX-456",
            "--step=build",
            "--api-url=http://localhost:7008",
            "--session-id=abc-123",
            "--no-auto-proceed",
            "--verbose",
            "--",
            "gemini",
            "--model",
            "pro",
        ])
        .unwrap();

        assert_eq!(args.ticket_id, Some("FIX-456".to_string()));
        assert_eq!(args.step, Some("build".to_string()));
        assert_eq!(args.api_url, Some("http://localhost:7008".to_string()));
        assert_eq!(args.session_id, Some("abc-123".to_string()));
        assert!(args.no_auto_proceed);
        assert!(args.verbose);
        assert_eq!(args.command, vec!["gemini", "--model", "pro"]);
    }

    #[test]
    fn test_command_parts() {
        let args = Args::try_parse_from([
            "opr8r",
            "--ticket-id=FEAT-1",
            "--step=plan",
            "--",
            "claude",
            "--prompt",
            "hello",
        ])
        .unwrap();

        let (program, rest) = args.command_parts().unwrap();
        assert_eq!(program, "claude");
        assert_eq!(rest, &["--prompt", "hello"]);
    }

    #[test]
    fn test_missing_required_args() {
        // Missing --ticket-id in step-wrapper mode: parses but validate_step_wrapper rejects
        let args = Args::try_parse_from(["opr8r", "--step=plan", "--", "claude"]).unwrap();
        assert!(args.validate_step_wrapper().is_err());

        // Missing --step in step-wrapper mode
        let args = Args::try_parse_from(["opr8r", "--ticket-id=FEAT-1", "--", "claude"]).unwrap();
        assert!(args.validate_step_wrapper().is_err());

        // Missing command in step-wrapper mode
        let args = Args::try_parse_from(["opr8r", "--ticket-id=FEAT-1", "--step=plan"]).unwrap();
        assert!(args.validate_step_wrapper().is_err());
    }

    #[test]
    fn test_empty_command_parts() {
        let args = Args {
            subcommand: None,
            ticket_id: Some("FEAT-1".to_string()),
            step: Some("plan".to_string()),
            api_url: None,
            session_id: None,
            no_auto_proceed: false,
            verbose: false,
            dry_run: false,
            command: vec![],
        };

        assert!(args.command_parts().is_none());
    }

    #[test]
    fn test_single_command_no_args() {
        let args =
            Args::try_parse_from(["opr8r", "--ticket-id=FEAT-1", "--step=plan", "--", "claude"])
                .unwrap();

        let (program, rest) = args.command_parts().unwrap();
        assert_eq!(program, "claude");
        assert!(rest.is_empty());
    }

    #[test]
    fn test_dry_run_flag() {
        let args = Args::try_parse_from([
            "opr8r",
            "--ticket-id=FEAT-1",
            "--step=plan",
            "--dry-run",
            "--",
            "claude",
        ])
        .unwrap();

        assert!(args.dry_run);
    }

    #[test]
    fn test_validate_step_wrapper_all_present() {
        let args =
            Args::try_parse_from(["opr8r", "--ticket-id=FEAT-1", "--step=plan", "--", "claude"])
                .unwrap();
        assert!(args.validate_step_wrapper().is_ok());
    }
}
