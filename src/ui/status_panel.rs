use std::collections::{HashMap, HashSet};

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::backstage::ServerStatus;
use crate::rest::RestApiStatus;

use super::sections::{
    ConfigSection, ConnectionsSection, DelegatorSection, GitSection, KanbanSection, LlmSection,
};

// ---------------------------------------------------------------------------
// Shared types (exported to TypeScript via ts-rs)
// ---------------------------------------------------------------------------

/// Identifies a collapsible section in the status tree.
///
/// String values match the `sectionId` used in the `VSCode` extension tree routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SectionId {
    #[serde(rename = "config")]
    Configuration,
    #[serde(rename = "connections")]
    Connections,
    #[serde(rename = "kanban")]
    Kanban,
    #[serde(rename = "llm")]
    LlmTools,
    #[serde(rename = "git")]
    Git,
    #[serde(rename = "issuetypes")]
    IssueTypes,
    #[serde(rename = "delegators")]
    Delegators,
    #[serde(rename = "projects")]
    ManagedProjects,
}

/// Health state of a section — controls the header color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SectionHealth {
    /// All good
    Green,
    /// Needs attention
    Yellow,
    /// Broken / missing
    Red,
    /// Info-only / not applicable
    Gray,
}

impl SectionHealth {
    pub fn to_color(self) -> Color {
        match self {
            SectionHealth::Green => Color::Rgb(0, 200, 83),
            SectionHealth::Yellow => Color::Rgb(255, 193, 7),
            SectionHealth::Red => Color::Rgb(244, 67, 54),
            SectionHealth::Gray => Color::Gray,
        }
    }
}

/// Declarative section metadata — shared between TUI and `VSCode`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[allow(dead_code)]
pub struct SectionDefinition {
    pub id: SectionId,
    pub label: String,
    pub prerequisites: Vec<SectionId>,
}

// ---------------------------------------------------------------------------
// Icon enum
// ---------------------------------------------------------------------------

/// Icon rendered beside a tree row.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum StatusIcon {
    Check,
    Cross,
    Warning,
    Folder,
    File,
    Plug,
    Key,
    Branch,
    Tool,
    None,
}

impl StatusIcon {
    pub fn as_span(self) -> Span<'static> {
        match self {
            StatusIcon::Check => Span::styled("✓ ", Style::default().fg(Color::Green)),
            StatusIcon::Cross => Span::styled("✗ ", Style::default().fg(Color::Red)),
            StatusIcon::Warning => Span::styled("⚠ ", Style::default().fg(Color::Yellow)),
            StatusIcon::Folder => Span::styled("D ", Style::default().fg(Color::Cyan)),
            StatusIcon::File => Span::styled("F ", Style::default().fg(Color::White)),
            StatusIcon::Plug => Span::styled("C ", Style::default().fg(Color::Green)),
            StatusIcon::Key => Span::styled("K ", Style::default().fg(Color::Yellow)),
            StatusIcon::Branch => Span::styled("⑂ ", Style::default().fg(Color::Cyan)),
            StatusIcon::Tool => Span::styled("T ", Style::default().fg(Color::Magenta)),
            StatusIcon::None => Span::raw("  "),
        }
    }
}

// ---------------------------------------------------------------------------
// Tree row and action
// ---------------------------------------------------------------------------

/// A single visible row in the status tree.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TreeRow {
    pub section_id: SectionId,
    pub depth: u16,
    pub label: String,
    pub description: String,
    pub icon: StatusIcon,
    pub is_header: bool,
    pub actions: ActionSet,
    pub health: SectionHealth,
}

/// Action to perform when a button is pressed on a status panel row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusAction {
    /// Toggle expand/collapse of a section header
    ToggleSection(SectionId),
    /// Open a directory in the OS file browser (`open` on macOS, `xdg-open` on Linux)
    OpenDirectory(String),
    /// Open a file in `$VISUAL` / `$EDITOR`
    EditFile(String),
    /// Open a URL in the default browser
    OpenUrl(String),
    /// Start the REST API server (without backstage)
    StartApi,
    /// Open Swagger UI for the running API
    OpenSwagger { port: u16 },
    /// Restart the session wrapper connection
    RestartWrapperConnection,
    /// Toggle the web servers (backstage + REST API)
    ToggleWebServers,
    /// Set the global default LLM tool and model
    SetDefaultLlm { tool_name: String, model: String },
    /// Open onboarding for a kanban provider (e.g. "jira", "linear")
    ConfigureKanbanProvider { provider: String },
    /// Open setup page for a git provider (e.g. "github", "gitlab")
    ConfigureGitProvider { provider: String },
    /// Re-check a specific section's health status
    RefreshSection(SectionId),
    /// Reset config to factory defaults (TUI: double-confirm dialog)
    ResetConfig,
    /// Reload config from disk and restart operator experience
    ReloadConfig,
    /// No action available for this row
    None,
}

