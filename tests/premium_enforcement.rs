//! Remote execution is gated *before* anything is provisioned.
//!
//! The gate matters less for what it returns than for what it prevents: a
//! refused launch must leave no claimed ticket, no agent row, no worktree and
//! no session behind. These tests drive the real `Launcher` against a temporary
//! workspace and assert on the filesystem and state store afterwards.

use std::path::{Path, PathBuf};

use operator::agents::{LaunchOptions, Launcher, RelaunchOptions};
use operator::config::{Config, SshTarget, TargetDef, TargetKind};
use operator::queue::Ticket;

const TICKET: &str = r"---
id: TASK-001
priority: P2-medium
status: queued
---

# Task: Verify the entitlement gate

## Context
Launched only if the configuration is entitled to the resolved target.
";

struct Workspace {
    config: Config,
    tickets: PathBuf,
    _directory: tempfile::TempDir,
}

impl Workspace {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let tickets = root.join("tickets");
        for sub in ["queue", "in-progress", "done", "operator"] {
            std::fs::create_dir_all(tickets.join(sub)).unwrap();
        }
        let project = root.join("testproject");
        std::fs::create_dir_all(&project).unwrap();

        let mut config = Config::default();
        config.paths.tickets = tickets.to_string_lossy().into_owned();
        config.paths.state = tickets.join("operator").to_string_lossy().into_owned();
        config.paths.projects = root.to_string_lossy().into_owned();
        config.paths.worktrees = root.join("worktrees").to_string_lossy().into_owned();
        config.projects = vec!["testproject".to_string()];
        config.profile.id = uuid::Uuid::new_v4();

        let filename = format!(
            "{}-TASK-testproject-task_001.md",
            chrono::Local::now().format("%Y%m%d-%H%M")
        );
        std::fs::write(tickets.join("queue").join(&filename), TICKET).unwrap();

        Self {
            config,
            tickets,
            _directory: directory,
        }
    }

    fn ticket(&self) -> Ticket {
        let queue = self.tickets.join("queue");
        let entry = std::fs::read_dir(&queue)
            .unwrap()
            .filter_map(Result::ok)
            .find(|e| e.path().extension().is_some_and(|x| x == "md"))
            .expect("the queued ticket");
        Ticket::from_file(&entry.path()).expect("ticket parses")
    }

    fn queued(&self) -> usize {
        count_markdown(&self.tickets.join("queue"))
    }

    fn in_progress(&self) -> usize {
        count_markdown(&self.tickets.join("in-progress"))
    }

    fn agents(&self) -> usize {
        operator::state::State::load(&self.config)
            .map(|state| state.agents.len())
            .unwrap_or(0)
    }

    fn worktrees_exist(&self) -> bool {
        Path::new(&self.config.worktrees_path())
            .read_dir()
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false)
    }
}

fn count_markdown(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
                .count()
        })
        .unwrap_or(0)
}

fn ssh_target() -> TargetDef {
    TargetDef {
        name: "build-host".to_string(),
        display_name: None,
        kind: TargetKind::Ssh(SshTarget {
            ssh_alias: "build".to_string(),
            workdir: "/srv/work".to_string(),
            ssh_config_path: None,
        }),
    }
}

fn options_for(target: TargetDef) -> LaunchOptions {
    LaunchOptions {
        target,
        ..LaunchOptions::default()
    }
}

/// The headline claim: refused, and nothing happened.
#[tokio::test]
async fn an_unlicensed_ssh_launch_is_blocked_before_any_side_effect() {
    let workspace = Workspace::new();
    let launcher = Launcher::new(&workspace.config).expect("launcher");
    let ticket = workspace.ticket();

    let error = launcher
        .prepare_launch(&ticket, options_for(ssh_target()))
        .await
        .expect_err("an unlicensed SSH launch must be refused");

    assert!(
        error.to_string().contains("Premium"),
        "unexpected refusal: {error}"
    );
    assert_eq!(workspace.queued(), 1, "the ticket must stay in the queue");
    assert_eq!(workspace.in_progress(), 0, "the ticket must not be claimed");
    assert_eq!(workspace.agents(), 0, "no agent may be recorded");
    assert!(!workspace.worktrees_exist(), "no worktree may be created");
}

/// Coder is the other premium target, and the one that provisions a remote
/// workspace - so the gate has to fire before the provisioning call, not after.
#[tokio::test]
async fn an_unlicensed_coder_launch_is_blocked_before_provisioning() {
    let workspace = Workspace::new();
    let launcher = Launcher::new(&workspace.config).expect("launcher");
    let ticket = workspace.ticket();
    let target = TargetDef {
        name: "coder-agents".to_string(),
        display_name: None,
        kind: TargetKind::Coder(operator::config::CoderConfig::default()),
    };

    let error = launcher
        .prepare_launch(&ticket, options_for(target))
        .await
        .expect_err("an unlicensed Coder launch must be refused");

    assert!(error.to_string().contains("Premium"), "{error}");
    assert_eq!(workspace.queued(), 1);
    assert_eq!(workspace.agents(), 0);
}

/// `relaunch` recovers a dead session, and is a second way into remote
/// execution. It is gated identically.
#[tokio::test]
async fn an_unlicensed_remote_relaunch_is_refused() {
    let workspace = Workspace::new();
    let launcher = Launcher::new(&workspace.config).expect("launcher");
    let ticket = workspace.ticket();
    let options = RelaunchOptions {
        launch_options: options_for(ssh_target()),
        ..RelaunchOptions::default()
    };

    let error = launcher
        .prepare_relaunch(&ticket, options)
        .await
        .expect_err("an unlicensed relaunch must be refused");

    assert!(error.to_string().contains("Premium"), "{error}");
    assert_eq!(workspace.queued(), 1);
    assert_eq!(workspace.agents(), 0);
}

/// The deprecated `[[hosts]]` path synthesises an SSH target, so it must be
/// gated exactly like a declared one - a legacy config is not a bypass.
#[test]
fn a_legacy_hosts_entry_resolves_to_a_gated_target() {
    let workspace = Workspace::new();
    let target = operator::agents::delegator_resolution::resolve_named_target(
        &workspace.config,
        "build-host",
    );

    // Unknown until declared; once declared as a host it is an SSH target.
    assert!(target.is_err(), "an undeclared target must not resolve");

    let mut config = workspace.config;
    config.hosts.push(operator::config::RemoteHost {
        name: "legacy-box".to_string(),
        ssh_alias: "legacy".to_string(),
        workdir: "/srv/legacy".to_string(),
        display_name: None,
        ssh_config_path: None,
    });

    let resolved =
        operator::agents::delegator_resolution::resolve_named_target(&config, "legacy-box")
            .expect("a declared host resolves");

    assert!(
        matches!(resolved.kind, TargetKind::Ssh(_)),
        "a legacy host is an SSH target"
    );
    assert!(
        operator::licensing::require_target(&config, &resolved).is_err(),
        "the legacy path must be gated like any other remote target"
    );
}

/// Local and container execution stay free. This is the other half of the
/// contract and the one a regression would quietly break.
#[test]
fn local_and_container_execution_need_no_licence() {
    let workspace = Workspace::new();

    for target in [
        TargetDef::local(),
        TargetDef {
            name: "docker".to_string(),
            display_name: None,
            kind: TargetKind::Docker(operator::config::DockerConfig::default()),
        },
    ] {
        assert!(
            operator::licensing::require_target(&workspace.config, &target).is_ok(),
            "{} must not require a licence",
            target.name
        );
    }
}
