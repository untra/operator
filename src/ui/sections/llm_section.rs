use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow,
};

pub struct LlmSection;

impl StatusSection for LlmSection {
    fn section_id(&self) -> SectionId {
        SectionId::LlmTools
    }

    fn label(&self) -> &'static str {
        "LLM Tools"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[SectionId::Connections]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        if snapshot.llm_tools.is_empty() {
            SectionHealth::Yellow
        } else {
            SectionHealth::Green
        }
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        match (&snapshot.default_llm_tool, &snapshot.default_llm_model) {
            (Some(tool), Some(model)) => format!("Default: {tool}:{model}"),
            (Some(tool), None) => format!("Default: {tool}"),
            _ => snapshot
                .llm_tools
                .first()
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "No tools detected".into()),
        }
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        let mut rows = Vec::new();

        for tool in &snapshot.llm_tools {
            // Depth 1: tool name + version
            rows.push(TreeRow {
                section_id: SectionId::LlmTools,
                id: tool.name.clone(),
                depth: 1,
                label: tool.name.clone(),
                description: tool.version.clone(),
                icon: StatusIcon::Tool,
                is_header: false,
                actions: ActionSet {
                    primary: StatusAction::None,
                    back: StatusAction::None,
                    special: StatusAction::EditFile(snapshot.config_path.clone()),
                    special_meta: Some(ActionMeta {
                        title: "Config",
                        tooltip: "Edit LLM tool configuration",
                    }),
                    refresh: StatusAction::None,
                    refresh_meta: None,
                },
                health: SectionHealth::Gray,
            });

            // Depth 2: model aliases — selecting sets as default
            for model in &tool.model_aliases {
                let is_default = snapshot.default_llm_tool.as_deref() == Some(&tool.name)
                    && snapshot.default_llm_model.as_deref() == Some(model.as_str());
                let icon = if is_default {
                    StatusIcon::Check
                } else {
                    StatusIcon::Key
                };
                let label = if is_default {
                    format!("{model} (default)")
                } else {
                    model.clone()
                };

                rows.push(TreeRow {
                    section_id: SectionId::LlmTools,
                    id: format!("{}:{}", tool.name, model),
                    depth: 2,
                    label,
                    description: format!("{}:{}", tool.name, model),
                    icon,
                    is_header: false,
                    actions: ActionSet::primary(StatusAction::SetDefaultLlm {
                        tool_name: tool.name.clone(),
                        model: model.clone(),
                    }),
                    health: SectionHealth::Gray,
                });
            }
        }

        rows
    }
}