/// Which button was pressed — maps to ABXY gamepad layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionButton {
    /// A (Enter) — primary/affirm/activate
    A,
    /// B (Esc/Backspace) — go back, collapse parent
    B,
    /// X (Shift+Enter) — special/tertiary action
    X,
    /// Y (Ctrl+Enter) — contextual refresh/update
    Y,
}

/// Display metadata for an action — short title for TUI and title+tooltip for `VSCode`.
#[derive(Debug, Clone)]
pub struct ActionMeta {
    /// Short label (max 6 chars) shown right-aligned on the selected row in TUI,
    /// and as the command title in `VSCode`.
    pub title: &'static str,
    /// Sentence description shown as tooltip in `VSCode` and in the help dialog.
    #[allow(dead_code)]
    pub tooltip: &'static str,
}

/// Four action slots mapped to ABXY gamepad buttons.
#[derive(Debug, Clone)]
pub struct ActionSet {
    /// A (Enter) — primary/affirm/activate
    pub primary: StatusAction,
    /// B (Esc) — go back, collapse parent
    pub back: StatusAction,
    /// X (Shift+Enter) — special/tertiary
    pub special: StatusAction,
    /// Display metadata for the special action (shown in TUI and `VSCode`).
    pub special_meta: Option<ActionMeta>,
    /// Y (Ctrl+Enter) — contextual refresh
    pub refresh: StatusAction,
    /// Display metadata for the refresh action.
    pub refresh_meta: Option<ActionMeta>,
}

impl ActionSet {
    /// Create an action set with only a primary action; others default to None.
    pub fn primary(action: StatusAction) -> Self {
        Self {
            primary: action,
            back: StatusAction::None,
            special: StatusAction::None,
            special_meta: None,
            refresh: StatusAction::None,
            refresh_meta: None,
        }
    }

    /// All actions are None.
    pub fn none() -> Self {
        Self::primary(StatusAction::None)
    }

    /// Select an action by button.
    pub fn for_button(&self, button: ActionButton) -> &StatusAction {
        match button {
            ActionButton::A => &self.primary,
            ActionButton::B => &self.back,
            ActionButton::X => &self.special,
            ActionButton::Y => &self.refresh,
        }
    }

    /// Get the short title for the special action, or `"*"` as fallback.
    pub fn special_title(&self) -> &str {
        self.special_meta.as_ref().map(|m| m.title).unwrap_or("*")
    }

    /// Get the short title for the refresh action, or `"⟳"` as fallback.
    pub fn refresh_title(&self) -> &str {
        self.refresh_meta
            .as_ref()
            .map(|m| m.title)
            .unwrap_or("\u{27F3}")
    }
}

// ---------------------------------------------------------------------------
// Snapshot data
// ---------------------------------------------------------------------------

/// Information about a configured kanban provider.
#[derive(Debug, Clone)]
pub struct KanbanProviderInfo {
    pub provider_type: String,
    pub domain: String,
}

/// Information about a configured LLM tool.
#[derive(Debug, Clone)]
pub struct LlmToolInfo {
    pub name: String,
    pub version: String,
    pub model_aliases: Vec<String>,
}

/// Information about a configured delegator.
#[derive(Debug, Clone)]
pub struct DelegatorInfo {
    pub name: String,
    pub display_name: Option<String>,
    pub llm_tool: String,
    pub model: String,
    pub yolo: bool,
}

/// Connection status for the active session wrapper.
#[derive(Debug, Clone)]
pub enum WrapperConnectionStatus {
    Tmux {
        available: bool,
        server_running: bool,
        version: Option<String>,
    },
    Vscode {
        webhook_running: bool,
        port: Option<u16>,
    },
    Cmux {
        binary_available: bool,
        in_cmux: bool,
    },
    Zellij {
        binary_available: bool,
        in_zellij: bool,
    },
}

impl WrapperConnectionStatus {
    pub fn is_connected(&self) -> bool {
        match self {
            Self::Tmux {
                available,
                server_running,
                ..
            } => *available && *server_running,
            Self::Vscode {
                webhook_running, ..
            } => *webhook_running,
            Self::Cmux {
                binary_available,
                in_cmux,
            } => *binary_available && *in_cmux,
            Self::Zellij {
                binary_available,
                in_zellij,
            } => *binary_available && *in_zellij,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Tmux { .. } => "tmux",
            Self::Vscode { .. } => "vscode",
            Self::Cmux { .. } => "cmux",
            Self::Zellij { .. } => "zellij",
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Tmux {
                available,
                server_running,
                version,
            } => match (available, server_running) {
                (true, true) => format!(
                    "Connected{}",
                    version
                        .as_ref()
                        .map(|v| format!(" ({v})"))
                        .unwrap_or_default()
                ),
                (true, false) => "Server not running".into(),
                (false, _) => "Not installed".into(),
            },
            Self::Vscode {
                webhook_running,
                port,
            } => {
                if *webhook_running {
                    format!("Webhook :{}", port.unwrap_or(7009))
                } else {
                    "Webhook stopped".into()
                }
            }
            Self::Cmux {
                binary_available,
                in_cmux,
            } => match (binary_available, in_cmux) {
                (true, true) => "Connected".into(),
                (true, false) => "Not in cmux session".into(),
                (false, _) => "Binary not found".into(),
            },
            Self::Zellij {
                binary_available,
                in_zellij,
            } => match (binary_available, in_zellij) {
                (true, true) => "Connected".into(),
                (true, false) => "Not in zellij session".into(),
                (false, _) => "Binary not found".into(),
            },
        }
    }
}

