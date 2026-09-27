//! LLM judge phase for multi-agent steps.
//!
//! The sync loop is synchronous and holds a `State` snapshot it saves whole,
//! so the judge runs as a detached task that reports through a side file keyed
//! by attempt id (see `StepManager::write_judge_outcome`). The loop polls that
//! file each tick and applies the deterministic rule on failure or deadline.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::config::Config;
use crate::llm::native::{judge_checked, JudgeOutcome, JudgeVerdict, NativeLlm, NativeLlmError};
use crate::state::JudgeAttempt;
use crate::templates::step_type::JudgePlan;

pub struct ConfiguredJudge {
    pub llm: Arc<dyn NativeLlm>,
    pub timeout_secs: u64,
}

/// Builds the judge from config. `Ok(None)` = no judge configured (silent
/// deterministic path); `Err` = configured but unusable (noted in history).
pub type JudgeFactory =
    Arc<dyn Fn(&Config) -> Result<Option<ConfiguredJudge>, NativeLlmError> + Send + Sync>;

pub fn default_judge_factory() -> JudgeFactory {
    Arc::new(judge_from_config)
}

fn judge_from_config(config: &Config) -> Result<Option<ConfiguredJudge>, NativeLlmError> {
    let Some(judge) = config.native_llm.judge.as_ref() else {
        return Ok(None);
    };
    build_judge(config, judge).map(|llm| {
        Some(ConfiguredJudge {
            llm,
            timeout_secs: judge.timeout_secs,
        })
    })
}

#[cfg(feature = "native-llm")]
fn build_judge(
    config: &Config,
    judge: &crate::config::JudgeConfig,
) -> Result<Arc<dyn NativeLlm>, NativeLlmError> {
    let server = crate::agents::delegator_resolution::resolve_model_server_by_name(
        config,
        &judge.model_server,
    )
    .map_err(|e| NativeLlmError::Config(e.to_string()))?;
    let policy = crate::auth::egress::EgressPolicy::from_config(config);
    let llm = crate::llm::native::rig::RigJudge::new(
        &server,
        &judge.model,
        &policy,
        std::time::Duration::from_secs(judge.timeout_secs),
    )?;
    Ok(Arc::new(llm))
}

#[cfg(not(feature = "native-llm"))]
fn build_judge(
    _config: &Config,
    _judge: &crate::config::JudgeConfig,
) -> Result<Arc<dyn NativeLlm>, NativeLlmError> {
    Err(NativeLlmError::Config(
        "operator was built without the native-llm feature".to_string(),
    ))
}

/// Task body: call the judge under a timeout and translate its pick back to
/// the aggregator's index. Never fails; failures become `JudgeOutcome::Failed`.
pub async fn run_judge(
    llm: Arc<dyn NativeLlm>,
    plan: JudgePlan,
    timeout_secs: u64,
) -> JudgeOutcome {
    let call = judge_checked(llm.as_ref(), &plan.request);
    let result =
        match tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), call).await {
            Ok(r) => r,
            Err(_) => Err(NativeLlmError::Timeout(timeout_secs)),
        };
    match result {
        Ok(verdict) => match plan.aggregate_index(verdict.winner_index) {
            Some(winner_index) => JudgeOutcome::Verdict(JudgeVerdict {
                winner_index,
                rationale: verdict.rationale,
            }),
            None => JudgeOutcome::Failed {
                reason: NativeLlmError::OutOfRange {
                    index: verdict.winner_index,
                    len: plan.aggregate_indices.len(),
                }
                .to_string(),
            },
        },
        Err(e) => JudgeOutcome::Failed {
            reason: e.to_string(),
        },
    }
}

/// What the sync loop does with a group in the judging phase this tick.
#[derive(Debug, Clone, PartialEq)]
pub enum JudgingStep {
    Wait,
    /// Finalize with the judge's pick (an aggregator index).
    Apply(JudgeVerdict),
    /// Finalize with the deterministic rule; the reason goes to ticket history.
    Fallback(String),
}

