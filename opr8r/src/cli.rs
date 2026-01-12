use clap::Parser;

/// Minimal CLI wrapper for LLM commands in multi-step ticket workflows.
///
/// Wraps LLM commands (claude, gemini, codex), passes through output,
/// and orchestrates step transitions via the Operator API.
#[derive(Parser, Debug)]
#[command(name = "opr8r")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Ticket ID being worked on (e.g., FEAT-123)
    #[arg(long, required = true)]
    pub ticket_id: String,

    /// Current step name (e.g., "plan", "build", "test")
    #[arg(long, required = true)]
    pub step: String,

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

    /// The LLM command and its arguments to execute
    #[arg(last = true, required = true)]
    pub command: Vec<String>,
}

impl Args {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Args::parse()
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

        assert_eq!(args.ticket_id, "FEAT-123");
        assert_eq!(args.step, "plan");
        assert!(args.api_url.is_none());
        assert!(!args.no_auto_proceed);
        assert!(!args.verbose);
        assert_eq!(args.command, vec!["claude", "--prompt", "test"]);
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

        assert_eq!(args.ticket_id, "FIX-456");
        assert_eq!(args.step, "build");
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
        // Missing --ticket-id
        let result = Args::try_parse_from(["opr8r", "--step=plan", "--", "claude"]);
        assert!(result.is_err());

        // Missing --step
        let result = Args::try_parse_from(["opr8r", "--ticket-id=FEAT-1", "--", "claude"]);
        assert!(result.is_err());

        // Missing command
        let result = Args::try_parse_from(["opr8r", "--ticket-id=FEAT-1", "--step=plan", "--"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_command_parts() {
        // Create args manually with empty command vec
        let args = Args {
            ticket_id: "FEAT-1".to_string(),
            step: "plan".to_string(),
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
}
