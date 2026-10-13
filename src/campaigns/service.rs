//! Campaign operations shared by REST, MCP and the CLI. Callers gate on the Premium licence first.

use anyhow::{bail, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use super::model::{Campaign, CampaignStatus, CampaignWorkspace};
use super::{members, plan, CampaignStore, NewCampaign};
use crate::config::{governance, Config, GovernancePolicy};
use crate::queue::{Queue, Ticket};
use crate::services::campaign_runner::{self, CampaignPort, TickReport};
use crate::state::State;

/// Fields for a new campaign.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct NewCampaignInput {
    pub title: String,
    /// Markdown goal written as the campaign body.
    #[serde(default)]
    pub goal: Option<String>,
    /// Name of a `[[governance]]` policy.
    #[serde(default)]
    pub governance: Option<String>,
    /// Inline policy layered over `governance`.
    #[serde(default)]
    pub governance_overrides: Option<GovernancePolicy>,
    #[serde(default)]
    pub workspace: Option<CampaignWorkspace>,
}

/// A member ticket as seen from its campaign.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CampaignMember {
    pub id: String,
    pub summary: String,
    pub project: String,
    /// `queue`, `in-progress` or `completed`.
    pub column: String,
    pub depends_on: Vec<String>,
    /// Launches made by the runner.
    pub launches: u32,
}

/// A campaign with its members and plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CampaignDetail {
    pub campaign: Campaign,
    pub members: Vec<CampaignMember>,
    pub waves: Vec<Vec<String>>,
    /// Member id → local tickets outside the campaign it waits on.
    pub gates: std::collections::BTreeMap<String, Vec<String>>,
    /// Member id → unsynced upstream refs it waits on.
    pub unsynced: std::collections::BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_error: Option<String>,
}

/// Usage counted against the campaign's caps.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS,
)]
#[ts(export)]
pub struct CampaignCounters {
    pub launched: u32,
    pub max_launches: Option<u32>,
    /// Members in progress.
    pub active: u32,
    pub max_parallel: Option<u32>,
    /// Most relaunches of any one member.
    pub retries: u32,
    pub max_retries: Option<u32>,
}

/// Live state of a campaign run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CampaignStatusView {
    pub id: String,
    pub status: CampaignStatus,
    pub tickets: Vec<CampaignMember>,
    pub waves: Vec<Vec<String>>,
    /// What the next tick would do.
    pub report: TickReport,
    pub counters: CampaignCounters,
    /// The policy the campaign layers over each delegator's own.
    pub policy: GovernancePolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause_reason: Option<String>,
}

/// Result of a launch or dry run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CampaignLaunchOutcome {
    pub id: String,
    pub dry_run: bool,
    pub status: CampaignStatus,
    pub waves: Vec<Vec<String>>,
    pub policy: GovernancePolicy,
    /// Reasons the campaign cannot start; empty when it can.
    pub violations: Vec<String>,
    /// The first tick, run or (for a dry run) assessed.
    pub report: TickReport,
}

fn all_tickets(config: &Config) -> Result<Vec<Ticket>> {
    let queue = Queue::new(config)?;
    let mut all = queue.list_queue()?;
    all.extend(queue.list_in_progress()?);
    all.extend(queue.list_completed()?);
    Ok(all)
}

fn get(config: &Config, id: &str) -> Result<Campaign> {
    CampaignStore::new(config)
        .get(id)
        .with_context(|| format!("campaign {id} not found"))
}

/// The campaign's own layered policy.
pub fn policy(config: &Config, campaign: &Campaign) -> Result<GovernancePolicy> {
    let mut layers: Vec<&GovernancePolicy> = Vec::new();
    if let Some(name) = &campaign.governance {
        layers.push(governance::find_policy(&config.governance, name)?);
    }
    if let Some(overrides) = &campaign.governance_overrides {
        layers.push(overrides);
    }
    Ok(governance::layer(&layers))
}

pub fn create_campaign(config: &Config, input: NewCampaignInput) -> Result<Campaign> {
    if let Some(name) = &input.governance {
        governance::find_policy(&config.governance, name)?;
    }
    CampaignStore::new(config).create(NewCampaign {
        title: input.title,
        goal: input.goal,
        governance: input.governance,
        governance_overrides: input.governance_overrides,
        workspace: input.workspace,
        external_ref: None,
    })
}

