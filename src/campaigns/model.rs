//! Campaign types: an epic of tickets planned and launched together.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::GovernancePolicy;

/// Lifecycle of a campaign.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    TS,
    utoipa::ToSchema,
)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStatus {
    #[default]
    Draft,
    Ready,
    Running,
    Paused,
    Completed,
    Cancelled,
}

/// One execution environment shared by every ticket in a campaign.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CampaignWorkspace {
    /// One Coder workspace created from this template.
    Coder { template: String },
    /// One container from this image.
    Docker { image: String },
}

/// A campaign, stored as frontmatter plus a markdown body under `<tickets>/campaigns/`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Campaign {
    /// Campaign id (e.g. `CAMP-0003`); member tickets reference it as `campaign:`.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    #[serde(default)]
    pub status: CampaignStatus,
    /// Creation date (`YYYY-MM-DD`).
    #[serde(default)]
    pub created: String,
    /// Name of a `[[governance]]` policy layered over each delegator's own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance: Option<String>,
    /// Inline policy layered over `governance`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_overrides: Option<GovernancePolicy>,
    /// Shared execution environment; absent runs each ticket in its own workspace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<CampaignWorkspace>,
    /// Upstream container this campaign was synced from (e.g. `jira:PROJ-1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    /// Markdown body after the frontmatter: the campaign's goal.
    #[serde(skip)]
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_is_tagged_by_kind() {
        let ws: CampaignWorkspace =
            serde_yaml::from_str("kind: coder\ntemplate: operator-agent").unwrap();
        assert_eq!(
            ws,
            CampaignWorkspace::Coder {
                template: "operator-agent".into()
            }
        );
        let ws: CampaignWorkspace = serde_yaml::from_str("kind: docker\nimage: img:1").unwrap();
        assert_eq!(
            ws,
            CampaignWorkspace::Docker {
                image: "img:1".into()
            }
        );
    }

    #[test]
    fn test_campaign_minimal_frontmatter_defaults_to_draft() {
        let c: Campaign = serde_yaml::from_str("id: CAMP-0001\ntitle: T").unwrap();
        assert_eq!(c.status, CampaignStatus::Draft);
        assert!(c.workspace.is_none() && c.governance.is_none());
    }
}
