use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow,
};

pub struct DelegatorSection;

impl StatusSection for DelegatorSection {
    fn section_id(&self) -> SectionId {
        SectionId::Delegators
    }

    fn label(&self) -> &'static str {
        "Delegators"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[SectionId::LlmTools]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        if snapshot.delegators.is_empty() {
            SectionHealth::Yellow
        } else {
            SectionHealth::Green
        }
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        let count = snapshot.delegators.len();
        if count == 0 {
            "None configured".into()
        } else {
            format!("{count} delegator{}", if count == 1 { "" } else { "s" })
        }
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        snapshot
            .delegators
            .iter()
            .map(|d| {
                let label = d.display_name.as_deref().unwrap_or(&d.name).to_string();
                let yolo_flag = if d.yolo { " · yolo" } else { "" };
                let description = format!("{}:{}{}", d.llm_tool, d.model, yolo_flag);

                TreeRow {
                    section_id: SectionId::Delegators,
                    depth: 1,
                    label,
                    description,
                    icon: StatusIcon::Tool,
                    is_header: false,
                    actions: ActionSet {
                        primary: StatusAction::None,
                        back: StatusAction::None,
                        special: StatusAction::EditFile(snapshot.config_path.clone()),
                        special_meta: Some(ActionMeta {
                            title: "Config",
                            tooltip: "Edit delegator configuration",
                        }),
                        refresh: StatusAction::None,
                        refresh_meta: None,
                    },
                    health: SectionHealth::Gray,
                }
            })
            .collect()
    }
}
