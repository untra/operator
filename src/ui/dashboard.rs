#![allow(dead_code)]

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use super::panels::{AgentsPanel, AwaitingPanel, CompletedPanel, HeaderBar, QueuePanel, StatusBar};
use crate::api::RateLimitInfo;
use crate::backstage::ServerStatus;
use crate::config::Config;
use crate::queue::Ticket;
use crate::state::{AgentState, CompletedTicket, OrphanSession};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Queue,
    Agents,
    Awaiting,
    Completed,
}

pub struct Dashboard {
    pub queue_panel: QueuePanel,
    pub agents_panel: AgentsPanel,
    pub awaiting_panel: AwaitingPanel,
    pub completed_panel: CompletedPanel,
    pub focused: FocusedPanel,
    pub paused: bool,
    pub max_agents: usize,
    /// Current rate limit info from AI provider
    pub rate_limit: Option<RateLimitInfo>,
    /// Backstage server status
    pub backstage_status: ServerStatus,
}

impl Dashboard {
    pub fn new(config: &Config) -> Self {
        Self {
            queue_panel: QueuePanel::new(config.ui.panel_names.queue.clone()),
            agents_panel: AgentsPanel::new(config.ui.panel_names.agents.clone()),
            awaiting_panel: AwaitingPanel::new(config.ui.panel_names.awaiting.clone()),
            completed_panel: CompletedPanel::new(config.ui.panel_names.completed.clone()),
            focused: FocusedPanel::Queue,
            paused: false,
            max_agents: config.effective_max_agents(),
            rate_limit: None,
            backstage_status: ServerStatus::Stopped,
        }
    }

    pub fn update_rate_limit(&mut self, rate_limit: Option<RateLimitInfo>) {
        self.rate_limit = rate_limit;
    }

    pub fn update_backstage_status(&mut self, status: ServerStatus) {
        self.backstage_status = status;
    }

    pub fn update_queue(&mut self, tickets: Vec<Ticket>) {
        self.queue_panel.tickets = tickets;
    }

    pub fn update_agents(&mut self, agents: Vec<AgentState>) {
        // Split into running and awaiting
        let (awaiting, running): (Vec<_>, Vec<_>) = agents
            .into_iter()
            .partition(|a| a.status == "awaiting_input");

        self.agents_panel.agents = running;
        self.awaiting_panel.agents = awaiting;
    }

    pub fn update_completed(&mut self, tickets: Vec<CompletedTicket>) {
        self.completed_panel.tickets = tickets;
    }

