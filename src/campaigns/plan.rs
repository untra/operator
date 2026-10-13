//! Plan a campaign's tickets into dependency waves.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::config::Config;
use crate::queue::deps::{resolve, Resolved};
use crate::queue::{queue_order_key, Ticket};

/// Tickets grouped into waves; every ticket in a wave can run in parallel.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CampaignPlan {
    /// Pending members in launch order; wave N depends only on earlier waves.
    pub waves: Vec<Vec<String>>,
    /// Member id → local tickets outside the campaign it waits on.
    pub gates: BTreeMap<String, Vec<String>>,
    /// Member id → references to issues that are not synced; these block.
    pub unsynced: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlanError {
    #[error("dependency cycle among {}", .0.join(", "))]
    Cycle(Vec<String>),
}

fn folder(ticket: &Ticket) -> Option<&str> {
    Path::new(&ticket.filepath).parent()?.file_name()?.to_str()
}

/// Done: in `completed/`, or marked completed.
pub fn is_completed(ticket: &Ticket) -> bool {
    folder(ticket) == Some("completed") || ticket.status == "completed"
}

fn is_queued(ticket: &Ticket) -> bool {
    folder(ticket) == Some("queue") && !is_completed(ticket)
}

/// Build waves over the pending members; completed members count as satisfied.
pub fn plan(
    config: &Config,
    members: &[&Ticket],
    all: &[Ticket],
) -> Result<CampaignPlan, PlanError> {
    let member_ids: BTreeSet<&str> = members.iter().map(|t| t.id.as_str()).collect();
    let pending: BTreeMap<&str, &Ticket> = members
        .iter()
        .filter(|t| !is_completed(t))
        .map(|t| (t.id.as_str(), *t))
        .collect();

    let mut result = CampaignPlan::default();
    let mut blockers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (id, ticket) in &pending {
        let entry = blockers.entry((*id).to_string()).or_default();
        for dep in &ticket.depends_on {
            match resolve(dep, all) {
                Resolved::Local(dep_id) if pending.contains_key(dep_id.as_str()) => {
                    entry.insert(dep_id);
                }
                Resolved::Local(dep_id) if member_ids.contains(dep_id.as_str()) => {}
                Resolved::Local(dep_id) => {
                    result
                        .gates
                        .entry((*id).to_string())
                        .or_default()
                        .push(dep_id);
                }
                Resolved::Unsynced(r) => {
                    result
                        .unsynced
                        .entry((*id).to_string())
                        .or_default()
                        .push(r);
                }
            }
        }
    }
    result.waves = waves(blockers, |id| queue_order_key(config, pending[id]))?;
    Ok(result)
}

/// Kahn layering: each wave holds nodes whose blockers are all in earlier waves.
pub fn waves<K: Ord>(
    mut blockers: BTreeMap<String, BTreeSet<String>>,
    order: impl Fn(&str) -> K,
) -> Result<Vec<Vec<String>>, PlanError> {
    let mut result = Vec::new();
    while !blockers.is_empty() {
        let mut wave: Vec<String> = blockers
            .iter()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(id, _)| id.clone())
            .collect();
        if wave.is_empty() {
            return Err(PlanError::Cycle(blockers.into_keys().collect()));
        }
        wave.sort_by_cached_key(|id| order(id));
        for id in &wave {
            blockers.remove(id);
        }
        for deps in blockers.values_mut() {
            for id in &wave {
                deps.remove(id);
            }
        }
        result.push(wave);
    }
    Ok(result)
}

