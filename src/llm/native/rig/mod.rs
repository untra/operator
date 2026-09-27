//! The only module that imports `rig_core`. Nothing here is re-exported with a Rig type in its signature.

mod client;

use std::fmt::Write as _;
use std::time::Duration;

use async_trait::async_trait;
use rig_core::completion::{CompletionResponse, ToolDefinition};
use rig_core::message::AssistantContent;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use super::{JudgeRequest, JudgeVerdict, NativeLlm, NativeLlmError};
use crate::auth::egress::EgressPolicy;
use crate::config::ModelServer;

const SUBMIT_TOOL: &str = "submit";
const SUBMIT_DESCRIPTION: &str = "Submit your answer as structured data.";
const JUDGE_MAX_TOKENS: u64 = 1024;
const JUDGE_PREAMBLE: &str = "You are a careful reviewer judging candidate outputs \
    from software agents. Answer only by calling the submit tool.";

pub struct RigJudge {
    model: client::JudgeModel,
}

impl RigJudge {
    pub fn new(
        server: &ModelServer,
        model: &str,
        policy: &EgressPolicy,
        timeout: Duration,
    ) -> Result<Self, NativeLlmError> {
        Ok(Self {
            model: client::build(server, model, policy, timeout)?,
        })
    }
}

#[async_trait]
impl NativeLlm for RigJudge {
    async fn judge(&self, request: &JudgeRequest) -> Result<JudgeVerdict, NativeLlmError> {
        extract(&self.model, JUDGE_PREAMBLE, judge_prompt(request)).await
    }
}

fn judge_prompt(request: &JudgeRequest) -> String {
    let mut prompt = format!(
        "{}\n\nThere are {} candidates. Reply with the zero-based index of the best one.\n",
        request.instruction,
        request.candidates.len()
    );
    for (i, c) in request.candidates.iter().enumerate() {
        let _ = write!(
            prompt,
            "\n<candidate index=\"{i}\" label=\"{}\">\n{}\n</candidate>\n",
            c.label, c.text
        );
    }
    prompt
}

/// Structured output via one forced tool call whose parameters are `T`'s JSON
/// Schema - the same technique as Rig's `Extractor` (in the separate
/// `rig-agent` crate), kept here to depend on `rig-core` alone.
async fn extract<T: JsonSchema + DeserializeOwned>(
    model: &client::JudgeModel,
    preamble: &str,
    prompt: String,
) -> Result<T, NativeLlmError> {
    let tool = submit_tool::<T>()?;
    let response = model
        .send_forced_tool(prompt, preamble, tool, JUDGE_MAX_TOKENS)
        .await
        .map_err(|e| NativeLlmError::Request(e.to_string()))?;
    parse_submit(&response)
}

fn submit_tool<T: JsonSchema>() -> Result<ToolDefinition, NativeLlmError> {
    let mut parameters = serde_json::to_value(schemars::schema_for!(T))
        .map_err(|e| NativeLlmError::InvalidOutput(e.to_string()))?;
    if let Some(obj) = parameters.as_object_mut() {
        obj.remove("$schema");
        obj.remove("title");
    }
    Ok(ToolDefinition {
        name: SUBMIT_TOOL.to_string(),
        description: SUBMIT_DESCRIPTION.to_string(),
        parameters,
    })
}

fn parse_submit<T: DeserializeOwned>(response: &CompletionResponse) -> Result<T, NativeLlmError> {
    let args = response
        .choice
        .iter()
        .find_map(|content| match content {
            AssistantContent::ToolCall(call) if call.function.name == SUBMIT_TOOL => {
                Some(call.function.arguments.clone())
            }
            _ => None,
        })
        .ok_or(NativeLlmError::NoToolCall)?;
    // Some wires carry arguments as a JSON-encoded string.
    let args = match args {
        serde_json::Value::String(s) => {
            serde_json::from_str(&s).map_err(|e| NativeLlmError::InvalidOutput(e.to_string()))?
        }
        other => other,
    };
    serde_json::from_value(args).map_err(|e| NativeLlmError::InvalidOutput(e.to_string()))
}

#[cfg(test)]
mod tests;
