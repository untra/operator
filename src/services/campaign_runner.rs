//! Drive running campaigns: plan, decide, launch, record. One owner: the API server.

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;
use ts_rs::TS;

use crate::agents::delegator_resolution::resolve_launch_options;
use crate::agents::Launcher;
use crate::campaigns::model::{Campaign, CampaignStatus};
use crate::campaigns::runner::{
    next_launches, BlockReason, RunTransition, Slots, TickDecision, TickInput,
};
use crate::campaigns::{members, plan, workspace, CampaignStore};
use crate::config::{governance, Config, GovernancePolicy, SessionWrapperType};
use crate::licensing::{self, PremiumFeature};
use crate::queue::{Queue, Ticket};
use crate::startup::templates::load_registry;
use crate::state::State;

/// The runner's view of the outside world.
pub trait CampaignPort: Sync {
    /// Launch one ticket; returns the agent id.
    fn launch(
        &self,
        campaign: &Campaign,
        ticket: &Ticket,
    ) -> impl Future<Output = Result<String>> + Send;
    fn entitled(&self) -> bool;
    /// Ticket ids with a live agent.
    fn live_agents(&self) -> BTreeSet<String>;
    fn slots(&self) -> Slots;
}

/// What one tick did, or would do when assessed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TickReport {
    pub status: CampaignStatus,
    pub launched: Vec<String>,
    pub awaiting_operator: Vec<String>,
    pub stalled: Vec<String>,
    pub blocked: BTreeMap<String, BlockReason>,
    pub pause_reason: Option<String>,
}

impl TickReport {
    fn idle(status: CampaignStatus) -> Self {
        Self {
            status,
            launched: Vec::new(),
            awaiting_operator: Vec::new(),
            stalled: Vec::new(),
            blocked: BTreeMap::new(),
            pause_reason: None,
        }
    }
}

/// Refuse wrappers whose sessions need a connected client.
pub fn validate_campaign_launchable(config: &Config) -> Result<()> {
    if config.sessions.wrapper == SessionWrapperType::Vscode {
        bail!("campaigns launch unattended; the vscode wrapper needs a connected client");
    }
    Ok(())
}

fn all_tickets(config: &Config) -> Result<Vec<Ticket>> {
    let queue = Queue::new(config)?;
    let mut all = queue.list_queue()?;
    all.extend(queue.list_in_progress()?);
    all.extend(queue.list_completed()?);
    Ok(all)
}

fn campaign_policy(config: &Config, campaign: &Campaign) -> Result<GovernancePolicy> {
    crate::campaigns::service::policy(config, campaign)
}

/// Why a campaign cannot start.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StartError {
    #[error("Campaigns requires a valid Premium license for this configuration")]
    NotEntitled,
    /// The campaign's lifecycle forbids starting (running, completed, cancelled).
    #[error("{0}")]
    Lifecycle(String),
    /// The configuration or plan cannot run (wrapper, policy, cycle).
    #[error("{0}")]
    Invalid(String),
}

/// Every reason `campaign` cannot start now; empty means it can.
pub fn preflight(
    config: &Config,
    port: &impl CampaignPort,
    campaign: &Campaign,
) -> Vec<StartError> {
    let id = &campaign.id;
    let mut problems = Vec::new();
    match campaign.status {
        CampaignStatus::Draft | CampaignStatus::Ready | CampaignStatus::Paused => {}
        CampaignStatus::Running => {
            problems.push(StartError::Lifecycle(format!(
                "campaign {id} is already running"
            )));
        }
        CampaignStatus::Completed => {
            problems.push(StartError::Lifecycle(format!("campaign {id} is completed")));
        }
        CampaignStatus::Cancelled => {
            problems.push(StartError::Lifecycle(format!("campaign {id} is cancelled")));
        }
    }
    if !port.entitled() {
        problems.push(StartError::NotEntitled);
    }
    if let Err(e) = validate_campaign_launchable(config) {
        problems.push(StartError::Invalid(e.to_string()));
    }
    if let Err(e) = campaign_policy(config, campaign) {
        problems.push(StartError::Invalid(e.to_string()));
    }
    match all_tickets(config) {
        Ok(all) => {
            if let Err(e) = plan::plan(config, &members(id, &all), &all) {
                problems.push(StartError::Invalid(e.to_string()));
            }
        }
        Err(e) => problems.push(StartError::Invalid(e.to_string())),
    }
    problems
}

/// Move a draft, ready or paused campaign to running after checking it can run unattended.
pub fn start_campaign(config: &Config, port: &impl CampaignPort, id: &str) -> Result<Campaign> {
    let store = CampaignStore::new(config);
    let campaign = store
        .get(id)
        .with_context(|| format!("campaign {id} not found"))?;
    if let Some(problem) = preflight(config, port, &campaign).into_iter().next() {
        return Err(match problem {
            StartError::NotEntitled => licensing::NotEntitled {
                feature: PremiumFeature::Campaigns,
            }
            .into(),
            other => other.into(),
        });
    }
    State::mutate(config, |state| {
        let run = state.campaign_runs.entry(id.to_string()).or_default();
        run.started_at.get_or_insert_with(chrono::Utc::now);
        run.pause_reason = None;
    })?;
    store.set_status(id, CampaignStatus::Running)
}

