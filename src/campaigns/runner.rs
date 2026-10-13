//! Decide what a campaign launches next. Pure: the driver supplies every fact.

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::plan::{is_completed, ready, CampaignPlan};
use crate::config::GovernancePolicy;
use crate::queue::Ticket;
use crate::state::CampaignRun;

/// Free launch capacity outside the campaign's own policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Slots {
    /// Agents that may still start across all projects.
    pub global: usize,
    /// Agents allowed per project.
    pub per_repo_limit: usize,
    /// Running agents per project.
    pub per_repo_busy: BTreeMap<String, usize>,
}

/// Everything one tick decides from.
pub struct TickInput<'a> {
    pub plan: &'a CampaignPlan,
    pub members: &'a [&'a Ticket],
    pub all: &'a [Ticket],
    pub run: &'a CampaignRun,
    pub policy: &'a GovernancePolicy,
    pub slots: Slots,
    pub entitled: bool,
    /// Ticket ids with a live agent session.
    pub live: &'a BTreeSet<String>,
    pub is_paired: &'a dyn Fn(&Ticket) -> bool,
}

/// Why a pending member is not launching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "kind", content = "refs", rename_all = "snake_case")]
pub enum BlockReason {
    /// Waiting on members in earlier waves.
    Dependencies,
    /// Waiting on local tickets outside the campaign.
    Gate(Vec<String>),
    /// Waiting on upstream issues that are not synced.
    Unsynced(Vec<String>),
    /// Ready, but no Premium entitlement.
    License,
    /// Ready, but no free slot this tick.
    Capacity,
}

/// What the run does after this tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunTransition {
    Complete,
    Pause(String),
}

/// One tick's decision.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TickDecision {
    pub launch: Vec<String>,
    pub awaiting_operator: Vec<String>,
    /// In progress with no live agent; left to the relaunch flows.
    pub stalled: Vec<String>,
    pub blocked: BTreeMap<String, BlockReason>,
    pub transition: Option<RunTransition>,
}

fn in_progress(ticket: &Ticket) -> bool {
    !is_completed(ticket)
        && std::path::Path::new(&ticket.filepath)
            .parent()
            .and_then(|p| p.file_name())
            .is_some_and(|n| n == "in-progress")
}

fn below(cap: Option<u32>, used: u32) -> bool {
    cap.is_none_or(|c| used < c)
}

fn waiting_reason(plan: &CampaignPlan, all: &[Ticket], id: &str) -> BlockReason {
    if let Some(refs) = plan.unsynced.get(id) {
        return BlockReason::Unsynced(refs.clone());
    }
    let open_gates: Vec<String> = plan
        .gates
        .get(id)
        .into_iter()
        .flatten()
        .filter(|g| !all.iter().any(|t| &t.id == *g && is_completed(t)))
        .cloned()
        .collect();
    if open_gates.is_empty() {
        BlockReason::Dependencies
    } else {
        BlockReason::Gate(open_gates)
    }
}

