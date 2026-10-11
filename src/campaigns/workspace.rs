//! Resolve a campaign's shared workspace to the target its tickets launch on.

use super::model::{Campaign, CampaignWorkspace};
use crate::config::{CoderConfig, Config, TargetDef, TargetKind, DEFAULT_CODER_TARGET_NAME};

/// Where every ticket of a shared-workspace campaign runs.
///
/// Tickets still get their own worktree and branch inside it, so a wave can run in parallel.
#[derive(Debug, Clone, PartialEq)]
pub struct CampaignTarget {
    pub target: TargetDef,
    /// Keys the one shared environment (e.g. the Coder workspace name) instead of a ticket id.
    pub workspace_key: String,
}

/// `None` when the campaign has no shared workspace; tickets resolve targets as usual.
pub fn resolve_workspace(config: &Config, campaign: &Campaign) -> Option<CampaignTarget> {
    let target = match campaign.workspace.as_ref()? {
        CampaignWorkspace::Coder { template } => {
            let (name, base) = config
                .targets
                .iter()
                .find_map(|t| match &t.kind {
                    TargetKind::Coder(c) => Some((t.name.clone(), c.clone())),
                    _ => None,
                })
                .unwrap_or_else(|| {
                    (
                        DEFAULT_CODER_TARGET_NAME.to_string(),
                        CoderConfig::default(),
                    )
                });
            TargetDef {
                name,
                display_name: None,
                kind: TargetKind::Coder(CoderConfig {
                    template: template.clone(),
                    ..base
                }),
            }
        }
        CampaignWorkspace::Docker { image } => {
            let mut docker = config.launch.docker.clone();
            docker.image = image.clone();
            TargetDef::docker(docker)
        }
    };
    Some(CampaignTarget {
        target,
        workspace_key: campaign.id.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaigns::model::CampaignStatus;

    fn campaign(workspace: Option<CampaignWorkspace>) -> Campaign {
        Campaign {
            id: "CAMP-0003".into(),
            title: "T".into(),
            status: CampaignStatus::Draft,
            created: String::new(),
            governance: None,
            governance_overrides: None,
            workspace,
            external_ref: None,
            body: String::new(),
        }
    }

    #[test]
    fn test_no_workspace_resolves_to_none() {
        assert!(resolve_workspace(&Config::default(), &campaign(None)).is_none());
    }

    #[test]
    fn test_coder_template_overrides_configured_target() {
        let mut config = Config::default();
        config.targets.push(TargetDef {
            name: "cloud".into(),
            display_name: None,
            kind: TargetKind::Coder(CoderConfig {
                template: "default-tpl".into(),
                name_prefix: "acme".into(),
                ..CoderConfig::default()
            }),
        });
        let resolved = resolve_workspace(
            &config,
            &campaign(Some(CampaignWorkspace::Coder {
                template: "campaign-tpl".into(),
            })),
        )
        .unwrap();
        assert_eq!(resolved.workspace_key, "CAMP-0003");
        assert_eq!(resolved.target.name, "cloud");
        let TargetKind::Coder(coder) = resolved.target.kind else {
            panic!("expected coder");
        };
        assert_eq!(coder.template, "campaign-tpl");
        assert_eq!(coder.name_prefix, "acme");
    }

    #[test]
    fn test_coder_without_configured_target_uses_defaults() {
        let resolved = resolve_workspace(
            &Config::default(),
            &campaign(Some(CampaignWorkspace::Coder {
                template: "tpl".into(),
            })),
        )
        .unwrap();
        assert_eq!(resolved.target.name, DEFAULT_CODER_TARGET_NAME);
    }

    #[test]
    fn test_docker_image_override() {
        let resolved = resolve_workspace(
            &Config::default(),
            &campaign(Some(CampaignWorkspace::Docker {
                image: "untra/operator:camp".into(),
            })),
        )
        .unwrap();
        let TargetKind::Docker(docker) = resolved.target.kind else {
            panic!("expected docker");
        };
        assert_eq!(docker.image, "untra/operator:camp");
    }

    #[test]
    fn test_shared_coder_still_needs_remote_entitlement() {
        let resolved = resolve_workspace(
            &Config::default(),
            &campaign(Some(CampaignWorkspace::Coder {
                template: "tpl".into(),
            })),
        )
        .unwrap();
        assert!(crate::licensing::require_target(&Config::default(), &resolved.target).is_err());
    }
}
