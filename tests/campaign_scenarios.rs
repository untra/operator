//! Campaign runner scenarios, replayed in-process against simulated agents.
//!
//! Each fixture under `tests/fixtures/campaigns/` is one campaign. Add a
//! directory and a line to `SCENARIOS` to test a new one.

mod campaign_common;

const SCENARIOS: &[&str] = &[
    "diamond",
    "gated-chain",
    "governed-fan",
    "cycle",
    "blocked-unknown-dep",
    "launch-error",
    "orphan",
    "license-lapse",
    "client-wrapper",
    "completed-campaign",
];

macro_rules! scenario {
    ($test:ident, $name:literal) => {
        #[tokio::test]
        async fn $test() {
            campaign_common::run($name).await;
        }
    };
}

scenario!(diamond_pair_launches_together, "diamond");
scenario!(gated_chain_waits_for_operator, "gated-chain");
scenario!(governed_fan_throttles_and_pauses_on_retries, "governed-fan");
scenario!(cycle_is_refused_at_start, "cycle");
scenario!(
    unknown_dependency_blocks_only_its_ticket,
    "blocked-unknown-dep"
);
scenario!(launch_error_pauses_run, "launch-error");
scenario!(orphan_reported_stalled_not_relaunched, "orphan");
scenario!(license_lapse_stops_new_launches, "license-lapse");
scenario!(client_wrapper_refused_before_running, "client-wrapper");
scenario!(completed_campaign_cannot_restart, "completed-campaign");

use campaign_common::{Scenario, SimPort, CAMPAIGN};
use operator::campaigns::model::CampaignStatus;
use operator::services::campaign_runner;

#[tokio::test]
async fn assess_previews_first_tick_without_side_effects() {
    let s = Scenario::load("diamond");
    let port = SimPort::new(&s.config, &[]);
    campaign_runner::start_campaign(&s.config, &port, CAMPAIGN).unwrap();
    let preview = campaign_runner::assess(&s.config, &port, CAMPAIGN).unwrap();
    assert_eq!(preview.launched, vec!["FEAT-0001"]);
    assert_eq!(port.running(), 0);
    let state = operator::state::State::load(&s.config).unwrap();
    assert!(state.campaign_runs[CAMPAIGN].launches.is_empty());
    let report = campaign_runner::tick(&s.config, &port, CAMPAIGN)
        .await
        .unwrap();
    assert_eq!(report.launched, preview.launched);
}

#[tokio::test]
async fn pause_then_resume_clears_reason_and_running_cannot_restart() {
    let s = Scenario::load("diamond");
    let port = SimPort::new(&s.config, &[]);
    campaign_runner::start_campaign(&s.config, &port, CAMPAIGN).unwrap();
    let err = campaign_runner::start_campaign(&s.config, &port, CAMPAIGN).unwrap_err();
    assert!(err.to_string().contains("already running"), "{err}");

    campaign_runner::pause_campaign(&s.config, CAMPAIGN, "paused by operator").unwrap();
    assert_eq!(s.campaign().status, CampaignStatus::Paused);
    let state = operator::state::State::load(&s.config).unwrap();
    assert_eq!(
        state.campaign_runs[CAMPAIGN].pause_reason.as_deref(),
        Some("paused by operator")
    );
    let report = campaign_runner::tick(&s.config, &port, CAMPAIGN)
        .await
        .unwrap();
    assert!(report.launched.is_empty(), "a paused run launches nothing");

    campaign_runner::start_campaign(&s.config, &port, CAMPAIGN).unwrap();
    assert_eq!(s.campaign().status, CampaignStatus::Running);
    let state = operator::state::State::load(&s.config).unwrap();
    assert!(state.campaign_runs[CAMPAIGN].pause_reason.is_none());
}

#[test]
fn every_fixture_is_a_listed_scenario() {
    let mut found: Vec<String> = std::fs::read_dir(campaign_common::fixtures_root())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().join("scenario.toml").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();
    let mut listed: Vec<String> = SCENARIOS.iter().map(ToString::to_string).collect();
    listed.sort();
    assert_eq!(found, listed);
}

#[test]
fn every_scenario_parses() {
    for name in SCENARIOS {
        campaign_common::spec(name);
    }
}