/// Decide launches for one tick.
pub fn next_launches(input: &TickInput) -> TickDecision {
    let mut decision = TickDecision::default();
    if input.members.iter().all(|t| is_completed(t)) {
        decision.transition = Some(RunTransition::Complete);
        return decision;
    }

    let active: Vec<&&Ticket> = input.members.iter().filter(|t| in_progress(t)).collect();
    decision.stalled = active
        .iter()
        .filter(|t| !input.live.contains(&t.id))
        .map(|t| t.id.clone())
        .collect();

    let ready_ids: BTreeSet<String> = ready(input.plan, input.all).into_iter().collect();
    let mut candidates: Vec<&Ticket> = Vec::new();
    for id in input.plan.waves.iter().flatten() {
        let Some(ticket) = input.members.iter().find(|t| &t.id == id) else {
            continue;
        };
        if in_progress(ticket) {
            continue;
        }
        if !ready_ids.contains(id) {
            decision
                .blocked
                .insert(id.clone(), waiting_reason(input.plan, input.all, id));
        } else if (input.is_paired)(ticket) {
            decision.awaiting_operator.push(id.clone());
        } else {
            candidates.push(ticket);
        }
    }

    let policy = input.policy;
    let mut global = input.slots.global;
    let mut parallel = policy
        .max_parallel
        .map(|cap| (cap as usize).saturating_sub(active.len()));
    let mut busy = input.slots.per_repo_busy.clone();
    let mut launched = input.run.launched();
    for ticket in candidates {
        let id = &ticket.id;
        if !input.entitled {
            decision.blocked.insert(id.clone(), BlockReason::License);
            continue;
        }
        if decision.transition.is_some() {
            decision.blocked.insert(id.clone(), BlockReason::Capacity);
            continue;
        }
        if !below(
            policy.max_retries.map(|c| c + 1),
            input.run.launches.get(id).copied().unwrap_or(0),
        ) {
            decision.transition =
                Some(RunTransition::Pause(format!("max_retries reached by {id}")));
            decision.blocked.insert(id.clone(), BlockReason::Capacity);
            continue;
        }
        if !below(policy.max_launches, launched) {
            decision.transition = Some(RunTransition::Pause("max_launches reached".into()));
            decision.blocked.insert(id.clone(), BlockReason::Capacity);
            continue;
        }
        let repo_busy = busy.entry(ticket.project.clone()).or_default();
        if global == 0 || parallel == Some(0) || *repo_busy >= input.slots.per_repo_limit {
            decision.blocked.insert(id.clone(), BlockReason::Capacity);
            continue;
        }
        *repo_busy += 1;
        global -= 1;
        parallel = parallel.map(|p| p - 1);
        launched += 1;
        decision.launch.push(id.clone());
    }
    decision
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct Board {
        dir: tempfile::TempDir,
        n: u32,
    }

    impl Board {
        fn new() -> Self {
            Self {
                dir: tempfile::tempdir().unwrap(),
                n: 0,
            }
        }

        fn add(&mut self, folder: &str, id: &str, repo: &str, deps: &[&str]) {
            self.n += 1;
            let dir = self.dir.path().join(folder);
            fs::create_dir_all(&dir).unwrap();
            let kind = id.split('-').next().unwrap();
            let mut fm =
                format!("id: {id}\nstatus: queued\npriority: P2-medium\ncampaign: CAMP-0001");
            if !deps.is_empty() {
                fm.push_str("\ndepends_on:");
                for d in deps {
                    fm.push_str(&format!("\n  - \"{d}\""));
                }
            }
            let name = format!("20260101-12{:02}-{kind}-{repo}-t{}.md", self.n, self.n);
            fs::write(dir.join(name), format!("---\n{fm}\n---\n\n# T\n")).unwrap();
        }

        fn tickets(&self) -> Vec<Ticket> {
            ["queue", "in-progress", "completed"]
                .iter()
                .filter_map(|d| fs::read_dir(self.dir.path().join(d)).ok())
                .flatten()
                .map(|e| Ticket::from_file(&e.unwrap().path()).unwrap())
                .collect()
        }
    }

    fn spike_paired(t: &Ticket) -> bool {
        t.ticket_type == "SPIKE"
    }

    fn slots(global: usize, per_repo: usize) -> Slots {
        Slots {
            global,
            per_repo_limit: per_repo,
            per_repo_busy: BTreeMap::new(),
        }
    }

    struct Case {
        all: Vec<Ticket>,
        run: CampaignRun,
        policy: GovernancePolicy,
        slots: Slots,
        entitled: bool,
        live: BTreeSet<String>,
    }

    impl Case {
        fn new(board: &Board) -> Self {
            Self {
                all: board.tickets(),
                run: CampaignRun::default(),
                policy: GovernancePolicy::default(),
                slots: slots(8, 8),
                entitled: true,
                live: BTreeSet::new(),
            }
        }

        fn decide(&self) -> TickDecision {
            let members: Vec<&Ticket> = self.all.iter().collect();
            let plan =
                super::super::plan::plan(&crate::config::Config::default(), &members, &self.all)
                    .unwrap();
            next_launches(&TickInput {
                plan: &plan,
                members: &members,
                all: &self.all,
                run: &self.run,
                policy: &self.policy,
                slots: self.slots.clone(),
                entitled: self.entitled,
                live: &self.live,
                is_paired: &spike_paired,
            })
        }
    }

    fn diamond() -> Board {
        let mut b = Board::new();
        b.add("queue", "FEAT-0001", "api", &[]);
        b.add("queue", "FEAT-0002", "api", &["FEAT-0001"]);
        b.add("queue", "FEAT-0003", "web", &["FEAT-0001"]);
        b.add("queue", "FEAT-0004", "web", &["FEAT-0002", "FEAT-0003"]);
        b
    }

    #[test]
    fn test_first_wave_launches_and_rest_wait_on_dependencies() {
        let d = Case::new(&diamond()).decide();
        assert_eq!(d.launch, vec!["FEAT-0001"]);
        assert_eq!(d.blocked["FEAT-0004"], BlockReason::Dependencies);
        assert_eq!(d.transition, None);
    }

    #[test]
    fn test_pair_launches_together_once_root_completes() {
        let mut b = Board::new();
        b.add("completed", "FEAT-0001", "api", &[]);
        b.add("queue", "FEAT-0002", "api", &["FEAT-0001"]);
        b.add("queue", "FEAT-0003", "web", &["FEAT-0001"]);
        let d = Case::new(&b).decide();
        assert_eq!(d.launch, vec!["FEAT-0002", "FEAT-0003"]);
    }

    #[test]
    fn test_per_repo_limit_holds_back_same_repo() {
        let mut b = Board::new();
        b.add("queue", "FEAT-0001", "api", &[]);
        b.add("queue", "FEAT-0002", "api", &[]);
        b.add("queue", "FEAT-0003", "web", &[]);
        let mut c = Case::new(&b);
        c.slots = slots(8, 1);
        let d = c.decide();
        assert_eq!(d.launch, vec!["FEAT-0001", "FEAT-0003"]);
        assert_eq!(d.blocked["FEAT-0002"], BlockReason::Capacity);
    }

    #[test]
    fn test_busy_repo_counts_against_limit() {
        let mut b = Board::new();
        b.add("queue", "FEAT-0001", "api", &[]);
        let mut c = Case::new(&b);
        c.slots = slots(8, 1);
        c.slots.per_repo_busy.insert("api".into(), 1);
        assert!(c.decide().launch.is_empty());
    }

    #[test]
    fn test_global_and_max_parallel_cap_launches() {
        let mut b = Board::new();
        for i in 1..=4 {
            b.add("queue", &format!("FEAT-000{i}"), &format!("r{i}"), &[]);
        }
        let mut c = Case::new(&b);
        c.slots = slots(3, 8);
        assert_eq!(c.decide().launch.len(), 3);
        c.policy.max_parallel = Some(2);
        assert_eq!(c.decide().launch.len(), 2);
    }

    #[test]
    fn test_max_parallel_counts_in_progress_members() {
        let mut b = Board::new();
        b.add("in-progress", "FEAT-0001", "a", &[]);
        b.add("queue", "FEAT-0002", "b", &[]);
        b.add("queue", "FEAT-0003", "c", &[]);
        let mut c = Case::new(&b);
        c.live.insert("FEAT-0001".into());
        c.policy.max_parallel = Some(2);
        assert_eq!(c.decide().launch, vec!["FEAT-0002"]);
    }

    #[test]
    fn test_paired_ticket_awaits_operator_and_never_launches() {
        let mut b = Board::new();
        b.add("queue", "SPIKE-0001", "api", &[]);
        b.add("queue", "FIX-0002", "api", &["SPIKE-0001"]);
        let d = Case::new(&b).decide();
        assert!(d.launch.is_empty());
        assert_eq!(d.awaiting_operator, vec!["SPIKE-0001"]);
    }

    #[test]
    fn test_unentitled_launches_nothing() {
        let mut c = Case::new(&diamond());
        c.entitled = false;
        let d = c.decide();
        assert!(d.launch.is_empty());
        assert_eq!(d.blocked["FEAT-0001"], BlockReason::License);
        assert_eq!(d.transition, None);
    }

    #[test]
    fn test_all_completed_completes_run() {
        let mut b = Board::new();
        b.add("completed", "FEAT-0001", "api", &[]);
        assert_eq!(
            Case::new(&b).decide().transition,
            Some(RunTransition::Complete)
        );
    }

    #[test]
    fn test_max_retries_pauses_when_ticket_needs_another_launch() {
        let mut b = Board::new();
        b.add("queue", "FEAT-0001", "api", &[]);
        let mut c = Case::new(&b);
        c.policy.max_retries = Some(1);
        c.run.record_launch("FEAT-0001");
        assert_eq!(c.decide().launch, vec!["FEAT-0001"]);
        c.run.record_launch("FEAT-0001");
        let d = c.decide();
        assert!(d.launch.is_empty());
        assert_eq!(
            d.transition,
            Some(RunTransition::Pause(
                "max_retries reached by FEAT-0001".into()
            ))
        );
    }

    #[test]
    fn test_max_launches_caps_the_run() {
        let mut b = Board::new();
        b.add("completed", "FEAT-0001", "a", &[]);
        b.add("queue", "FEAT-0002", "b", &[]);
        b.add("queue", "FEAT-0003", "c", &[]);
        let mut c = Case::new(&b);
        c.run.record_launch("FEAT-0001");
        c.policy.max_launches = Some(2);
        let d = c.decide();
        assert_eq!(d.launch, vec!["FEAT-0002"]);
        assert_eq!(
            d.transition,
            Some(RunTransition::Pause("max_launches reached".into()))
        );
    }

    #[test]
    fn test_in_progress_without_live_agent_is_stalled() {
        let mut b = Board::new();
        b.add("in-progress", "FEAT-0001", "a", &[]);
        b.add("in-progress", "FEAT-0002", "b", &[]);
        let mut c = Case::new(&b);
        c.live.insert("FEAT-0002".into());
        let d = c.decide();
        assert_eq!(d.stalled, vec!["FEAT-0001"]);
        assert!(d.launch.is_empty());
    }

    #[test]
    fn test_unsynced_and_gate_reasons() {
        let mut b = Board::new();
        b.add("queue", "FEAT-0001", "a", &["jira:PROJ-9"]);
        let mut c = Case::new(&b);
        let d = c.decide();
        assert_eq!(
            d.blocked["FEAT-0001"],
            BlockReason::Unsynced(vec!["jira:PROJ-9".into()])
        );

        let mut b = Board::new();
        b.add("queue", "FEAT-0001", "a", &["TASK-0009"]);
        let dir = b.dir.path().join("queue");
        fs::write(
            dir.join("20260101-1100-TASK-a-gate.md"),
            "---\nid: TASK-0009\nstatus: queued\n---\n\n# G\n",
        )
        .unwrap();
        c = Case::new(&b);
        c.all
            .retain(|t| t.campaign.is_some() || t.id == "TASK-0009");
        let members: Vec<&Ticket> = c.all.iter().filter(|t| t.campaign.is_some()).collect();
        let plan =
            super::super::plan::plan(&crate::config::Config::default(), &members, &c.all).unwrap();
        let d = next_launches(&TickInput {
            plan: &plan,
            members: &members,
            all: &c.all,
            run: &c.run,
            policy: &c.policy,
            slots: c.slots.clone(),
            entitled: true,
            live: &c.live,
            is_paired: &spike_paired,
        });
        assert_eq!(
            d.blocked["FEAT-0001"],
            BlockReason::Gate(vec!["TASK-0009".into()])
        );
    }
}