/// A point-in-time snapshot of everything the status panel needs to render.
#[derive(Debug)]
#[allow(dead_code)]
pub struct StatusSnapshot {
    pub working_dir: String,
    pub config_file_found: bool,
    pub config_path: String,
    pub tickets_dir: String,
    pub tickets_dir_exists: bool,
    pub wrapper_type: String,
    pub operator_version: String,
    pub api_status: RestApiStatus,
    pub backstage_status: ServerStatus,
    pub backstage_display: bool,
    pub kanban_providers: Vec<KanbanProviderInfo>,
    pub llm_tools: Vec<LlmToolInfo>,
    pub default_llm_tool: Option<String>,
    pub default_llm_model: Option<String>,
    pub delegators: Vec<DelegatorInfo>,
    pub git_provider: Option<String>,
    pub git_token_set: bool,
    pub git_branch_format: Option<String>,
    pub git_use_worktrees: bool,
    pub update_available_version: Option<String>,
    pub wrapper_connection_status: WrapperConnectionStatus,
    /// Resolved `$EDITOR` value
    pub env_editor: String,
    /// Resolved `$VISUAL` value
    pub env_visual: String,
}

// ---------------------------------------------------------------------------
// Section trait
// ---------------------------------------------------------------------------

/// Trait for each status panel section (mirrors the `StatusSection` interface from the `VSCode` extension).
pub trait StatusSection {
    /// Unique identifier for this section.
    fn section_id(&self) -> SectionId;

    /// Display label for the section header.
    fn label(&self) -> &'static str;

    /// Which section IDs must be Green before this section is visible.
    fn prerequisites(&self) -> &[SectionId];

    /// Current health state — determines header color.
    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth;

    /// Summary description shown next to the section header.
    fn description(&self, snapshot: &StatusSnapshot) -> String;

    /// Child rows when this section is expanded.
    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow>;

