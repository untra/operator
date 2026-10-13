//! Campaign scenarios: fixture directories replayed tick by tick against the runner.
//!
//! A fixture under `tests/fixtures/campaigns/<name>/` holds `campaign.md`,
//! member tickets under `queue/` (and optionally `completed/`), and
//! `scenario.toml` with config overrides, scripted agent and operator actions,
//! and the expected trace. `base = "<fixture>"` inherits another fixture's files.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use anyhow::{anyhow, Result};
use serde::Deserialize;

use operator::campaigns::model::{Campaign, CampaignStatus};
use operator::campaigns::runner::{BlockReason, Slots};
use operator::campaigns::{plan, CampaignStore};
use operator::config::{Config, SessionWrapperType};
use operator::queue::{Queue, Ticket};
use operator::services::campaign_runner::{self, CampaignPort, TickReport};

pub const CAMPAIGN: &str = "CAMP-0001";

pub fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/campaigns")
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overrides {
    pub max_parallel: Option<usize>,
    pub max_agents_per_repo: Option<usize>,
    pub wrapper: Option<SessionWrapperType>,
    /// Premium is revoked from this tick on.
    pub revoke_license_at: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expect {
    #[serde(default)]
    pub waves: Option<Vec<Vec<String>>>,
    pub plan_error: Option<String>,
    pub start_error: Option<String>,
    pub final_status: String,
    pub pause_reason: Option<String>,
    #[serde(default)]
    pub blocked: BTreeMap<String, String>,
    #[serde(default)]
    pub stalled: Vec<String>,
    #[serde(default)]
    pub completed: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Complete,
    Requeue,
    Error,
    Orphan,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentScript {
    pub ticket: String,
    #[serde(default)]
    pub outcomes: Vec<Outcome>,
    #[serde(default = "one")]
    pub ticks: u32,
}

fn one() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorAction {
    pub tick: usize,
    pub complete: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedTick {
    #[serde(default)]
    pub launch: Vec<String>,
    #[serde(default)]
    pub awaiting_operator: Vec<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub base: Option<String>,
    #[serde(default)]
    pub config: Overrides,
    pub expect: Expect,
    #[serde(default)]
    pub agent: Vec<AgentScript>,
    #[serde(default)]
    pub operator: Vec<OperatorAction>,
    #[serde(default)]
    pub tick: Vec<ExpectedTick>,
}

/// A fixture copied into an isolated workspace.
pub struct Scenario {
    pub name: String,
    pub spec: Spec,
    pub config: Config,
    _dir: tempfile::TempDir,
}

fn copy_tree(src: &Path, dst: &Path) {
    for entry in fs::read_dir(src).into_iter().flatten().flatten() {
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            fs::create_dir_all(&target).unwrap();
            copy_tree(&path, &target);
        } else {
            fs::copy(&path, &target).unwrap();
        }
    }
}

fn install(fixture: &Path, tickets: &Path) {
    for folder in ["queue", "completed"] {
        let src = fixture.join(folder);
        if src.is_dir() {
            copy_tree(&src, &tickets.join(folder));
        }
    }
    let campaign = fixture.join("campaign.md");
    if campaign.is_file() {
        fs::copy(campaign, tickets.join("campaigns/CAMP-0001-scenario.md")).unwrap();
    }
}

/// Copy a fixture (and its `base`) into a tickets directory.
pub fn install_fixture(name: &str, spec: &Spec, tickets: &Path) {
    for sub in ["queue", "in-progress", "completed", "campaigns"] {
        fs::create_dir_all(tickets.join(sub)).unwrap();
    }
    if let Some(base) = &spec.base {
        install(&fixtures_root().join(base), tickets);
    }
    install(&fixtures_root().join(name), tickets);
}

pub fn spec(name: &str) -> Spec {
    let raw = fs::read_to_string(fixtures_root().join(name).join("scenario.toml"))
        .unwrap_or_else(|e| panic!("{name}/scenario.toml: {e}"));
    toml::from_str(&raw).unwrap_or_else(|e| panic!("{name}/scenario.toml: {e}"))
}

impl Scenario {
    pub fn load(name: &str) -> Self {
        let spec = spec(name);
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let tickets = root.join("tickets");
        for sub in ["queue", "in-progress", "completed", "campaigns", "operator"] {
            fs::create_dir_all(tickets.join(sub)).unwrap();
        }
        install_fixture(name, &spec, &tickets);

        let mut config = Config::default();
        config.paths.tickets = tickets.to_string_lossy().into_owned();
        config.paths.state = tickets.join("operator").to_string_lossy().into_owned();
        config.paths.projects = root.to_string_lossy().into_owned();
        config.profile.id = uuid::Uuid::new_v4();
        if let Some(n) = spec.config.max_parallel {
            config.agents.max_parallel = n;
        }
        if let Some(n) = spec.config.max_agents_per_repo {
            config.agents.max_agents_per_repo = n;
        }
        if let Some(wrapper) = spec.config.wrapper {
            config.sessions.wrapper = wrapper;
        }
        Self {
            name: name.to_string(),
            spec,
            config,
            _dir: dir,
        }
    }

    pub fn tickets(&self) -> Vec<Ticket> {
        let queue = Queue::new(&self.config).unwrap();
        let mut all = queue.list_queue().unwrap();
        all.extend(queue.list_in_progress().unwrap());
        all.extend(queue.list_completed().unwrap());
        all
    }

    pub fn campaign(&self) -> Campaign {
        CampaignStore::new(&self.config)
            .get(CAMPAIGN)
            .expect("fixture campaign")
    }

    fn folder_of(&self, id: &str) -> Option<String> {
        self.tickets()
            .into_iter()
            .find(|t| t.id == id)
            .and_then(|t| {
                Path::new(&t.filepath)
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().into_owned())
            })
    }
}

struct Running {
    project: String,
    remaining: u32,
    outcome: Outcome,
}

/// Simulated agents: launching claims the ticket; outcomes land as the harness advances.
pub struct SimPort {
    config: Config,
    scripts: BTreeMap<String, AgentScript>,
    attempts: Mutex<BTreeMap<String, usize>>,
    running: Mutex<BTreeMap<String, Running>>,
    entitled: AtomicBool,
}

impl SimPort {
    pub fn new(config: &Config, scripts: &[AgentScript]) -> Self {
        Self {
            config: config.clone(),
            scripts: scripts
                .iter()
                .map(|s| (s.ticket.clone(), s.clone()))
                .collect(),
            attempts: Mutex::default(),
            running: Mutex::default(),
            entitled: AtomicBool::new(true),
        }
    }

    pub fn set_entitled(&self, premium: bool) {
        self.entitled.store(premium, Ordering::SeqCst);
    }

    pub fn running(&self) -> usize {
        self.running.lock().unwrap().len()
    }

    /// One unit of agent time: finished agents complete, requeue or vanish.
    pub fn advance(&self) {
        let queue = Queue::new(&self.config).unwrap();
        let mut running = self.running.lock().unwrap();
        let mut done = Vec::new();
        for (id, agent) in running.iter_mut() {
            agent.remaining = agent.remaining.saturating_sub(1);
            if agent.remaining == 0 {
                done.push((id.clone(), agent.outcome));
            }
        }
        for (id, outcome) in done {
            running.remove(&id);
            let ticket = queue.find_ticket(&id).unwrap().expect("running ticket");
            match outcome {
                Outcome::Complete => queue.complete_ticket(&ticket).unwrap(),
                Outcome::Requeue => queue.return_to_queue(&ticket).unwrap(),
                Outcome::Orphan | Outcome::Error => {}
            }
        }
    }
}

impl SimPort {
    fn start(&self, ticket: &Ticket) -> Result<String> {
        let attempt = {
            let mut attempts = self.attempts.lock().unwrap();
            let n = attempts.entry(ticket.id.clone()).or_default();
            *n += 1;
            *n - 1
        };
        let script = self.scripts.get(&ticket.id);
        let outcome = script
            .and_then(|s| s.outcomes.get(attempt).copied())
            .unwrap_or(Outcome::Complete);
        if outcome == Outcome::Error {
            return Err(anyhow!("simulated provisioning failure"));
        }
        Queue::new(&self.config)?.claim_ticket(ticket)?;
        self.running.lock().unwrap().insert(
            ticket.id.clone(),
            Running {
                project: ticket.project.clone(),
                remaining: script.map_or(1, |s| s.ticks.max(1)),
                outcome,
            },
        );
        Ok(format!("sim-{}-{attempt}", ticket.id))
    }
}

impl CampaignPort for SimPort {
    fn launch(
        &self,
        _campaign: &Campaign,
        ticket: &Ticket,
    ) -> impl std::future::Future<Output = Result<String>> + Send {
        std::future::ready(self.start(ticket))
    }

    fn entitled(&self) -> bool {
        self.entitled.load(Ordering::SeqCst)
    }

    fn live_agents(&self) -> BTreeSet<String> {
        self.running.lock().unwrap().keys().cloned().collect()
    }

    fn slots(&self) -> Slots {
        let running = self.running.lock().unwrap();
        let mut per_repo_busy = BTreeMap::new();
        for agent in running.values() {
            *per_repo_busy.entry(agent.project.clone()).or_default() += 1;
        }
        Slots {
            global: self
                .config
                .agents
                .max_parallel
                .saturating_sub(running.len()),
            per_repo_limit: self.config.agents.max_agents_per_repo.max(1),
            per_repo_busy,
        }
    }
}

fn status_name(status: CampaignStatus) -> String {
    serde_json::to_value(status)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

fn reason_name(reason: &BlockReason) -> String {
    serde_json::to_value(reason).unwrap()["kind"]
        .as_str()
        .unwrap()
        .to_string()
}

fn observed(report: &TickReport) -> ExpectedTick {
    ExpectedTick {
        launch: report.launched.clone(),
        awaiting_operator: report.awaiting_operator.clone(),
        status: status_name(report.status),
    }
}

/// Invariants that hold on every tick of every scenario.
fn check_invariants(
    s: &Scenario,
    before: &[Ticket],
    report: &TickReport,
    port: &SimPort,
    entitled: bool,
) {
    let name = &s.name;
    let done = |id: &str| before.iter().any(|t| t.id == id && plan::is_completed(t));
    for id in &report.launched {
        assert!(entitled, "{name}: launched {id} without entitlement");
        let ticket = before.iter().find(|t| &t.id == id).unwrap();
        assert!(!ticket.is_paired(), "{name}: launched paired ticket {id}");
        for dep in &ticket.depends_on {
            assert!(done(dep), "{name}: launched {id} before {dep} completed");
        }
    }
    let slots = port.slots();
    assert!(
        port.running() <= s.config.agents.max_parallel,
        "{name}: global slots exceeded"
    );
    for (repo, busy) in &slots.per_repo_busy {
        assert!(
            *busy <= slots.per_repo_limit,
            "{name}: repo {repo} over its limit"
        );
    }
    let campaign = s.campaign();
    let policy = campaign.governance_overrides.unwrap_or_default();
    if let Some(cap) = policy.max_parallel {
        let active = s
            .tickets()
            .iter()
            .filter(|t| t.campaign.as_deref() == Some(CAMPAIGN))
            .filter(|t| s.folder_of(&t.id).as_deref() == Some("in-progress"))
            .count();
        assert!(
            active <= cap as usize,
            "{name}: max_parallel {cap} exceeded"
        );
    }
    let state = operator::state::State::load(&s.config).unwrap();
    if let (Some(run), Some(cap)) = (state.campaign_runs.get(CAMPAIGN), policy.max_retries) {
        for (id, n) in &run.launches {
            assert!(*n <= cap + 1, "{name}: {id} launched {n} times");
        }
    }
}

/// Replay one fixture and assert its structure, trace, invariants and outcome.
pub async fn run(name: &str) {
    let s = Scenario::load(name);
    let expect = &s.spec.expect;

    let all = s.tickets();
    let members = operator::campaigns::members(CAMPAIGN, &all);
    assert!(!members.is_empty(), "{name}: no members");
    let planned = plan::plan(&s.config, &members, &all);
    match (&expect.plan_error, &planned) {
        (Some(want), Err(e)) => assert!(e.to_string().contains(want), "{name}: {e}"),
        (Some(want), Ok(p)) => panic!("{name}: expected plan error '{want}', got {p:?}"),
        (None, Err(e)) => panic!("{name}: plan failed: {e}"),
        (None, Ok(p)) => {
            if let Some(waves) = &expect.waves {
                assert_eq!(&p.waves, waves, "{name}: waves");
            }
        }
    }

    let port = SimPort::new(&s.config, &s.spec.agent);
    let started = campaign_runner::start_campaign(&s.config, &port, CAMPAIGN);
    if let Some(want) = &expect.start_error {
        let e = started.expect_err("start must be refused");
        assert!(e.to_string().contains(want), "{name}: {e}");
        assert_eq!(status_name(s.campaign().status), expect.final_status);
        assert_eq!(port.running(), 0, "{name}: refused start launched");
        return;
    }
    started.unwrap_or_else(|e| panic!("{name}: start failed: {e}"));

    let bound = members.len() * 4;
    let mut trace = Vec::new();
    let mut last = None;
    for n in 0..bound {
        if s.spec.config.revoke_license_at.is_some_and(|at| n >= at) {
            port.set_entitled(false);
        }
        let queue = Queue::new(&s.config).unwrap();
        for action in s.spec.operator.iter().filter(|a| a.tick == n) {
            let ticket = queue.find_ticket(&action.complete).unwrap().unwrap();
            queue.complete_ticket(&ticket).unwrap();
        }
        let before = s.tickets();
        let report = campaign_runner::tick(&s.config, &port, CAMPAIGN)
            .await
            .unwrap_or_else(|e| panic!("{name}: tick {n}: {e}"));
        check_invariants(&s, &before, &report, &port, port.entitled());
        trace.push(observed(&report));
        let terminal = matches!(
            report.status,
            CampaignStatus::Completed | CampaignStatus::Paused
        );
        let pending_operator = s.spec.operator.iter().any(|a| a.tick > n);
        let idle = report.launched.is_empty() && port.running() == 0 && !pending_operator;
        last = Some(report);
        if terminal || idle {
            break;
        }
        port.advance();
    }
    let last = last.expect("at least one tick");
    assert!(
        trace.len() < bound,
        "{name}: did not settle within {bound} ticks"
    );

    assert_eq!(trace, s.spec.tick, "{name}: trace");
    assert_eq!(
        status_name(last.status),
        expect.final_status,
        "{name}: final status"
    );
    assert_eq!(
        status_name(s.campaign().status),
        expect.final_status,
        "{name}: stored status"
    );
    if let Some(want) = &expect.pause_reason {
        let got = last.pause_reason.clone().unwrap_or_default();
        assert!(got.contains(want), "{name}: pause reason '{got}'");
    }
    let blocked: BTreeMap<String, String> = last
        .blocked
        .iter()
        .filter(|(id, _)| expect.blocked.contains_key(*id))
        .map(|(id, r)| (id.clone(), reason_name(r)))
        .collect();
    assert_eq!(blocked, expect.blocked, "{name}: blocked");
    assert_eq!(last.stalled, expect.stalled, "{name}: stalled");
    for id in &expect.completed {
        assert_eq!(
            s.folder_of(id).as_deref(),
            Some("completed"),
            "{name}: {id} not completed"
        );
    }
}