/// Stop new launches; agents already running are untouched.
pub fn pause_campaign(config: &Config, id: &str, reason: &str) -> Result<TickReport> {
    CampaignStore::new(config)
        .get(id)
        .with_context(|| format!("campaign {id} not found"))?;
    pause(config, id, reason.to_string())
}

fn pause(config: &Config, id: &str, reason: String) -> Result<TickReport> {
    State::mutate(config, |state| {
        let run = state.campaign_runs.entry(id.to_string()).or_default();
        run.pause_reason = Some(reason.clone());
    })?;
    CampaignStore::new(config).set_status(id, CampaignStatus::Paused)?;
    Ok(TickReport {
        pause_reason: Some(reason),
        ..TickReport::idle(CampaignStatus::Paused)
    })
}

struct Ready {
    campaign: Campaign,
    members: Vec<Ticket>,
    decision: TickDecision,
}

/// `Ok(Err(reason))` when the run cannot proceed: unknown policy or a dependency cycle.
fn decide(
    config: &Config,
    port: &impl CampaignPort,
    campaign: Campaign,
) -> Result<Result<Ready, String>> {
    let id = campaign.id.clone();
    let policy = match campaign_policy(config, &campaign) {
        Ok(policy) => policy,
        Err(e) => return Ok(Err(e.to_string())),
    };
    let all = all_tickets(config)?;
    let member_list = members(&id, &all);
    let planned = match plan::plan(config, &member_list, &all) {
        Ok(planned) => planned,
        Err(e) => return Ok(Err(e.to_string())),
    };
    let run = State::load(config)?
        .campaign_runs
        .get(&id)
        .cloned()
        .unwrap_or_default();
    let registry = load_registry(&config.tickets_path());
    let is_paired =
        |t: &Ticket| crate::templates::is_paired_with_registry(&t.ticket_type, &registry);
    let decision = next_launches(&TickInput {
        plan: &planned,
        members: &member_list,
        all: &all,
        run: &run,
        policy: &policy,
        slots: port.slots(),
        entitled: port.entitled(),
        live: &port.live_agents(),
        is_paired: &is_paired,
    });
    let members = member_list.into_iter().cloned().collect();
    Ok(Ok(Ready {
        campaign,
        members,
        decision,
    }))
}

/// What the next tick would do, without launching or writing; `launched` lists would-be launches.
pub fn assess(config: &Config, port: &impl CampaignPort, id: &str) -> Result<TickReport> {
    let campaign = CampaignStore::new(config)
        .get(id)
        .with_context(|| format!("campaign {id} not found"))?;
    let status = campaign.status;
    Ok(match decide(config, port, campaign)? {
        Err(reason) => TickReport {
            pause_reason: Some(reason),
            ..TickReport::idle(status)
        },
        Ok(Ready { decision, .. }) => TickReport {
            status,
            pause_reason: match decision.transition {
                Some(RunTransition::Pause(reason)) => Some(reason),
                _ => None,
            },
            launched: decision.launch,
            awaiting_operator: decision.awaiting_operator,
            stalled: decision.stalled,
            blocked: decision.blocked,
        },
    })
}

/// Advance one running campaign by one step.
pub async fn tick(config: &Config, port: &impl CampaignPort, id: &str) -> Result<TickReport> {
    let store = CampaignStore::new(config);
    let campaign = store
        .get(id)
        .with_context(|| format!("campaign {id} not found"))?;
    if campaign.status != CampaignStatus::Running {
        return Ok(TickReport::idle(campaign.status));
    }
    let (campaign, member_list, decision) = match decide(config, port, campaign)? {
        Err(reason) => return pause(config, id, reason),
        Ok(Ready {
            campaign,
            members,
            decision,
        }) => (campaign, members, decision),
    };

    let mut transition = decision.transition.clone();
    let mut launched = Vec::new();
    for ticket_id in &decision.launch {
        let Some(ticket) = member_list.iter().find(|t| &t.id == ticket_id) else {
            continue;
        };
        match port.launch(&campaign, ticket).await {
            Ok(_) => launched.push(ticket_id.clone()),
            Err(e) => {
                transition = Some(RunTransition::Pause(format!(
                    "launch of {ticket_id} failed: {e}"
                )));
                break;
            }
        }
    }

    let pause_reason = match &transition {
        Some(RunTransition::Pause(reason)) => Some(reason.clone()),
        _ => None,
    };
    State::mutate(config, |state| {
        let run = state.campaign_runs.entry(id.to_string()).or_default();
        for ticket_id in &launched {
            run.record_launch(ticket_id);
        }
        if let Some(reason) = &pause_reason {
            run.pause_reason = Some(reason.clone());
            run.last_error = Some(reason.clone());
        }
    })?;
    let status = match transition {
        Some(RunTransition::Complete) => CampaignStatus::Completed,
        Some(RunTransition::Pause(_)) => CampaignStatus::Paused,
        None => CampaignStatus::Running,
    };
    if status != CampaignStatus::Running {
        store.set_status(id, status)?;
    }
    Ok(TickReport {
        status,
        launched,
        awaiting_operator: decision.awaiting_operator,
        stalled: decision.stalled,
        blocked: decision.blocked,
        pause_reason,
    })
}