pub fn judging_step(
    attempt: &JudgeAttempt,
    outcome: Option<JudgeOutcome>,
    now: DateTime<Utc>,
) -> JudgingStep {
    match outcome {
        Some(JudgeOutcome::Verdict(v)) => JudgingStep::Apply(v),
        Some(JudgeOutcome::Failed { reason }) => JudgingStep::Fallback(reason),
        None if now >= attempt.deadline() => JudgingStep::Fallback(format!(
            "no verdict by deadline (attempt {})",
            attempt.attempt_id
        )),
        None => JudgingStep::Wait,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::native::fake::FakeNativeLlm;
    use crate::llm::native::{JudgeCandidate, JudgeRequest, JudgeVerdict};
    use async_trait::async_trait;

    fn plan(aggregate_indices: Vec<usize>) -> JudgePlan {
        JudgePlan {
            request: JudgeRequest {
                instruction: "pick".to_string(),
                candidates: aggregate_indices
                    .iter()
                    .map(|i| JudgeCandidate {
                        label: i.to_string(),
                        text: format!("answer {i}"),
                    })
                    .collect(),
            },
            aggregate_indices,
        }
    }

    fn verdict(i: usize) -> JudgeVerdict {
        JudgeVerdict {
            winner_index: i,
            rationale: "r".to_string(),
        }
    }

    fn attempt(started_at: DateTime<Utc>) -> JudgeAttempt {
        JudgeAttempt {
            attempt_id: "att".to_string(),
            started_at,
            timeout_secs: 10,
        }
    }

    #[tokio::test]
    async fn run_judge_maps_candidate_index_to_aggregate_index() {
        let llm = Arc::new(FakeNativeLlm(Ok(verdict(1))));
        let outcome = run_judge(llm, plan(vec![0, 2]), 5).await;
        assert_eq!(outcome, JudgeOutcome::Verdict(verdict(2)));
    }

    #[tokio::test]
    async fn run_judge_turns_errors_into_failed_outcome() {
        let llm = Arc::new(FakeNativeLlm(Err(NativeLlmError::NoToolCall)));
        let outcome = run_judge(llm, plan(vec![0, 1]), 5).await;
        assert!(matches!(outcome, JudgeOutcome::Failed { reason } if reason.contains("submit")));
    }

    #[tokio::test]
    async fn run_judge_rejects_out_of_range_pick() {
        let llm = Arc::new(FakeNativeLlm(Ok(verdict(5))));
        let outcome = run_judge(llm, plan(vec![0, 1]), 5).await;
        assert!(matches!(outcome, JudgeOutcome::Failed { .. }));
    }

    struct NeverAnswers;

    #[async_trait]
    impl NativeLlm for NeverAnswers {
        async fn judge(&self, _: &JudgeRequest) -> Result<JudgeVerdict, NativeLlmError> {
            std::future::pending().await
        }
    }

    #[tokio::test(start_paused = true)]
    async fn run_judge_times_out() {
        let outcome = run_judge(Arc::new(NeverAnswers), plan(vec![0, 1]), 3).await;
        assert_eq!(
            outcome,
            JudgeOutcome::Failed {
                reason: NativeLlmError::Timeout(3).to_string()
            }
        );
    }

    #[test]
    fn judging_step_applies_verdict() {
        let a = attempt(Utc::now());
        let step = judging_step(&a, Some(JudgeOutcome::Verdict(verdict(1))), Utc::now());
        assert_eq!(step, JudgingStep::Apply(verdict(1)));
    }

    #[test]
    fn judging_step_falls_back_on_failure() {
        let a = attempt(Utc::now());
        let failed = JudgeOutcome::Failed {
            reason: "boom".to_string(),
        };
        assert_eq!(
            judging_step(&a, Some(failed), Utc::now()),
            JudgingStep::Fallback("boom".to_string())
        );
    }

    #[test]
    fn judging_step_waits_before_deadline() {
        let a = attempt(Utc::now());
        assert_eq!(judging_step(&a, None, Utc::now()), JudgingStep::Wait);
    }

    #[test]
    fn judging_step_falls_back_after_deadline_when_task_is_gone() {
        // e.g. the daemon restarted mid-judge: no task will ever write the file
        let a = attempt(Utc::now() - chrono::Duration::hours(1));
        assert!(matches!(
            judging_step(&a, None, Utc::now()),
            JudgingStep::Fallback(_)
        ));
    }

    #[test]
    fn factory_returns_none_without_judge_config() {
        assert!(judge_from_config(&Config::default()).unwrap().is_none());
    }

    #[cfg(not(feature = "native-llm"))]
    #[test]
    fn factory_errors_when_configured_but_feature_off() {
        let mut config = Config::default();
        config.native_llm.judge = Some(crate::config::JudgeConfig {
            model_server: "anthropic-api".to_string(),
            model: "claude-sonnet-5".to_string(),
            timeout_secs: 10,
        });
        assert!(matches!(
            judge_from_config(&config),
            Err(NativeLlmError::Config(_))
        ));
    }
}
