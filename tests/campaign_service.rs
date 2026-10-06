//! The campaign service behind REST and MCP, driven licensed through simulated agents.

mod campaign_common;

use std::fs;

use campaign_common::{Scenario, SimPort, CAMPAIGN};
use operator::campaigns::model::CampaignStatus;
use operator::campaigns::service::{self, NewCampaignInput};
use operator::config::{GovernancePolicy, SessionWrapperType};

fn stray(s: &Scenario, id: &str) {
    let path = std::path::Path::new(&s.config.paths.tickets)
        .join("queue")
        .join(format!("20260101-1300-FEAT-api-{}.md", id.to_lowercase()));
    fs::write(
        path,
        format!("---\nid: {id}\nstatus: queued\n---\n\n# {id}\n"),
    )
    .unwrap();
}

#[test]
fn create_campaign_validates_named_policy_and_keeps_overrides() {
    let mut s = Scenario::load("diamond");
    let err = service::create_campaign(
        &s.config,
        NewCampaignInput {
            title: "Bad".into(),
            governance: Some("missing".into()),
            ..NewCampaignInput::default()
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("missing"), "{err}");

    s.config.governance.push(GovernancePolicy {
        name: "team".into(),
        ..GovernancePolicy::default()
    });
    let created = service::create_campaign(
        &s.config,
        NewCampaignInput {
            title: "Epic".into(),
            goal: Some("Ship it.".into()),
            governance: Some("team".into()),
            governance_overrides: Some(GovernancePolicy {
                name: "epic".into(),
                max_parallel: Some(2),
                ..GovernancePolicy::default()
            }),
            ..NewCampaignInput::default()
        },
    )
    .unwrap();
    assert_eq!(created.id, "CAMP-0002");
    let detail = service::detail(&s.config, &created.id).unwrap();
    assert_eq!(
        detail.campaign.governance_overrides.unwrap().max_parallel,
        Some(2)
    );
    assert!(detail.members.is_empty());
}

#[test]
fn assign_ticket_joins_campaign_and_rejects_cycles() {
    let s = Scenario::load("diamond");
    stray(&s, "FEAT-0010");
    let ticket = service::assign_ticket(
        &s.config,
        "FEAT-0010",
        Some(CAMPAIGN),
        Some(vec!["TASK-0004".into()]),
    )
    .unwrap();
    assert_eq!(ticket.campaign.as_deref(), Some(CAMPAIGN));
    let detail = service::detail(&s.config, CAMPAIGN).unwrap();
    assert_eq!(detail.waves.last().unwrap(), &vec!["FEAT-0010".to_string()]);

    let err = service::assign_ticket(
        &s.config,
        "FEAT-0001",
        Some(CAMPAIGN),
        Some(vec!["FEAT-0010".into()]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("cycle"), "{err}");
    let root = s
        .tickets()
        .into_iter()
        .find(|t| t.id == "FEAT-0001")
        .unwrap();
    assert!(root.depends_on.is_empty(), "a rejected edit is reverted");

    let err = service::assign_ticket(&s.config, "FEAT-0010", Some("CAMP-0404"), None).unwrap_err();
    assert!(err.to_string().contains("CAMP-0404"), "{err}");

    let ticket = service::assign_ticket(&s.config, "FEAT-0010", None, None).unwrap();
    assert!(ticket.campaign.is_none());
}

#[test]
fn detail_lists_members_with_columns_and_waves() {
    let s = Scenario::load("diamond");
    let detail = service::detail(&s.config, CAMPAIGN).unwrap();
    assert_eq!(detail.members.len(), 4);
    assert!(detail.members.iter().all(|m| m.column == "queue"));
    assert_eq!(detail.waves.len(), 3);
    assert!(detail.plan_error.is_none());
}

#[tokio::test]
async fn dry_run_reports_violations_and_changes_nothing() {
    let mut s = Scenario::load("diamond");
    let port = SimPort::new(&s.config, &[]);
    let outcome = service::launch(&s.config, &port, CAMPAIGN, true)
        .await
        .unwrap();
    assert!(outcome.violations.is_empty(), "{:?}", outcome.violations);
    assert_eq!(outcome.report.launched, vec!["FEAT-0001"]);
    assert_eq!(port.running(), 0);
    assert_eq!(s.campaign().status, CampaignStatus::Ready);

    port.set_entitled(false);
    s.config.sessions.wrapper = SessionWrapperType::Vscode;
    let outcome = service::launch(&s.config, &port, CAMPAIGN, true)
        .await
        .unwrap();
    let text = outcome.violations.join(" | ");
    assert!(text.contains("Premium"), "{text}");
    assert!(text.contains("client"), "{text}");
}

#[tokio::test]
async fn launch_starts_and_runs_the_first_wave_then_status_counts_it() {
    let s = Scenario::load("governed-fan");
    let port = SimPort::new(&s.config, &[]);
    let outcome = service::launch(&s.config, &port, CAMPAIGN, false)
        .await
        .unwrap();
    assert_eq!(outcome.status, CampaignStatus::Running);
    assert_eq!(outcome.report.launched, vec!["FEAT-0001"]);
    assert_eq!(port.running(), 1);

    let view = service::status(&s.config, &port, CAMPAIGN).unwrap();
    assert_eq!(view.status, CampaignStatus::Running);
    assert_eq!(view.counters.launched, 1);
    assert_eq!(view.counters.active, 1);
    assert_eq!(view.counters.max_parallel, Some(2));
    assert_eq!(view.counters.max_retries, Some(1));
    let root = view.tickets.iter().find(|t| t.id == "FEAT-0001").unwrap();
    assert_eq!((root.column.as_str(), root.launches), ("in-progress", 1));
    assert!(view.report.launched.is_empty(), "root still running");

    let err = service::launch(&s.config, &port, CAMPAIGN, false)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("already running"), "{err}");
}