/// Production port: the server-side launcher, the licence and `state.json`.
pub struct LauncherPort {
    config: Config,
}

impl LauncherPort {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

/// Free capacity from the configured limits and the agents already running.
pub fn slots_from(config: &Config, state: &State) -> Slots {
    let running = state.running_agents();
    let mut per_repo_busy = BTreeMap::new();
    for agent in &running {
        *per_repo_busy.entry(agent.project.clone()).or_default() += 1;
    }
    Slots {
        global: config.effective_max_agents().saturating_sub(running.len()),
        per_repo_limit: config.effective_max_agents_per_repo(),
        per_repo_busy,
    }
}

impl CampaignPort for LauncherPort {
    async fn launch(&self, campaign: &Campaign, ticket: &Ticket) -> Result<String> {
        let mut options =
            resolve_launch_options(&self.config, None, None, None, None, false, None)?;
        if let Some(shared) = workspace::resolve_workspace(&self.config, campaign) {
            options.target = shared.target;
        }
        Launcher::new(&self.config)?
            .launch_with_options(ticket, options)
            .await
    }

    fn entitled(&self) -> bool {
        licensing::entitlements(&self.config).allows(PremiumFeature::Campaigns)
    }

    fn live_agents(&self) -> BTreeSet<String> {
        State::load(&self.config)
            .map(|s| {
                s.running_agents()
                    .iter()
                    .map(|a| a.ticket_id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn slots(&self) -> Slots {
        State::load(&self.config)
            .map(|s| slots_from(&self.config, &s))
            .unwrap_or_default()
    }
}

/// Wakes the runner when a ticket changes column.
pub fn wake() -> Arc<Notify> {
    static WAKE: std::sync::OnceLock<Arc<Notify>> = std::sync::OnceLock::new();
    Arc::clone(WAKE.get_or_init(|| Arc::new(Notify::new())))
}

const TICK_INTERVAL: Duration = Duration::from_secs(30);

/// Tick every running campaign on an interval and whenever a ticket moves.
pub fn spawn_campaign_runner(config: impl Fn() -> Arc<Config> + Send + 'static) {
    tokio::spawn(async move {
        let notify = wake();
        loop {
            let config = config();
            let port = LauncherPort::new((*config).clone());
            for campaign in CampaignStore::new(&config).list() {
                if campaign.status != CampaignStatus::Running {
                    continue;
                }
                if let Err(e) = tick(&config, &port, &campaign.id).await {
                    tracing::warn!(campaign = %campaign.id, error = %e, "campaign tick failed");
                }
            }
            tokio::select! {
                () = notify.notified() => {}
                () = tokio::time::sleep(TICK_INTERVAL) => {}
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AgentState;

    fn agent(project: &str, status: &str) -> AgentState {
        let mut a: AgentState = serde_json::from_value(serde_json::json!({
            "id": "a", "ticket_id": "T-1", "ticket_type": "FEAT", "project": project,
            "status": status, "started_at": "2026-01-01T00:00:00Z",
            "last_activity": "2026-01-01T00:00:00Z", "paired": false
        }))
        .unwrap();
        a.project = project.to_string();
        a
    }

    #[test]
    fn test_slots_subtract_running_agents_per_repo() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = dir.path().to_string_lossy().into_owned();
        config.agents.max_agents_per_repo = 2;
        let mut state = State::load(&config).unwrap();
        state.agents = vec![
            agent("api", "running"),
            agent("api", "running"),
            agent("web", "completed"),
        ];
        let slots = slots_from(&config, &state);
        assert_eq!(slots.per_repo_busy.get("api"), Some(&2));
        assert_eq!(slots.per_repo_busy.get("web"), None);
        assert_eq!(slots.per_repo_limit, 2);
        assert_eq!(
            slots.global,
            config.effective_max_agents().saturating_sub(2)
        );
    }

    #[test]
    fn test_vscode_wrapper_is_refused() {
        let mut config = Config::default();
        assert!(validate_campaign_launchable(&config).is_ok());
        config.sessions.wrapper = SessionWrapperType::Vscode;
        let err = validate_campaign_launchable(&config).unwrap_err();
        assert!(err.to_string().contains("client"));
    }
}
