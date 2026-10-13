//! Governance: named, layerable launch policies.
//!
//! A `[[governance]]` policy is referenced by name from a delegator (and, later,
//! a campaign). Layered policies intersect, so the most restrictive rule wins.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A named launch policy. Every constraint is optional; an absent one is unconstrained.
#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema,
)]
#[ts(export)]
pub struct GovernancePolicy {
    /// Unique name, referenced by `Delegator.governance`.
    pub name: String,
    /// Delegator names allowed to launch. Absent = any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_delegators: Option<Vec<String>>,
    /// LLM tool names allowed to launch (e.g. `claude`). Absent = any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_llm_tools: Option<Vec<String>>,
    /// Model server names allowed (declared or implicit, e.g. `anthropic-api`). Absent = any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_model_servers: Option<Vec<String>>,
    /// Execution target names allowed (e.g. `local`, `docker`, a `[[targets]]` name). Absent = any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_targets: Option<Vec<String>>,
    /// Maximum concurrently running agents under this policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_parallel: Option<u32>,
    /// Maximum total launches under this policy (counted per campaign run).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_launches: Option<u32>,
    /// Maximum relaunches of a failed ticket (counted per campaign run).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    /// Maximum wall-clock minutes per agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_runtime_minutes: Option<u32>,
    /// Spend ceiling in USD. Reserved: accepted and reported, not yet enforced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_usd: Option<f64>,
    /// Require operator confirmation before each launch.
    #[serde(default)]
    pub require_confirmation: bool,
}

/// What a launch is about to use, checked against a policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LaunchFacts {
    pub delegator: Option<String>,
    pub llm_tool: Option<String>,
    pub model_server: Option<String>,
    pub target: String,
}

/// Usage already counted against a policy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GovernanceCounters {
    pub active: u32,
    pub launched: u32,
    pub retries: u32,
}

/// A launch refused by governance.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Governance policy '{policy}' denied launch: {reason}")]
pub struct GovernanceViolation {
    pub policy: String,
    pub reason: String,
}

fn intersect(a: Option<&Vec<String>>, b: Option<&Vec<String>>) -> Option<Vec<String>> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) | (None, Some(x)) => Some(x.clone()),
        (Some(x), Some(y)) => Some(x.iter().filter(|v| y.contains(v)).cloned().collect()),
    }
}

fn min_cap<T: PartialOrd + Copy>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(x), Some(y)) => Some(if y < x { y } else { x }),
        (x, y) => x.or(y),
    }
}

/// Stack policies into one effective policy; the most restrictive rule wins.
pub fn layer(policies: &[&GovernancePolicy]) -> GovernancePolicy {
    let name = policies
        .iter()
        .map(|p| p.name.as_str())
        .collect::<Vec<_>>()
        .join("+");
    policies.iter().fold(
        GovernancePolicy {
            name,
            ..GovernancePolicy::default()
        },
        |acc, p| GovernancePolicy {
            name: acc.name,
            allowed_delegators: intersect(
                acc.allowed_delegators.as_ref(),
                p.allowed_delegators.as_ref(),
            ),
            allowed_llm_tools: intersect(
                acc.allowed_llm_tools.as_ref(),
                p.allowed_llm_tools.as_ref(),
            ),
            allowed_model_servers: intersect(
                acc.allowed_model_servers.as_ref(),
                p.allowed_model_servers.as_ref(),
            ),
            allowed_targets: intersect(acc.allowed_targets.as_ref(), p.allowed_targets.as_ref()),
            max_parallel: min_cap(acc.max_parallel, p.max_parallel),
            max_launches: min_cap(acc.max_launches, p.max_launches),
            max_retries: min_cap(acc.max_retries, p.max_retries),
            max_runtime_minutes: min_cap(acc.max_runtime_minutes, p.max_runtime_minutes),
            budget_usd: min_cap(acc.budget_usd, p.budget_usd),
            require_confirmation: acc.require_confirmation || p.require_confirmation,
        },
    )
}

fn check_allowed(
    policy: &str,
    kind: &str,
    allowed: Option<&Vec<String>>,
    value: Option<&str>,
) -> Result<(), GovernanceViolation> {
    let Some(allowed) = allowed else {
        return Ok(());
    };
    match value {
        Some(v) if allowed.iter().any(|a| a == v) => Ok(()),
        Some(v) => Err(GovernanceViolation {
            policy: policy.to_string(),
            reason: format!(
                "{kind} '{v}' is not in allowed {kind}s [{}]",
                allowed.join(", ")
            ),
        }),
        None => Err(GovernanceViolation {
            policy: policy.to_string(),
            reason: format!("launch has no {kind}, but the policy restricts {kind}s"),
        }),
    }
}

fn check_cap(
    policy: &str,
    kind: &str,
    cap: Option<u32>,
    used: u32,
) -> Result<(), GovernanceViolation> {
    match cap {
        Some(cap) if used >= cap => Err(GovernanceViolation {
            policy: policy.to_string(),
            reason: format!("{kind} limit of {cap} reached"),
        }),
        _ => Ok(()),
    }
}

/// Check one launch against a policy and its counted usage.
pub fn evaluate(
    policy: &GovernancePolicy,
    facts: &LaunchFacts,
    counters: &GovernanceCounters,
) -> Result<(), GovernanceViolation> {
    let name = policy.name.as_str();
    check_allowed(
        name,
        "delegator",
        policy.allowed_delegators.as_ref(),
        facts.delegator.as_deref(),
    )?;
    check_allowed(
        name,
        "llm_tool",
        policy.allowed_llm_tools.as_ref(),
        facts.llm_tool.as_deref(),
    )?;
    check_allowed(
        name,
        "model_server",
        policy.allowed_model_servers.as_ref(),
        facts.model_server.as_deref(),
    )?;
    check_allowed(
        name,
        "target",
        policy.allowed_targets.as_ref(),
        Some(&facts.target),
    )?;
    check_cap(name, "max_parallel", policy.max_parallel, counters.active)?;
    check_cap(name, "max_launches", policy.max_launches, counters.launched)?;
    check_cap(name, "max_retries", policy.max_retries, counters.retries)?;
    Ok(())
}

