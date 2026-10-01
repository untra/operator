//! Native (in-daemon) LLM calls.
//!
//! Everything here is Operator-owned and compiles without the `native-llm` feature,
//! callers and tests never see a provider SDK type. The only SDK binding lives in [`rig`],
//! behind the feature; swapping or dropping the SDK touches that submodule alone.

#[cfg(feature = "native-llm")]
pub mod rig;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Default judge instruction when a step declares no `voting_prompt` /
/// `selection_prompt`. Selection-only: the judge returns an index, it never synthesizes a new answer.
pub const DEFAULT_SELECTION_INSTRUCTION: &str =
    "Review the candidate outputs and select the single best one.";

pub const DEFAULT_JUDGE_TIMEOUT_SECS: u64 = 120;

#[derive(Debug, Clone, PartialEq)]
pub struct JudgeCandidate {
    pub label: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JudgeRequest {
    pub instruction: String,
    pub candidates: Vec<JudgeCandidate>,
}

/// The judge's structured answer. `winner_index` is into the request's `candidates`, zero-based.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct JudgeVerdict {
    /// Zero-based index of the best candidate
    pub winner_index: usize,
    /// One or two sentences explaining the choice
    pub rationale: String,
}

// Most variants are only produced by the feature-gated SDK binding.
#[cfg_attr(not(feature = "native-llm"), allow(dead_code))]
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum NativeLlmError {
    #[error("model server rejected by egress policy: {0}")]
    Egress(String),
    #[error("model server misconfigured: {0}")]
    Config(String),
    #[error("request failed: {0}")]
    Request(String),
    #[error("model did not call the submit tool")]
    NoToolCall,
    #[error("model returned invalid structured output: {0}")]
    InvalidOutput(String),
    #[error("judge picked candidate {index} but only {len} exist")]
    OutOfRange { index: usize, len: usize },
    #[error("judge timed out after {0}s")]
    Timeout(u64),
}

#[async_trait]
pub trait NativeLlm: Send + Sync {
    async fn judge(&self, request: &JudgeRequest) -> Result<JudgeVerdict, NativeLlmError>;
}

/// Run the judge and reject an out-of-range pick, so every caller gets a
/// verdict it can index with, or an error it falls back on.
pub async fn judge_checked(
    llm: &dyn NativeLlm,
    request: &JudgeRequest,
) -> Result<JudgeVerdict, NativeLlmError> {
    let verdict = llm.judge(request).await?;
    let len = request.candidates.len();
    if verdict.winner_index >= len {
        return Err(NativeLlmError::OutOfRange {
            index: verdict.winner_index,
            len,
        });
    }
    Ok(verdict)
}

/// Terminal result of one judge attempt, persisted as a side file so the sync
/// loop can pick it up without the judge task touching `State`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum JudgeOutcome {
    Verdict(JudgeVerdict),
    Failed { reason: String },
}

#[cfg(test)]
pub mod fake {
    use super::*;

    /// Scripted `NativeLlm` for tests: returns the same result for every call.
    pub struct FakeNativeLlm(pub Result<JudgeVerdict, NativeLlmError>);

    #[async_trait]
    impl NativeLlm for FakeNativeLlm {
        async fn judge(&self, _request: &JudgeRequest) -> Result<JudgeVerdict, NativeLlmError> {
            self.0.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::FakeNativeLlm;
    use super::*;

    fn request(n: usize) -> JudgeRequest {
        JudgeRequest {
            instruction: DEFAULT_SELECTION_INSTRUCTION.to_string(),
            candidates: (0..n)
                .map(|i| JudgeCandidate {
                    label: format!("c{i}"),
                    text: format!("answer {i}"),
                })
                .collect(),
        }
    }

    fn verdict(i: usize) -> JudgeVerdict {
        JudgeVerdict {
            winner_index: i,
            rationale: "best".to_string(),
        }
    }

    #[tokio::test]
    async fn judge_checked_passes_in_range_verdict() {
        let llm = FakeNativeLlm(Ok(verdict(1)));
        assert_eq!(judge_checked(&llm, &request(2)).await, Ok(verdict(1)));
    }

    #[tokio::test]
    async fn judge_checked_rejects_out_of_range_index() {
        let llm = FakeNativeLlm(Ok(verdict(2)));
        assert_eq!(
            judge_checked(&llm, &request(2)).await,
            Err(NativeLlmError::OutOfRange { index: 2, len: 2 })
        );
    }

    #[tokio::test]
    async fn judge_checked_propagates_errors() {
        let llm = FakeNativeLlm(Err(NativeLlmError::NoToolCall));
        assert_eq!(
            judge_checked(&llm, &request(2)).await,
            Err(NativeLlmError::NoToolCall)
        );
    }

    #[test]
    fn outcome_roundtrips_as_tagged_json() {
        let v = JudgeOutcome::Verdict(verdict(0));
        let json = serde_json::to_value(&v).unwrap();
        assert_eq!(json["outcome"], "verdict");
        assert_eq!(json["winner_index"], 0);
        assert_eq!(serde_json::from_value::<JudgeOutcome>(json).unwrap(), v);

        let f = JudgeOutcome::Failed {
            reason: "x".to_string(),
        };
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["outcome"], "failed");
        assert_eq!(serde_json::from_value::<JudgeOutcome>(json).unwrap(), f);
    }
}
