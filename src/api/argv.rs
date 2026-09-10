//! Provider CLI command construction, separated from execution.
//!
//! Each provider builds its arg vector as a pure function so it can be
//! asserted without spawning a binary. The three providers spell the same
//! request very differently -- `gh` uses `--head`/`--base`/`--body`, `glab`
//! uses `--source-branch`/`--target-branch`/`--description`, and `tea` posts
//! JSON -- and nothing caught a bad flag until the argv became testable.

/// A provider CLI invocation: the binary and its arguments, ready to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCommand {
    /// The binary, always sourced from [`crate::api::cli_detection`].
    pub program: &'static str,
    pub args: Vec<String>,
}

impl ProviderCommand {
    pub fn new(program: &'static str, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            program,
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    /// Borrowed args, for handing straight to `Command::args`.
    pub fn arg_refs(&self) -> Vec<&str> {
        self.args.iter().map(String::as_str).collect()
    }

    /// The long flags this invocation passes (`--json`, `--draft`, ...).
    ///
    /// Used to check an argv against the installed CLI's own `--help`, which
    /// is how a flag that the binary does not accept gets caught without
    /// credentials or a live repository.
    pub fn long_flags(&self) -> Vec<&str> {
        self.args
            .iter()
            .map(String::as_str)
            .filter(|a| a.starts_with("--") && a.len() > 2)
            .collect()
    }

    /// The subcommand path before the first flag (`["pr", "create"]`).
    pub fn subcommand(&self) -> Vec<&str> {
        self.args
            .iter()
            .map(String::as_str)
            .take_while(|a| !a.starts_with('-'))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_flags_and_subcommand_split_an_argv() {
        let cmd = ProviderCommand::new(
            "gh",
            ["pr", "create", "--repo", "o/r", "--draft", "-q", ".number"],
        );
        assert_eq!(cmd.subcommand(), ["pr", "create"]);
        assert_eq!(cmd.long_flags(), ["--repo", "--draft"]);
    }

    #[test]
    fn a_bare_double_dash_is_not_a_flag() {
        let cmd = ProviderCommand::new("gh", ["pr", "list", "--"]);
        assert!(cmd.long_flags().is_empty());
    }
}
