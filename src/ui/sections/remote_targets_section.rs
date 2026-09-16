//! The **Remote Targets** status section: SSH hosts and Coder workspaces, and
//! whether this configuration may currently launch onto them.
//!
//! Deliberately visible without a licence. Hiding it would make Premium look
//! like a missing feature rather than a locked one, and configured targets stay
//! readable - and removable - whatever the licence says. When the configuration
//! is not entitled, the rows are preceded by the paywall panel, which names the
//! feature and where to get it.

use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow,
};

pub struct RemoteTargetsSection;

impl StatusSection for RemoteTargetsSection {
    fn section_id(&self) -> SectionId {
        SectionId::RemoteTargets
    }

    fn label(&self) -> &'static str {
        "Remote Targets"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        if snapshot.remote_targets.is_empty() {
            // Nothing configured is not a fault: local execution is the default.
            SectionHealth::Gray
        } else if snapshot.license.premium {
            SectionHealth::Green
        } else {
            // Configured but unusable is exactly what a warning is for.
            SectionHealth::Yellow
        }
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        let count = snapshot.remote_targets.len();
        match (count, snapshot.license.premium) {
            (0, _) => "Premium · none configured".to_string(),
            (n, true) => format!("{n} configured"),
            (n, false) => format!("{n} configured · license required"),
        }
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        let mut rows = Vec::new();

        if !snapshot.license.premium {
            rows.extend(paywall_rows(snapshot));
        }

        for target in &snapshot.remote_targets {
            rows.push(TreeRow {
                section_id: SectionId::RemoteTargets,
                id: format!("remote-target-{}", target.name),
                depth: 1,
                label: target.name.clone(),
                description: format!(
                    "{} · {} · {}",
                    target.kind,
                    target.detail,
                    if snapshot.license.premium {
                        "available"
                    } else {
                        "license required"
                    }
                ),
                icon: StatusIcon::Plug,
                brand_icon: None,
                is_header: false,
                actions: manage_action(snapshot),
                health: if snapshot.license.premium {
                    SectionHealth::Green
                } else {
                    SectionHealth::Yellow
                },
            });
        }
        rows
    }
}

/// The terminal paywall: what the feature is, why it is unavailable, and the
/// two ways forward. A terminal cannot open a browser for the reader, so the
/// destination is rendered as text.
fn paywall_rows(snapshot: &StatusSnapshot) -> Vec<TreeRow> {
    let mut rows = vec![
        paywall_row(
            "remote-targets-paywall",
            "Remote targets",
            "Premium · available with a license for this configuration",
            StatusIcon::Key,
        ),
        paywall_row(
            "remote-targets-free",
            "Included",
            "Multiple local agents and local containers",
            StatusIcon::Check,
        ),
    ];
    if let Some(url) = snapshot
        .license
        .purchase_url
        .as_ref()
        .filter(|url| url.starts_with("https://"))
    {
        rows.push(TreeRow {
            actions: ActionSet {
                primary: StatusAction::OpenUrl(url.clone()),
                back: StatusAction::None,
                special: StatusAction::None,
                special_meta: None,
                refresh: StatusAction::None,
                refresh_meta: None,
            },
            ..paywall_row(
                "remote-targets-purchase",
                "Get Premium",
                url,
                StatusIcon::Plug,
            )
        });
    }
    rows
}

fn paywall_row(id: &str, label: &str, description: &str, icon: StatusIcon) -> TreeRow {
    TreeRow {
        section_id: SectionId::RemoteTargets,
        id: id.to_string(),
        depth: 1,
        label: label.to_string(),
        description: description.to_string(),
        icon,
        brand_icon: None,
        is_header: false,
        actions: ActionSet {
            primary: StatusAction::None,
            back: StatusAction::None,
            special: StatusAction::None,
            special_meta: None,
            refresh: StatusAction::None,
            refresh_meta: None,
        },
        health: SectionHealth::Yellow,
    }
}

/// Managing targets is a dashboard job; the terminal links to it when the API
/// is up.
fn manage_action(snapshot: &StatusSnapshot) -> ActionSet {
    let primary = match snapshot.api_port() {
        Some(port) => StatusAction::OpenWebUiAt {
            port,
            route: "/remote-targets".into(),
        },
        None => StatusAction::None,
    };
    ActionSet {
        primary,
        back: StatusAction::None,
        special: StatusAction::None,
        special_meta: None,
        refresh: StatusAction::None,
        refresh_meta: None,
    }
}

#[allow(dead_code)]
fn unused(_: ActionMeta) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, SshTarget, TargetDef, TargetKind};

    fn snapshot_with_targets(targets: Vec<TargetDef>) -> StatusSnapshot {
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        config.targets = targets;
        StatusSnapshot::from_config(&config, vec![])
    }

    fn ssh(name: &str) -> TargetDef {
        TargetDef {
            name: name.to_string(),
            display_name: None,
            kind: TargetKind::Ssh(SshTarget {
                ssh_alias: "alias".to_string(),
                workdir: "/srv".to_string(),
                ssh_config_path: None,
            }),
        }
    }

    /// The section stays discoverable without a licence - that is the point.
    #[test]
    fn the_section_is_visible_and_gated_without_a_licence() {
        let snapshot = snapshot_with_targets(vec![ssh("build-host")]);

        assert_eq!(
            RemoteTargetsSection.health(&snapshot),
            SectionHealth::Yellow
        );
        assert!(RemoteTargetsSection
            .description(&snapshot)
            .contains("license required"));

        let rows = RemoteTargetsSection.children(&snapshot);
        assert!(
            rows.iter().any(|r| r.id == "remote-targets-paywall"),
            "an unentitled configuration renders the paywall"
        );
        assert!(
            rows.iter().any(|r| r.label == "build-host"),
            "configured targets stay readable without Premium"
        );
    }

    #[test]
    fn no_configured_targets_is_not_a_fault() {
        let snapshot = snapshot_with_targets(vec![]);
        assert_eq!(RemoteTargetsSection.health(&snapshot), SectionHealth::Gray);
    }

    /// Local and docker are built in; this section is about remote execution.
    #[test]
    fn built_in_targets_are_not_listed() {
        let snapshot = snapshot_with_targets(vec![TargetDef {
            name: "container".to_string(),
            display_name: None,
            kind: TargetKind::Docker(crate::config::DockerConfig::default()),
        }]);

        assert!(snapshot.remote_targets.is_empty());
        assert!(!RemoteTargetsSection
            .children(&snapshot)
            .iter()
            .any(|r| r.label == "container"));
    }
}
