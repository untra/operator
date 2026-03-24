use std::path::Path;

use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow,
};

pub struct ConfigSection;

impl StatusSection for ConfigSection {
    fn section_id(&self) -> SectionId {
        SectionId::Configuration
    }

    fn label(&self) -> &'static str {
        "Configuration"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[] // Always visible
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        if !snapshot.config_file_found {
            return SectionHealth::Red;
        }
        if !snapshot.tickets_dir_exists || snapshot.working_dir.is_empty() {
            return SectionHealth::Yellow;
        }
        SectionHealth::Green
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        if !snapshot.config_file_found {
            return "Config not found".into();
        }
        if !snapshot.tickets_dir_exists {
            return "Tickets dir missing".into();
        }
        if snapshot.working_dir.is_empty() {
            return "Working dir not set".into();
        }
        Path::new(&snapshot.working_dir)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| snapshot.working_dir.clone())
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        vec![
            // Working Dir: primary=open, special=none (must launch from dir), refresh=none
            TreeRow {
                section_id: SectionId::Configuration,
                depth: 1,
                label: "Working Dir".into(),
                description: if snapshot.working_dir.is_empty() {
                    "Not set".into()
                } else {
                    Path::new(&snapshot.working_dir)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| snapshot.working_dir.clone())
                },
                icon: if snapshot.working_dir.is_empty() {
                    StatusIcon::Warning
                } else {
                    StatusIcon::Check
                },
                is_header: false,
                actions: ActionSet::primary(if snapshot.working_dir.is_empty() {
                    StatusAction::None
                } else {
                    StatusAction::OpenDirectory(snapshot.working_dir.clone())
                }),
                health: SectionHealth::Gray,
            },
            // Config: primary=edit, special=reset to defaults, refresh=reload config
            TreeRow {
                section_id: SectionId::Configuration,
                depth: 1,
                label: "Config".into(),
                description: if snapshot.config_file_found {
                    snapshot.config_path.clone()
                } else {
                    "Not found".into()
                },
                icon: if snapshot.config_file_found {
                    StatusIcon::Check
                } else {
                    StatusIcon::Cross
                },
                is_header: false,
                actions: if snapshot.config_file_found {
                    ActionSet {
                        primary: StatusAction::EditFile(snapshot.config_path.clone()),
                        back: StatusAction::None,
                        special: StatusAction::ResetConfig,
                        special_meta: Some(ActionMeta {
                            title: "Reset",
                            tooltip:
                                "Reset configuration to factory defaults (requires confirmation)",
                        }),
                        refresh: StatusAction::ReloadConfig,
                        refresh_meta: Some(ActionMeta {
                            title: "Reload",
                            tooltip: "Reload configuration from disk and restart",
                        }),
                    }
                } else {
                    ActionSet::none()
                },
                health: SectionHealth::Gray,
            },
            // Tickets: primary=open dir, no special or refresh
            TreeRow {
                section_id: SectionId::Configuration,
                depth: 1,
                label: "Tickets".into(),
                description: if snapshot.tickets_dir_exists {
                    snapshot.tickets_dir.clone()
                } else {
                    "Not found".into()
                },
                icon: if snapshot.tickets_dir_exists {
                    StatusIcon::Check
                } else {
                    StatusIcon::Cross
                },
                is_header: false,
                actions: ActionSet::primary(if snapshot.tickets_dir_exists {
                    StatusAction::OpenDirectory(snapshot.tickets_dir.clone())
                } else {
                    StatusAction::None
                }),
                health: SectionHealth::Gray,
            },
            // Wrapper connection status (moved from Connections section)
            {
                let wrapper = &snapshot.wrapper_connection_status;
                TreeRow {
                    section_id: SectionId::Configuration,
                    depth: 1,
                    label: wrapper.label().into(),
                    description: wrapper.description(),
                    icon: if wrapper.is_connected() {
                        StatusIcon::Check
                    } else {
                        StatusIcon::Cross
                    },
                    is_header: false,
                    actions: ActionSet {
                        primary: if wrapper.is_connected() {
                            StatusAction::None
                        } else {
                            StatusAction::RestartWrapperConnection
                        },
                        back: StatusAction::None,
                        special: StatusAction::None,
                        special_meta: None,
                        refresh: StatusAction::RestartWrapperConnection,
                        refresh_meta: Some(ActionMeta {
                            title: "Retry",
                            tooltip: "Reconnect the session wrapper",
                        }),
                    },
                    health: SectionHealth::Gray,
                }
            },
            // Wrapper type: display-only
            TreeRow {
                section_id: SectionId::Configuration,
                depth: 1,
                label: "Wrapper".into(),
                description: snapshot.wrapper_type.clone(),
                icon: StatusIcon::Tool,
                is_header: false,
                actions: ActionSet::none(),
                health: SectionHealth::Gray,
            },
            // $EDITOR: display-only
            TreeRow {
                section_id: SectionId::Configuration,
                depth: 1,
                label: "$EDITOR".into(),
                description: if snapshot.env_editor.is_empty() {
                    "Not set".into()
                } else {
                    snapshot.env_editor.clone()
                },
                icon: if snapshot.env_editor.is_empty() {
                    StatusIcon::Warning
                } else {
                    StatusIcon::Check
                },
                is_header: false,
                actions: ActionSet::none(),
                health: SectionHealth::Gray,
            },
            // $VISUAL: display-only
            TreeRow {
                section_id: SectionId::Configuration,
                depth: 1,
                label: "$VISUAL".into(),
                description: if snapshot.env_visual.is_empty() {
                    "Not set".into()
                } else {
                    snapshot.env_visual.clone()
                },
                icon: if snapshot.env_visual.is_empty() {
                    StatusIcon::Warning
                } else {
                    StatusIcon::Check
                },
                is_header: false,
                actions: ActionSet::none(),
                health: SectionHealth::Gray,
            },
            // Version: primary=open downloads, refresh=check for updates
            TreeRow {
                section_id: SectionId::Configuration,
                depth: 1,
                label: "Version".into(),
                description: if let Some(ref update) = snapshot.update_available_version {
                    format!("{} → {} available", snapshot.operator_version, update)
                } else {
                    snapshot.operator_version.clone()
                },
                icon: if snapshot.update_available_version.is_some() {
                    StatusIcon::Warning
                } else {
                    StatusIcon::None
                },
                is_header: false,
                actions: ActionSet {
                    primary: StatusAction::OpenUrl("https://operator.untra.io/downloads/".into()),
                    back: StatusAction::None,
                    special: StatusAction::None,
                    special_meta: None,
                    refresh: StatusAction::RefreshSection(SectionId::Configuration),
                    refresh_meta: Some(ActionMeta {
                        title: "Check",
                        tooltip: "Check for new operator versions",
                    }),
                },
                health: SectionHealth::Gray,
            },
        ]
    }
}
