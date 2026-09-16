//! The **License** status section: which tier this configuration runs at, and
//! the verified terms behind that answer.
//!
//! Read-only. Installing a licence happens in the setup wizard or the web
//! dashboard; this section exists so the terminal can answer "am I licensed,
//! and until when?" without either.
//!
//! Health maps the licence status to a semantic role rather than to the word:
//! a missing licence is the free tier working as intended, not a fault.

use crate::licensing::LicenseStatus;
use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow,
};

pub struct LicenseSection;

fn label_and_health(status: LicenseStatus) -> (&'static str, SectionHealth) {
    match status {
        LicenseStatus::Valid => ("Premium", SectionHealth::Green),
        LicenseStatus::Missing => ("Free", SectionHealth::Gray),
        LicenseStatus::Expired => ("Expired", SectionHealth::Yellow),
        LicenseStatus::NotYetValid => ("Not yet valid", SectionHealth::Yellow),
        LicenseStatus::Invalid => ("Invalid", SectionHealth::Red),
    }
}

fn timestamp(seconds: i64) -> String {
    chrono::DateTime::from_timestamp(seconds, 0)
        .map_or_else(|| "-".to_string(), |t| t.format("%Y-%m-%d").to_string())
}

impl StatusSection for LicenseSection {
    fn section_id(&self) -> SectionId {
        SectionId::License
    }

    fn label(&self) -> &'static str {
        "License"
    }

    fn prerequisites(&self) -> &[SectionId] {
        // Always answerable: the free tier is a valid answer.
        &[]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        label_and_health(snapshot.license.status).1
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        let (label, _) = label_and_health(snapshot.license.status);
        match &snapshot.license.terms {
            Some(terms) => format!("{label} · expires {}", timestamp(terms.exp)),
            None => format!("{label} · local execution included"),
        }
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        let health = self.health(snapshot);
        let mut rows = vec![row(
            "license-configuration",
            "Configuration",
            &snapshot.license.profile_id.to_string(),
            StatusIcon::Key,
            health,
        )];

        if let Some(terms) = &snapshot.license.terms {
            rows.push(row(
                "license-subject",
                "Licensed to",
                &terms.sub,
                StatusIcon::Check,
                health,
            ));
            rows.push(row(
                "license-id",
                "License ID",
                &terms.jti,
                StatusIcon::File,
                health,
            ));
            rows.push(row(
                "license-validity",
                "Valid",
                &format!("{} to {}", timestamp(terms.nbf), timestamp(terms.exp)),
                StatusIcon::File,
                health,
            ));
        } else {
            rows.push(row(
                "license-free",
                "Included",
                "Multiple local agents and local containers",
                StatusIcon::Check,
                SectionHealth::Gray,
            ));
        }

        if let Some(url) = purchase_url(snapshot) {
            rows.push(TreeRow {
                section_id: SectionId::License,
                id: "license-purchase".to_string(),
                depth: 1,
                label: "Operator Premium".to_string(),
                description: url.clone(),
                icon: StatusIcon::Plug,
                brand_icon: None,
                is_header: false,
                actions: ActionSet {
                    primary: StatusAction::OpenUrl(url),
                    back: StatusAction::None,
                    special: StatusAction::None,
                    special_meta: None,
                    refresh: StatusAction::None,
                    refresh_meta: None,
                },
                health: SectionHealth::Gray,
            });
        }
        rows
    }
}

/// Only an https destination is ever offered as a link.
fn purchase_url(snapshot: &StatusSnapshot) -> Option<String> {
    snapshot
        .license
        .purchase_url
        .as_ref()
        .filter(|url| url.starts_with("https://"))
        .cloned()
}

fn row(
    id: &str,
    label: &str,
    description: &str,
    icon: StatusIcon,
    health: SectionHealth,
) -> TreeRow {
    TreeRow {
        section_id: SectionId::License,
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
        health,
    }
}

#[allow(dead_code)]
fn unused(_: ActionMeta) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn a_missing_licence_reads_as_free_not_broken() {
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        let snapshot = StatusSnapshot::from_config(&config, vec![]);

        assert_eq!(LicenseSection.health(&snapshot), SectionHealth::Gray);
        assert!(LicenseSection.description(&snapshot).starts_with("Free"));
        assert!(LicenseSection.prerequisites().is_empty());
    }

    #[test]
    fn the_configuration_binding_is_always_shown() {
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        let snapshot = StatusSnapshot::from_config(&config, vec![]);

        let rows = LicenseSection.children(&snapshot);
        assert!(rows.iter().any(|r| r.label == "Configuration"));
    }
}