/// Find a named policy; an unknown name is an error, never a silent pass.
pub fn find_policy<'a>(
    policies: &'a [GovernancePolicy],
    name: &str,
) -> Result<&'a GovernancePolicy, GovernanceViolation> {
    policies
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| GovernanceViolation {
            policy: name.to_string(),
            reason: "no [[governance]] policy with that name".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strs(v: &[&str]) -> Option<Vec<String>> {
        Some(v.iter().map(|s| (*s).to_string()).collect())
    }

    fn facts(target: &str) -> LaunchFacts {
        LaunchFacts {
            delegator: Some("opus".into()),
            llm_tool: Some("claude".into()),
            model_server: Some("anthropic-api".into()),
            target: target.into(),
        }
    }

    #[test]
    fn test_evaluate_empty_policy_allows_everything() {
        let policy = GovernancePolicy::default();
        assert!(evaluate(&policy, &facts("coder"), &GovernanceCounters::default()).is_ok());
    }

    #[test]
    fn test_evaluate_disallowed_target_is_denied() {
        let policy = GovernancePolicy {
            name: "local-only".into(),
            allowed_targets: strs(&["local", "docker"]),
            ..Default::default()
        };
        let err = evaluate(&policy, &facts("coder"), &GovernanceCounters::default()).unwrap_err();
        assert_eq!(err.policy, "local-only");
        assert!(err.reason.contains("target 'coder'"));
        assert!(evaluate(&policy, &facts("docker"), &GovernanceCounters::default()).is_ok());
    }

    #[test]
    fn test_evaluate_restricted_kind_missing_from_launch_is_denied() {
        let policy = GovernancePolicy {
            name: "named".into(),
            allowed_delegators: strs(&["opus"]),
            ..Default::default()
        };
        let adhoc = LaunchFacts {
            delegator: None,
            ..facts("local")
        };
        assert!(evaluate(&policy, &adhoc, &GovernanceCounters::default()).is_err());
    }

    #[test]
    fn test_evaluate_caps_deny_at_limit() {
        let policy = GovernancePolicy {
            name: "capped".into(),
            max_parallel: Some(2),
            max_launches: Some(5),
            ..Default::default()
        };
        let under = GovernanceCounters {
            active: 1,
            launched: 4,
            retries: 0,
        };
        assert!(evaluate(&policy, &facts("local"), &under).is_ok());
        let full = GovernanceCounters { active: 2, ..under };
        assert!(evaluate(&policy, &facts("local"), &full)
            .unwrap_err()
            .reason
            .contains("max_parallel"));
        let spent = GovernanceCounters {
            launched: 5,
            ..under
        };
        assert!(evaluate(&policy, &facts("local"), &spent)
            .unwrap_err()
            .reason
            .contains("max_launches"));
    }

    #[test]
    fn test_layer_intersects_lists_and_takes_minimum_caps() {
        let campaign = GovernancePolicy {
            name: "campaign".into(),
            allowed_targets: strs(&["local", "coder"]),
            max_parallel: Some(4),
            budget_usd: Some(50.0),
            ..Default::default()
        };
        let delegator = GovernancePolicy {
            name: "delegator".into(),
            allowed_targets: strs(&["coder", "ssh"]),
            allowed_llm_tools: strs(&["claude"]),
            max_parallel: Some(2),
            require_confirmation: true,
            ..Default::default()
        };
        let effective = layer(&[&campaign, &delegator]);
        assert_eq!(effective.name, "campaign+delegator");
        assert_eq!(effective.allowed_targets, strs(&["coder"]));
        assert_eq!(effective.allowed_llm_tools, strs(&["claude"]));
        assert_eq!(effective.allowed_delegators, None);
        assert_eq!(effective.max_parallel, Some(2));
        assert_eq!(effective.budget_usd, Some(50.0));
        assert!(effective.require_confirmation);
    }

    #[test]
    fn test_layer_disjoint_lists_allow_nothing() {
        let a = GovernancePolicy {
            name: "a".into(),
            allowed_targets: strs(&["local"]),
            ..Default::default()
        };
        let b = GovernancePolicy {
            name: "b".into(),
            allowed_targets: strs(&["coder"]),
            ..Default::default()
        };
        let effective = layer(&[&a, &b]);
        assert_eq!(effective.allowed_targets, Some(vec![]));
        assert!(evaluate(&effective, &facts("local"), &GovernanceCounters::default()).is_err());
    }

    #[test]
    fn test_find_policy_unknown_name_errors() {
        let policies = vec![GovernancePolicy {
            name: "strict".into(),
            ..Default::default()
        }];
        assert!(find_policy(&policies, "strict").is_ok());
        assert!(find_policy(&policies, "missing").is_err());
    }

    #[test]
    fn test_policy_deserializes_from_toml_with_falsey_defaults() {
        let policy: GovernancePolicy = toml::from_str(
            r#"
            name = "team"
            allowed_targets = ["local"]
            max_parallel = 3
            "#,
        )
        .unwrap();
        assert_eq!(policy.max_parallel, Some(3));
        assert!(!policy.require_confirmation);
        assert_eq!(policy.budget_usd, None);
    }
}
