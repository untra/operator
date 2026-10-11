//! Campaign scenarios through the real launcher: tmux sessions running the mock LLM.
//!
//! Replays fixtures from `tests/fixtures/campaigns/` with `LauncherPort`. The
//! test plays the agents: once a ticket's session appears it kills the session
//! and completes the ticket, then ticks again.
//!
//! ```bash
//! OPERATOR_LAUNCH_TEST_ENABLED=true cargo test --test campaign_launch_integration -- --test-threads=1 --nocapture
//! ```

mod campaign_common;
mod launch_common;

use std::collections::BTreeSet;
use std::fs;
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::Result;

use campaign_common::{ExpectedTick, CAMPAIGN};
use launch_common::{LaunchTestContext, WrapperTestMode};
use operator::campaigns::model::{Campaign, CampaignStatus};
use operator::campaigns::runner::Slots;
use operator::config::Config;
use operator::queue::{Queue, Ticket};
use operator::services::campaign_runner::{self, CampaignPort, LauncherPort};
use operator::state::State;

fn tmux_available() -> bool {
    Command::new("tmux")
        .arg("-V")
        .output()
        .is_ok_and(|o| o.status.success())
}

fn sessions() -> Vec<String> {
    Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|s| s.starts_with("op-"))
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn wait_for_session(ticket_id: &str) -> Option<String> {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(15) {
        if let Some(s) = sessions().into_iter().find(|s| s.contains(ticket_id)) {
            return Some(s);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    None
}

fn kill(session: &str) {
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", session])
        .output();
}

/// The real port, licensed for the test.
struct Licensed(LauncherPort);

impl CampaignPort for Licensed {
    fn launch(
        &self,
        campaign: &Campaign,
        ticket: &Ticket,
    ) -> impl std::future::Future<Output = Result<String>> + Send {
        self.0.launch(campaign, ticket)
    }

    fn entitled(&self) -> bool {
        true
    }

    fn live_agents(&self) -> BTreeSet<String> {
        self.0.live_agents()
    }

    fn slots(&self) -> Slots {
        self.0.slots()
    }
}

struct Run {
    ctx: LaunchTestContext,
    config: Config,
    launched: Vec<String>,
}

impl Drop for Run {
    fn drop(&mut self) {
        for id in &self.launched {
            for s in sessions().into_iter().filter(|s| s.contains(id.as_str())) {
                kill(&s);
            }
        }
    }
}

impl Run {
    fn new(name: &str) -> Self {
        let ctx = LaunchTestContext::new(&format!("campaign_{name}"), WrapperTestMode::Tmux);
        for project in ["api", "web"] {
            let dir = ctx.projects_path.join(project);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("CLAUDE.md"), format!("# {project}\n")).unwrap();
        }
        let spec = campaign_common::spec(name);
        campaign_common::install_fixture(name, &spec, &ctx.tickets_path);
        let mut config = Config::load(Some(ctx.config_path.to_str().unwrap())).unwrap();
        if let Some(n) = spec.config.max_parallel {
            config.agents.max_parallel = n;
        }
        if let Some(n) = spec.config.max_agents_per_repo {
            config.agents.max_agents_per_repo = n;
        }
        std::env::set_var("MOCK_LLM_OUTPUT_DIR", ctx.output_dir.path());
        Self {
            ctx,
            config,
            launched: Vec::new(),
        }
    }

    fn wait_for_invocations(&self, n: usize) {
        let start = Instant::now();
        while self.ctx.get_invocations().len() < n && start.elapsed() < Duration::from_secs(15) {
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Play the agent: its session started, so end it and complete the ticket.
    fn finish(&self, ticket_id: &str) {
        let session = wait_for_session(ticket_id)
            .unwrap_or_else(|| panic!("no tmux session for {ticket_id}: {:?}", sessions()));
        kill(&session);
        let queue = Queue::new(&self.config).unwrap();
        let ticket = queue.find_ticket(ticket_id).unwrap().unwrap();
        queue.complete_ticket(&ticket).unwrap();
        State::mutate(&self.config, |s| {
            s.agents.retain(|a| a.ticket_id != ticket_id);
        })
        .unwrap();
    }
}

async fn replay(name: &str) {
    let spec = campaign_common::spec(name);
    let mut run = Run::new(name);
    let port = Licensed(LauncherPort::new(run.config.clone()));
    campaign_runner::start_campaign(&run.config, &port, CAMPAIGN).expect("start");

    let mut trace = Vec::new();
    for n in 0..spec.tick.len() + 2 {
        let queue = Queue::new(&run.config).unwrap();
        for action in spec.operator.iter().filter(|a| a.tick == n) {
            let ticket = queue.find_ticket(&action.complete).unwrap().unwrap();
            queue.complete_ticket(&ticket).unwrap();
        }
        let report = campaign_runner::tick(&run.config, &port, CAMPAIGN)
            .await
            .unwrap_or_else(|e| panic!("{name}: tick {n}: {e}"));
        let live = sessions()
            .iter()
            .filter(|s| run.launched.iter().any(|id| s.contains(id.as_str())))
            .count();
        assert!(
            live + report.launched.len() <= run.config.agents.max_parallel,
            "{name}: more sessions than slots"
        );
        trace.push(ExpectedTick {
            launch: report.launched.clone(),
            awaiting_operator: report.awaiting_operator.clone(),
            status: serde_json::to_value(report.status)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        });
        run.launched.extend(report.launched.iter().cloned());
        run.wait_for_invocations(run.launched.len());
        for id in &report.launched {
            run.finish(id);
        }
        if report.status == CampaignStatus::Completed {
            break;
        }
    }
    assert_eq!(trace, spec.tick, "{name}: trace");
    let invocations = run.ctx.get_invocations().len();
    assert_eq!(
        invocations,
        run.launched.len(),
        "{name}: one mock LLM invocation per launch"
    );
}

macro_rules! skip_unless_enabled {
    () => {
        if !launch_common::launch_tests_enabled() || !tmux_available() {
            eprintln!("Skipping: needs OPERATOR_LAUNCH_TEST_ENABLED=true and tmux");
            return;
        }
    };
}

#[tokio::test]
async fn diamond_launches_waves_through_tmux() {
    skip_unless_enabled!();
    Box::pin(replay("diamond")).await;
}

#[tokio::test]
async fn gated_chain_launches_through_tmux() {
    skip_unless_enabled!();
    Box::pin(replay("gated-chain")).await;
}