    pub fn update_orphan_sessions(&mut self, orphans: Vec<OrphanSession>) {
        self.agents_panel.orphan_sessions = orphans;
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(10),   // Main content
                Constraint::Length(2), // Status bar
            ])
            .split(frame.area());

        // Header with rate limit meter
        let header = HeaderBar {
            version: "0.1.0",
            rate_limit: self.rate_limit.as_ref(),
        };
        header.render(frame, chunks[0]);

        // Main content - 4 columns
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Queue
                Constraint::Percentage(30), // Running
                Constraint::Percentage(25), // Awaiting
                Constraint::Percentage(20), // Completed
            ])
            .split(chunks[1]);

        // Render panels
        self.queue_panel
            .render(frame, main_chunks[0], self.focused == FocusedPanel::Queue);

        self.agents_panel.render(
            frame,
            main_chunks[1],
            self.focused == FocusedPanel::Agents,
            self.max_agents,
        );

        self.awaiting_panel.render(
            frame,
            main_chunks[2],
            self.focused == FocusedPanel::Awaiting,
        );

        self.completed_panel.render(
            frame,
            main_chunks[3],
            self.focused == FocusedPanel::Completed,
        );

        // Status bar
        let status = StatusBar {
            paused: self.paused,
            agent_count: self.agents_panel.agents.len() + self.awaiting_panel.agents.len(),
            max_agents: self.max_agents,
            backstage_status: self.backstage_status.clone(),
        };
        status.render(frame, chunks[2]);
    }

    pub fn focus_next(&mut self) {
        self.focused = match self.focused {
            FocusedPanel::Queue => FocusedPanel::Agents,
            FocusedPanel::Agents => FocusedPanel::Awaiting,
            FocusedPanel::Awaiting => FocusedPanel::Completed,
            FocusedPanel::Completed => FocusedPanel::Queue,
        };
    }

    pub fn focus_prev(&mut self) {
        self.focused = match self.focused {
            FocusedPanel::Queue => FocusedPanel::Completed,
            FocusedPanel::Agents => FocusedPanel::Queue,
            FocusedPanel::Awaiting => FocusedPanel::Agents,
            FocusedPanel::Completed => FocusedPanel::Awaiting,
        };
    }

    pub fn select_next(&mut self) {
        match self.focused {
            FocusedPanel::Queue => {
                let len = self.queue_panel.tickets.len();
                if len > 0 {
                    let i = self.queue_panel.state.selected().map_or(0, |i| {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.queue_panel.state.select(Some(i));
                }
            }
            FocusedPanel::Agents => {
                // Include orphan sessions in total count
                let len = self.agents_panel.total_items();
                if len > 0 {
                    let i = self.agents_panel.state.selected().map_or(0, |i| {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.agents_panel.state.select(Some(i));
                }
            }
            FocusedPanel::Awaiting => {
                let len = self.awaiting_panel.agents.len();
                if len > 0 {
                    let i = self.awaiting_panel.state.selected().map_or(0, |i| {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.awaiting_panel.state.select(Some(i));
                }
            }
            FocusedPanel::Completed => {}
        }
    }

    pub fn select_prev(&mut self) {
        match self.focused {
            FocusedPanel::Queue => {
                let len = self.queue_panel.tickets.len();
                if len > 0 {
                    let i = self.queue_panel.state.selected().map_or(0, |i| {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    });
                    self.queue_panel.state.select(Some(i));
                }
            }
            FocusedPanel::Agents => {
                // Include orphan sessions in total count
                let len = self.agents_panel.total_items();
                if len > 0 {
                    let i = self.agents_panel.state.selected().map_or(0, |i| {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    });
                    self.agents_panel.state.select(Some(i));
                }
            }
            FocusedPanel::Awaiting => {
                let len = self.awaiting_panel.agents.len();
                if len > 0 {
                    let i = self.awaiting_panel.state.selected().map_or(0, |i| {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    });
                    self.awaiting_panel.state.select(Some(i));
                }
            }
            FocusedPanel::Completed => {}
        }
    }

    pub fn selected_ticket(&self) -> Option<&Ticket> {
        if self.focused == FocusedPanel::Queue {
            self.queue_panel
                .state
                .selected()
                .and_then(|i| self.queue_panel.tickets.get(i))
        } else {
            None
        }
    }

    pub fn selected_agent(&self) -> Option<&AgentState> {
        match self.focused {
            FocusedPanel::Agents => self
                .agents_panel
                .state
                .selected()
                .and_then(|i| self.agents_panel.agents.get(i)),
            FocusedPanel::Awaiting => self
                .awaiting_panel
                .state
                .selected()
                .and_then(|i| self.awaiting_panel.agents.get(i)),
            _ => None,
        }
    }

    /// Get the selected running agent (from agents panel)
    pub fn selected_running_agent(&self) -> Option<&AgentState> {
        self.agents_panel
            .state
            .selected()
            .and_then(|i| self.agents_panel.agents.get(i))
    }

    /// Get the selected awaiting agent (from awaiting panel)
    pub fn selected_awaiting_agent(&self) -> Option<&AgentState> {
        self.awaiting_panel
            .state
            .selected()
            .and_then(|i| self.awaiting_panel.agents.get(i))
    }

    /// Get the selected orphan session (from agents panel, below the fold)
    pub fn selected_orphan(&self) -> Option<&OrphanSession> {
        self.agents_panel.selected_orphan()
    }
}