    /// Build the `SectionDefinition` metadata for this section.
    #[allow(dead_code)]
    fn definition(&self) -> SectionDefinition {
        SectionDefinition {
            id: self.section_id(),
            label: self.label().to_string(),
            prerequisites: self.prerequisites().to_vec(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tree state
// ---------------------------------------------------------------------------

/// Tracks which sections are expanded/collapsed and the cursor position.
#[derive(Debug, Clone)]
pub struct TreeState {
    pub expanded: HashMap<SectionId, bool>,
    pub selected: usize,
    pub scroll_offset: usize,
    /// Rows currently running a refresh action (`section_id`, row label).
    /// Used to render ⟳ in yellow while refreshing.
    pub refreshing: HashSet<(SectionId, String)>,
}

impl TreeState {
    pub fn new() -> Self {
        let mut expanded = HashMap::new();
        expanded.insert(SectionId::Configuration, true);
        expanded.insert(SectionId::Connections, false);
        expanded.insert(SectionId::Kanban, false);
        expanded.insert(SectionId::LlmTools, false);
        expanded.insert(SectionId::Delegators, false);
        expanded.insert(SectionId::Git, false);
        Self {
            expanded,
            selected: 0,
            scroll_offset: 0,
            refreshing: HashSet::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Status panel (orchestrator)
// ---------------------------------------------------------------------------

/// The status panel widget — a collapsible tree with progressive disclosure.
pub struct StatusPanel {
    pub tree_state: TreeState,
    pub title: String,
    sections: Vec<Box<dyn StatusSection>>,
}

impl StatusPanel {
    pub fn new(title: String) -> Self {
        let sections: Vec<Box<dyn StatusSection>> = vec![
            Box::new(ConfigSection),
            Box::new(ConnectionsSection),
            Box::new(KanbanSection),
            Box::new(LlmSection),
            Box::new(DelegatorSection),
            Box::new(GitSection),
        ];
        Self {
            tree_state: TreeState::new(),
            title,
            sections,
        }
    }

    fn is_expanded(&self, id: SectionId) -> bool {
        self.tree_state.expanded.get(&id).copied().unwrap_or(false)
    }

    /// Check if all prerequisite sections are Green (transitively).
    /// A section is visible only if its prerequisites are Green AND those
    /// prerequisites' own prerequisites are also met.
    fn prerequisites_met(&self, section: &dyn StatusSection, snapshot: &StatusSnapshot) -> bool {
        section.prerequisites().iter().all(|prereq_id| {
            self.sections
                .iter()
                .find(|s| s.section_id() == *prereq_id)
                .is_some_and(|s| {
                    // Prerequisite must itself be visible (transitive check)
                    self.prerequisites_met_by_id(s.section_id(), snapshot)
                        && s.health(snapshot) == SectionHealth::Green
                })
        })
    }

    fn prerequisites_met_by_id(&self, id: SectionId, snapshot: &StatusSnapshot) -> bool {
        self.sections
            .iter()
            .find(|s| s.section_id() == id)
            .is_some_and(|s| self.prerequisites_met(s.as_ref(), snapshot))
    }

    /// Build the list of visible rows, respecting expand/collapse and
    /// prerequisite-based progressive disclosure.
    pub fn flatten(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        let mut rows: Vec<TreeRow> = Vec::new();

        for section in &self.sections {
            if !self.prerequisites_met(section.as_ref(), snapshot) {
                continue;
            }

            let health = section.health(snapshot);

            // Header row
            rows.push(TreeRow {
                section_id: section.section_id(),
                depth: 0,
                label: section.label().to_string(),
                description: section.description(snapshot),
                icon: StatusIcon::None,
                is_header: true,
                actions: ActionSet::primary(StatusAction::ToggleSection(section.section_id())),
                health,
            });

            // Children (if expanded)
            if self.is_expanded(section.section_id()) {
                let sid = section.section_id();
                let mut children = section.children(snapshot);
                // Auto-populate back action on child rows: collapse parent section
                for child in &mut children {
                    if child.actions.back == StatusAction::None {
                        child.actions.back = StatusAction::ToggleSection(sid);
                    }
                }
                rows.extend(children);
            }
        }

        rows
    }

    /// Returns true if any visible section has Yellow or Red health.
    pub fn has_attention_needed(&self, snapshot: &StatusSnapshot) -> bool {
        self.sections.iter().any(|s| {
            self.prerequisites_met(s.as_ref(), snapshot)
                && matches!(
                    s.health(snapshot),
                    SectionHealth::Yellow | SectionHealth::Red
                )
        })
    }

    /// Select the first header row that has Yellow or Red health.
    /// Expands that section so its children are visible for interaction.
    pub fn focus_first_attention(&mut self, snapshot: &StatusSnapshot) {
        let rows = self.flatten(snapshot);
        for (i, row) in rows.iter().enumerate() {
            if row.is_header && matches!(row.health, SectionHealth::Yellow | SectionHealth::Red) {
                self.tree_state.selected = i;
                // Expand the section so the user sees what needs attention
                self.tree_state.expanded.insert(row.section_id, true);
                return;
            }
        }
    }

    /// Get the action for the currently selected row and button.
    /// If the action is `ToggleSection`, perform the toggle internally.
    pub fn action_for_current(
        &mut self,
        snapshot: &StatusSnapshot,
        button: ActionButton,
    ) -> StatusAction {
        let rows = self.flatten(snapshot);
        let action = rows
            .get(self.tree_state.selected)
            .map(|r| r.actions.for_button(button).clone())
            .unwrap_or(StatusAction::None);

        // Only toggle sections on primary (A) button press
        if button == ActionButton::A {
            if let StatusAction::ToggleSection(section_id) = &action {
                let entry = self.tree_state.expanded.entry(*section_id).or_insert(false);
                *entry = !*entry;
            }
        }

        // B button on headers: toggle collapse
        if button == ActionButton::B {
            if let Some(row) = rows.get(self.tree_state.selected) {
                if row.is_header && self.is_expanded(row.section_id) {
                    self.tree_state.expanded.insert(row.section_id, false);
                    return StatusAction::None;
                }
            }
        }

        action
    }

    /// Move selection down, wrapping to the top.
    pub fn select_next(&mut self, snapshot: &StatusSnapshot) {
        let count = self.flatten(snapshot).len();
        if count == 0 {
            return;
        }
        self.tree_state.selected = (self.tree_state.selected + 1) % count;
        self.adjust_scroll(snapshot);
    }

    /// Move selection up, wrapping to the bottom.
    pub fn select_prev(&mut self, snapshot: &StatusSnapshot) {
        let count = self.flatten(snapshot).len();
        if count == 0 {
            return;
        }
        if self.tree_state.selected == 0 {
            self.tree_state.selected = count - 1;
        } else {
            self.tree_state.selected -= 1;
        }
        self.adjust_scroll(snapshot);
    }

    /// Number of currently visible (flattened) rows.
    #[allow(dead_code)]
    pub fn visible_count(&self, snapshot: &StatusSnapshot) -> usize {
        self.flatten(snapshot).len()
    }

    /// Render the status panel into the given area.
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, snapshot: &StatusSnapshot) {
        let rows = self.flatten(snapshot);
        let inner_height = area.height.saturating_sub(2) as usize;
        let offset = self.tree_state.scroll_offset;

        let visible_rows = rows.iter().skip(offset).take(inner_height);

        let lines: Vec<Line> = visible_rows
            .enumerate()
            .map(|(i, row)| {
                let abs_idx = offset + i;
                let is_selected = abs_idx == self.tree_state.selected;

                let mut spans: Vec<Span> = Vec::new();

                if row.is_header {
                    let chevron = if self.is_expanded(row.section_id) {
                        "▾ "
                    } else {
                        "▸ "
                    };
                    spans.push(Span::raw(chevron));
                    // Header label colored by health state
                    spans.push(Span::styled(
                        row.label.clone(),
                        Style::default()
                            .fg(row.health.to_color())
                            .add_modifier(Modifier::BOLD),
                    ));
                    if !row.description.is_empty() {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(
                            row.description.clone(),
                            Style::default().fg(Color::Gray),
                        ));
                    }
                } else {
                    spans.push(Span::raw("  "));
                    spans.push(row.icon.as_span());
                    spans.push(Span::raw(row.label.clone()));
                    if !row.description.is_empty() {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(
                            row.description.clone(),
                            Style::default().fg(Color::Gray),
                        ));
                    }
                }

                // Right-aligned action indicators
                let has_special = row.actions.special != StatusAction::None;
                let has_refresh = row.actions.refresh != StatusAction::None;

                if (is_selected && has_special) || has_refresh {
                    // Calculate content width so far
                    let content_width: usize = spans.iter().map(|s| s.content.len()).sum();
                    // Inner width = area minus border chars (2)
                    let inner_width = area.width.saturating_sub(2) as usize;

                    // Build the right-side indicator string
                    let mut indicator = String::new();
                    if is_selected && has_special {
                        let title = row.actions.special_title();
                        indicator.push_str(title);
                    }
                    if has_refresh {
                        if !indicator.is_empty() {
                            indicator.push(' ');
                        }
                        indicator.push_str(row.actions.refresh_title());
                    }

                    // Pad to right-align
                    let indicator_width = indicator.chars().count();
                    let gap = inner_width.saturating_sub(content_width + indicator_width);
                    if gap > 0 {
                        spans.push(Span::raw(" ".repeat(gap)));
                    }

                    // Render indicator spans with appropriate colors
                    if is_selected && has_special {
                        let title = row.actions.special_title();
                        spans.push(Span::styled(
                            title.to_string(),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                    if has_refresh {
                        if is_selected && has_special {
                            spans.push(Span::raw(" "));
                        }
                        let is_refreshing = self
                            .tree_state
                            .refreshing
                            .contains(&(row.section_id, row.label.clone()));
                        let color = if is_refreshing {
                            Color::Rgb(255, 193, 7) // Yellow while refreshing
                        } else {
                            Color::White
                        };
                        spans.push(Span::styled(
                            row.actions.refresh_title().to_string(),
                            Style::default().fg(color),
                        ));
                    }
                }

                let line = Line::from(spans);
                if is_selected {
                    line.style(Style::default().add_modifier(Modifier::REVERSED))
                } else {
                    line
                }
            })
            .collect();

        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(self.title.clone());

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    fn adjust_scroll(&mut self, snapshot: &StatusSnapshot) {
        let rows = self.flatten(snapshot);
        let count = rows.len();
        if count == 0 {
            self.tree_state.scroll_offset = 0;
            return;
        }
        if self.tree_state.selected < self.tree_state.scroll_offset {
            self.tree_state.scroll_offset = self.tree_state.selected;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_snapshot() -> StatusSnapshot {
        StatusSnapshot {
            working_dir: "/home/user/project".into(),
            config_file_found: true,
            config_path: "operator.toml".into(),
            tickets_dir: ".tickets".into(),
            tickets_dir_exists: true,
            wrapper_type: "tmux".into(),
            operator_version: "0.1.28".into(),
            api_status: RestApiStatus::Running { port: 3100 },
            backstage_status: ServerStatus::Running {
                port: 7007,
                pid: 1234,
            },
            kanban_providers: vec![KanbanProviderInfo {
                provider_type: "Linear".into(),
                domain: "myteam.linear.app".into(),
            }],
            llm_tools: vec![LlmToolInfo {
                name: "Claude".into(),
                version: "3.5".into(),
                model_aliases: vec!["opus".into(), "sonnet".into(), "haiku".into()],
            }],
            default_llm_tool: None,
            default_llm_model: None,
            delegators: vec![DelegatorInfo {
                name: "claude-opus".into(),
                display_name: Some("Claude Opus".into()),
                llm_tool: "claude".into(),
                model: "opus".into(),
                yolo: false,
            }],
            git_provider: Some("GitHub".into()),
            git_token_set: true,
            git_branch_format: Some("feature/{ticket}".into()),
            git_use_worktrees: false,
            update_available_version: None,
            wrapper_connection_status: WrapperConnectionStatus::Tmux {
                available: true,
                server_running: true,
                version: Some("3.4".into()),
            },
            env_editor: "vim".into(),
            env_visual: String::new(),
            backstage_display: false,
        }
    }

    #[test]
    fn test_flatten_tier0_always_visible() {
        let panel = StatusPanel::new("Status".into());
        // With a healthy snapshot, Configuration is always visible
        let snap = test_snapshot();
        let rows = panel.flatten(&snap);

        assert!(rows[0].is_header);
        assert_eq!(rows[0].label, "Configuration");
    }

    #[test]
    fn test_flatten_progressive_disclosure() {
        let panel = StatusPanel::new("Status".into());

        // With all green, all sections visible
        let snap = test_snapshot();
        let rows = panel.flatten(&snap);
        assert!(
            rows.iter().any(|r| r.section_id == SectionId::Connections),
            "Connections should appear when Configuration is green"
        );
        assert!(
            rows.iter().any(|r| r.section_id == SectionId::Kanban),
            "Kanban should appear when Connections is green"
        );

        // With config missing, only Configuration shows (red)
        let mut bad_snap = test_snapshot();
        bad_snap.config_file_found = false;
        let rows = panel.flatten(&bad_snap);
        assert!(
            !rows.iter().any(|r| r.section_id == SectionId::Connections),
            "Connections should NOT appear when Configuration is red"
        );
    }

    #[test]
    fn test_flatten_expanded_shows_children() {
        let panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();

        // Configuration is expanded by default
        let rows = panel.flatten(&snap);
        let config_children: Vec<_> = rows
            .iter()
            .filter(|r| r.section_id == SectionId::Configuration && !r.is_header)
            .collect();
        assert_eq!(config_children.len(), 8, "Should have 8 config children");
        assert_eq!(config_children[0].label, "Working Dir");
        assert_eq!(config_children[1].label, "Config");
        assert_eq!(config_children[2].label, "Tickets");
        assert_eq!(config_children[3].label, "tmux"); // wrapper connection
        assert_eq!(config_children[4].label, "Wrapper");
        assert_eq!(config_children[5].label, "$EDITOR");
        assert_eq!(config_children[6].label, "$VISUAL");
        assert_eq!(config_children[7].label, "Version");
    }

    #[test]
    fn test_action_for_current_toggles_header() {
        let mut panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();

        panel.tree_state.selected = 0;
        assert!(panel.is_expanded(SectionId::Configuration));

        let action = panel.action_for_current(&snap, ActionButton::A);
        assert_eq!(
            action,
            StatusAction::ToggleSection(SectionId::Configuration)
        );
        assert!(!panel.is_expanded(SectionId::Configuration));

        let action = panel.action_for_current(&snap, ActionButton::A);
        assert_eq!(
            action,
            StatusAction::ToggleSection(SectionId::Configuration)
        );
        assert!(panel.is_expanded(SectionId::Configuration));
    }

    #[test]
    fn test_action_for_current_child_rows() {
        let mut panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();

        // Working Dir (index 1) — should open directory
        panel.tree_state.selected = 1;
        let action = panel.action_for_current(&snap, ActionButton::A);
        assert!(matches!(action, StatusAction::OpenDirectory(_)));

        // Config (index 2) — should edit file
        panel.tree_state.selected = 2;
        let action = panel.action_for_current(&snap, ActionButton::A);
        assert!(matches!(action, StatusAction::EditFile(_)));

        // Tickets (index 3) — should open directory
        panel.tree_state.selected = 3;
        let action = panel.action_for_current(&snap, ActionButton::A);
        assert!(matches!(action, StatusAction::OpenDirectory(_)));

        // Wrapper (index 4) — read-only
        panel.tree_state.selected = 4;
        let action = panel.action_for_current(&snap, ActionButton::A);
        assert_eq!(action, StatusAction::None);

        // $EDITOR (index 5) — read-only
        panel.tree_state.selected = 5;
        let action = panel.action_for_current(&snap, ActionButton::A);
        assert_eq!(action, StatusAction::None);

        // $VISUAL (index 6) — read-only
        panel.tree_state.selected = 6;
        let action = panel.action_for_current(&snap, ActionButton::A);
        assert_eq!(action, StatusAction::None);

        // $IDE (index 7) — read-only
        panel.tree_state.selected = 7;
        let action = panel.action_for_current(&snap, ActionButton::A);
        assert_eq!(action, StatusAction::None);

        // Version (index 8) — opens downloads URL
        panel.tree_state.selected = 8;
        let action = panel.action_for_current(&snap, ActionButton::A);
        assert!(matches!(action, StatusAction::OpenUrl(_)));
    }

    #[test]
    fn test_section_health_colors() {
        let snap = test_snapshot();
        let panel = StatusPanel::new("Status".into());
        let rows = panel.flatten(&snap);

        // Configuration should be green (all good)
        let config_header = rows
            .iter()
            .find(|r| r.section_id == SectionId::Configuration && r.is_header)
            .unwrap();
        assert_eq!(config_header.health, SectionHealth::Green);

        // Test red state
        let mut bad_snap = test_snapshot();
        bad_snap.config_file_found = false;
        let rows = panel.flatten(&bad_snap);
        let config_header = rows
            .iter()
            .find(|r| r.section_id == SectionId::Configuration && r.is_header)
            .unwrap();
        assert_eq!(config_header.health, SectionHealth::Red);

        // Test yellow state
        let mut warn_snap = test_snapshot();
        warn_snap.tickets_dir_exists = false;
        let rows = panel.flatten(&warn_snap);
        let config_header = rows
            .iter()
            .find(|r| r.section_id == SectionId::Configuration && r.is_header)
            .unwrap();
        assert_eq!(config_header.health, SectionHealth::Yellow);
    }

    #[test]
    fn test_working_dir_shows_check_when_configured() {
        let panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();
        let rows = panel.flatten(&snap);

        let working_dir = rows
            .iter()
            .find(|r| r.label == "Working Dir" && !r.is_header)
            .unwrap();
        assert!(
            matches!(working_dir.icon, StatusIcon::Check),
            "Working Dir should show Check icon when configured"
        );
    }

    #[test]
    fn test_select_next_wraps() {
        let mut panel = StatusPanel::new("Status".into());
        // Collapse config so only the header is visible
        panel
            .tree_state
            .expanded
            .insert(SectionId::Configuration, false);

        // Use a snapshot where only Configuration is green but Connections prerequisites fail
        let mut snap = test_snapshot();
        snap.config_file_found = false; // Makes Configuration red, hiding Connections
        let count = panel.visible_count(&snap);
        assert_eq!(count, 1, "Only 1 row visible");

        panel.tree_state.selected = 0;
        panel.select_next(&snap);
        assert_eq!(panel.tree_state.selected, 0, "Should wrap");
    }

    #[test]
    fn test_visible_count() {
        let panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();
        let count = panel.visible_count(&snap);
        let rows = panel.flatten(&snap);
        assert_eq!(count, rows.len());
    }

    #[test]
    fn test_wrapper_connection_tmux_connected() {
        let status = WrapperConnectionStatus::Tmux {
            available: true,
            server_running: true,
            version: Some("tmux 3.4".into()),
        };
        assert!(status.is_connected());
        assert_eq!(status.label(), "tmux");
        assert_eq!(status.description(), "Connected (tmux 3.4)");
    }

    #[test]
    fn test_wrapper_connection_tmux_server_not_running() {
        let status = WrapperConnectionStatus::Tmux {
            available: true,
            server_running: false,
            version: Some("tmux 3.4".into()),
        };
        assert!(!status.is_connected());
        assert_eq!(status.description(), "Server not running");
    }

    #[test]
    fn test_wrapper_connection_tmux_not_installed() {
        let status = WrapperConnectionStatus::Tmux {
            available: false,
            server_running: false,
            version: None,
        };
        assert!(!status.is_connected());
        assert_eq!(status.description(), "Not installed");
    }

    #[test]
    fn test_wrapper_connection_vscode() {
        let status = WrapperConnectionStatus::Vscode {
            webhook_running: true,
            port: Some(7009),
        };
        assert!(status.is_connected());
        assert_eq!(status.label(), "vscode");
        assert_eq!(status.description(), "Webhook :7009");

        let stopped = WrapperConnectionStatus::Vscode {
            webhook_running: false,
            port: None,
        };
        assert!(!stopped.is_connected());
        assert_eq!(stopped.description(), "Webhook stopped");
    }

    #[test]
    fn test_wrapper_connection_cmux() {
        let status = WrapperConnectionStatus::Cmux {
            binary_available: true,
            in_cmux: true,
        };
        assert!(status.is_connected());
        assert_eq!(status.label(), "cmux");

        let not_in = WrapperConnectionStatus::Cmux {
            binary_available: true,
            in_cmux: false,
        };
        assert!(!not_in.is_connected());
        assert_eq!(not_in.description(), "Not in cmux session");
    }

    #[test]
    fn test_wrapper_connection_zellij() {
        let status = WrapperConnectionStatus::Zellij {
            binary_available: true,
            in_zellij: true,
        };
        assert!(status.is_connected());
        assert_eq!(status.label(), "zellij");

        let no_binary = WrapperConnectionStatus::Zellij {
            binary_available: false,
            in_zellij: false,
        };
        assert!(!no_binary.is_connected());
        assert_eq!(no_binary.description(), "Binary not found");
    }

    #[test]
    fn test_action_set_primary_constructor() {
        let set = ActionSet::primary(StatusAction::StartApi);
        assert_eq!(set.primary, StatusAction::StartApi);
        assert_eq!(set.back, StatusAction::None);
        assert_eq!(set.special, StatusAction::None);
        assert_eq!(set.refresh, StatusAction::None);
    }

    #[test]
    fn test_action_set_none_constructor() {
        let set = ActionSet::none();
        assert_eq!(set.primary, StatusAction::None);
        assert_eq!(set.back, StatusAction::None);
        assert_eq!(set.special, StatusAction::None);
        assert_eq!(set.refresh, StatusAction::None);
    }

    #[test]
    fn test_action_set_for_button() {
        let set = ActionSet {
            primary: StatusAction::StartApi,
            back: StatusAction::ToggleSection(SectionId::Configuration),
            special: StatusAction::EditFile("config.toml".into()),
            special_meta: Some(ActionMeta {
                title: "Config",
                tooltip: "Edit config",
            }),
            refresh: StatusAction::RefreshSection(SectionId::Connections),
            refresh_meta: Some(ActionMeta {
                title: "Sync",
                tooltip: "Refresh connections",
            }),
        };
        assert_eq!(*set.for_button(ActionButton::A), StatusAction::StartApi);
        assert_eq!(
            *set.for_button(ActionButton::B),
            StatusAction::ToggleSection(SectionId::Configuration)
        );
        assert_eq!(
            *set.for_button(ActionButton::X),
            StatusAction::EditFile("config.toml".into())
        );
        assert_eq!(
            *set.for_button(ActionButton::Y),
            StatusAction::RefreshSection(SectionId::Connections)
        );
    }

    #[test]
    fn test_flatten_auto_populates_back_on_children() {
        let panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();
        let rows = panel.flatten(&snap);

        // Config children should have back = ToggleSection(Configuration)
        let config_child = rows
            .iter()
            .find(|r| r.label == "Working Dir" && !r.is_header)
            .unwrap();
        assert_eq!(
            config_child.actions.back,
            StatusAction::ToggleSection(SectionId::Configuration)
        );
    }

    #[test]
    fn test_action_for_current_b_collapses_header() {
        let mut panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();

        // Configuration is expanded
        panel.tree_state.selected = 0;
        assert!(panel.is_expanded(SectionId::Configuration));

        // B on expanded header should collapse it
        let action = panel.action_for_current(&snap, ActionButton::B);
        assert_eq!(action, StatusAction::None);
        assert!(!panel.is_expanded(SectionId::Configuration));
    }

    #[test]
    fn test_action_for_current_x_returns_special() {
        let mut panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();

        // Config row (index 2) should have a special action (ResetConfig)
        panel.tree_state.selected = 2;
        let action = panel.action_for_current(&snap, ActionButton::X);
        assert_eq!(
            action,
            StatusAction::ResetConfig,
            "Config special action should be ResetConfig, got {action:?}"
        );
    }

    #[test]
    fn test_action_for_current_y_returns_refresh() {
        let mut panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();

        // Version row (index 8) should have a refresh action
        panel.tree_state.selected = 8;
        let action = panel.action_for_current(&snap, ActionButton::Y);
        assert!(
            matches!(action, StatusAction::RefreshSection(_)),
            "Version refresh action should be RefreshSection, got {action:?}"
        );
    }

    #[test]
    fn test_special_indicator_only_on_rows_with_special_action() {
        let panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();
        let rows = panel.flatten(&snap);

        // Wrapper row should NOT have special action
        let wrapper = rows.iter().find(|r| r.label == "Wrapper").unwrap();
        assert_eq!(wrapper.actions.special, StatusAction::None);

        // Config row SHOULD have special action (ResetConfig)
        let config = rows.iter().find(|r| r.label == "Config").unwrap();
        assert_ne!(config.actions.special, StatusAction::None);
    }

    #[test]
    fn test_refresh_indicator_only_on_rows_with_refresh_action() {
        let panel = StatusPanel::new("Status".into());
        let snap = test_snapshot();
        let rows = panel.flatten(&snap);

        // Version row should have refresh action
        let version = rows.iter().find(|r| r.label == "Version").unwrap();
        assert_ne!(version.actions.refresh, StatusAction::None);

        // Config row SHOULD have refresh action (ReloadConfig)
        let config = rows.iter().find(|r| r.label == "Config").unwrap();
        assert_ne!(config.actions.refresh, StatusAction::None);
    }

    #[test]
    fn test_refreshing_set_tracks_state() {
        let mut state = TreeState::new();
        let key = (SectionId::Configuration, "Version".to_string());
        assert!(!state.refreshing.contains(&key));
        state.refreshing.insert(key.clone());
        assert!(state.refreshing.contains(&key));
        state.refreshing.remove(&key);
        assert!(!state.refreshing.contains(&key));
    }
}