/// Move a ticket into (or out of) a campaign; an edit that creates a cycle is reverted.
pub fn assign_ticket(
    config: &Config,
    ticket_id: &str,
    campaign: Option<&str>,
    depends_on: Option<Vec<String>>,
) -> Result<Ticket> {
    if let Some(id) = campaign {
        get(config, id)?;
    }
    let queue = Queue::new(config)?;
    let mut ticket = queue
        .find_ticket(ticket_id)?
        .with_context(|| format!("ticket {ticket_id} not found"))?;
    let before = (ticket.campaign.clone(), ticket.depends_on.clone());
    ticket.set_campaign(campaign)?;
    if let Some(deps) = &depends_on {
        ticket.set_depends_on(deps)?;
    }
    if let Some(id) = campaign {
        let all = all_tickets(config)?;
        if let Err(e) = plan::plan(config, &members(id, &all), &all) {
            ticket.set_campaign(before.0.as_deref())?;
            ticket.set_depends_on(&before.1)?;
            bail!(e);
        }
    }
    Ok(ticket)
}

fn member_views(config: &Config, id: &str, all: &[Ticket]) -> Result<Vec<CampaignMember>> {
    let run = State::load(config)?
        .campaign_runs
        .remove(id)
        .unwrap_or_default();
    Ok(members(id, all)
        .into_iter()
        .map(|t| CampaignMember {
            id: t.id.clone(),
            summary: t.summary.clone(),
            project: t.project.clone(),
            column: t.column(),
            depends_on: t.depends_on.clone(),
            launches: run.launches.get(&t.id).copied().unwrap_or(0),
        })
        .collect())
}

pub fn detail(config: &Config, id: &str) -> Result<CampaignDetail> {
    let campaign = get(config, id)?;
    let all = all_tickets(config)?;
    let members_view = member_views(config, id, &all)?;
    let (planned, plan_error) = match plan::plan(config, &members(id, &all), &all) {
        Ok(planned) => (planned, None),
        Err(e) => (plan::CampaignPlan::default(), Some(e.to_string())),
    };
    Ok(CampaignDetail {
        campaign,
        members: members_view,
        waves: planned.waves,
        gates: planned.gates,
        unsynced: planned.unsynced,
        plan_error,
    })
}

/// Start the campaign and run its first tick inline; a dry run only reports.
pub async fn launch(
    config: &Config,
    port: &impl CampaignPort,
    id: &str,
    dry_run: bool,
) -> Result<CampaignLaunchOutcome> {
    let campaign = get(config, id)?;
    let violations: Vec<String> = campaign_runner::preflight(config, port, &campaign)
        .iter()
        .map(ToString::to_string)
        .collect();
    let policy = policy(config, &campaign).unwrap_or_default();
    let waves = detail(config, id)?.waves;
    let (status, report) = if dry_run {
        (campaign.status, campaign_runner::assess(config, port, id)?)
    } else {
        campaign_runner::start_campaign(config, port, id)?;
        let report = campaign_runner::tick(config, port, id).await?;
        campaign_runner::wake().notify_one();
        (report.status, report)
    };
    Ok(CampaignLaunchOutcome {
        id: id.to_string(),
        dry_run,
        status,
        waves,
        policy,
        violations,
        report,
    })
}

pub fn status(config: &Config, port: &impl CampaignPort, id: &str) -> Result<CampaignStatusView> {
    let campaign = get(config, id)?;
    let all = all_tickets(config)?;
    let tickets = member_views(config, id, &all)?;
    let run = State::load(config)?
        .campaign_runs
        .remove(id)
        .unwrap_or_default();
    let policy = policy(config, &campaign).unwrap_or_default();
    let report = campaign_runner::assess(config, port, id)?;
    let waves = plan::plan(config, &members(id, &all), &all)
        .map(|p| p.waves)
        .unwrap_or_default();
    let counters = CampaignCounters {
        launched: run.launched(),
        max_launches: policy.max_launches,
        active: u32::try_from(tickets.iter().filter(|t| t.column == "in-progress").count())
            .unwrap_or(u32::MAX),
        max_parallel: policy.max_parallel,
        retries: tickets
            .iter()
            .map(|t| run.retries(&t.id))
            .max()
            .unwrap_or(0),
        max_retries: policy.max_retries,
    };
    Ok(CampaignStatusView {
        id: id.to_string(),
        status: campaign.status,
        tickets,
        waves,
        report,
        counters,
        policy,
        pause_reason: run.pause_reason,
    })
}