/// Queued members whose dependencies are all completed, in plan order.
pub fn ready(plan: &CampaignPlan, all: &[Ticket]) -> Vec<String> {
    let done = |id: &str| all.iter().any(|t| t.id == id && is_completed(t));
    plan.waves
        .iter()
        .flatten()
        .filter(|id| !plan.unsynced.contains_key(*id))
        .filter_map(|id| all.iter().find(|t| &t.id == id))
        .filter(|t| is_queued(t))
        .filter(|t| {
            t.depends_on.iter().all(|dep| match resolve(dep, all) {
                Resolved::Local(dep_id) => done(&dep_id),
                Resolved::Unsynced(_) => false,
            })
        })
        .map(|t| t.id.clone())
        .collect()
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

        fn add(&mut self, folder: &str, id: &str, priority: &str, extra: &str) {
            self.n += 1;
            let dir = self.dir.path().join(folder);
            fs::create_dir_all(&dir).unwrap();
            let ticket_type = id.split('-').next().unwrap();
            let name = format!("20260101-12{:02}-{ticket_type}-demo-t{}.md", self.n, self.n);
            fs::write(
                dir.join(name),
                format!(
                    "---\nid: {id}\nstatus: queued\npriority: {priority}\n{extra}\n---\n\n# T\n"
                ),
            )
            .unwrap();
        }

        fn member(&mut self, id: &str, deps: &[&str]) {
            self.add("queue", id, "P2-medium", &member_extra(deps));
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

    fn member_extra(deps: &[&str]) -> String {
        let mut extra = "campaign: CAMP-0001".to_string();
        if !deps.is_empty() {
            extra.push_str("\ndepends_on:");
            for d in deps {
                extra.push_str(&format!("\n  - \"{d}\""));
            }
        }
        extra
    }

    fn plan_board(board: &Board) -> (Result<CampaignPlan, PlanError>, Vec<Ticket>) {
        let all = board.tickets();
        let members: Vec<&Ticket> = all
            .iter()
            .filter(|t| t.campaign.as_deref() == Some("CAMP-0001"))
            .collect();
        (plan(&Config::default(), &members, &all), all)
    }

    #[test]
    fn test_diamond_plans_three_waves() {
        let mut b = Board::new();
        b.member("FEAT-0001", &[]);
        b.member("FEAT-0002", &["FEAT-0001"]);
        b.member("FEAT-0003", &["FEAT-0001"]);
        b.member("FEAT-0004", &["FEAT-0002", "FEAT-0003"]);
        let (plan, all) = plan_board(&b);
        let plan = plan.unwrap();
        assert_eq!(
            plan.waves,
            vec![
                vec!["FEAT-0001"],
                vec!["FEAT-0002", "FEAT-0003"],
                vec!["FEAT-0004"]
            ]
        );
        assert_eq!(ready(&plan, &all), vec!["FEAT-0001"]);
    }

    #[test]
    fn test_cycle_is_reported() {
        let mut b = Board::new();
        b.member("FEAT-0001", &["FEAT-0002"]);
        b.member("FEAT-0002", &["FEAT-0001"]);
        b.member("FEAT-0003", &[]);
        let (plan, _) = plan_board(&b);
        assert_eq!(
            plan.unwrap_err(),
            PlanError::Cycle(vec!["FEAT-0001".into(), "FEAT-0002".into()])
        );
    }

    #[test]
    fn test_wave_order_follows_queue_priority() {
        let mut b = Board::new();
        b.add("queue", "FEAT-0001", "P3-low", &member_extra(&[]));
        b.add("queue", "FEAT-0002", "P0-critical", &member_extra(&[]));
        b.add("queue", "FIX-0001", "P3-low", &member_extra(&[]));
        let (plan, _) = plan_board(&b);
        assert_eq!(
            plan.unwrap().waves,
            vec![vec!["FIX-0001", "FEAT-0002", "FEAT-0001"]]
        );
    }

    #[test]
    fn test_completed_members_drop_out_and_unblock() {
        let mut b = Board::new();
        b.add("completed", "FEAT-0001", "P2-medium", &member_extra(&[]));
        b.member("FEAT-0002", &["FEAT-0001"]);
        let (plan, all) = plan_board(&b);
        let plan = plan.unwrap();
        assert_eq!(plan.waves, vec![vec!["FEAT-0002"]]);
        assert_eq!(ready(&plan, &all), vec!["FEAT-0002"]);
    }

    #[test]
    fn test_external_gate_blocks_until_completed() {
        let mut b = Board::new();
        b.add("in-progress", "TASK-0009", "P2-medium", "");
        b.member("FEAT-0001", &["TASK-0009"]);
        let (plan, all) = plan_board(&b);
        let plan = plan.unwrap();
        assert_eq!(plan.gates["FEAT-0001"], vec!["TASK-0009"]);
        assert!(ready(&plan, &all).is_empty());

        let mut b = Board::new();
        b.add("completed", "TASK-0009", "P2-medium", "");
        b.member("FEAT-0001", &["TASK-0009"]);
        let (plan, all) = plan_board(&b);
        assert_eq!(ready(&plan.unwrap(), &all), vec!["FEAT-0001"]);
    }

    #[test]
    fn test_unsynced_ref_blocks() {
        let mut b = Board::new();
        b.member("FEAT-0001", &["jira:PROJ-99"]);
        let (plan, all) = plan_board(&b);
        let plan = plan.unwrap();
        assert_eq!(plan.unsynced["FEAT-0001"], vec!["jira:PROJ-99"]);
        assert_eq!(plan.waves, vec![vec!["FEAT-0001"]]);
        assert!(ready(&plan, &all).is_empty());
    }

    #[test]
    fn test_provider_refs_resolve_to_members_despite_type_mapping() {
        let mut b = Board::new();
        b.add(
            "queue",
            "FIX-PROJ12",
            "P2-medium",
            &format!(
                "{}\nexternal_id: PROJ-12\nexternal_provider: jira",
                member_extra(&[])
            ),
        );
        b.member("TASK-PROJ42", &["jira:PROJ-12"]);
        let (plan, _) = plan_board(&b);
        assert_eq!(
            plan.unwrap().waves,
            vec![vec!["FIX-PROJ12"], vec!["TASK-PROJ42"]]
        );
    }
}
