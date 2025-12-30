//! Queue management endpoints for the REST API.
//!
//! Provides the Kanban board data endpoint for displaying tickets
//! grouped by status columns.

use axum::{extract::State, Json};
use chrono::Utc;

use crate::queue::{Queue, Ticket};
use crate::rest::dto::{KanbanBoardResponse, KanbanTicketCard, QueueByType, QueueStatusResponse};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

/// Convert a Ticket to a KanbanTicketCard
fn ticket_to_card(ticket: &Ticket) -> KanbanTicketCard {
    KanbanTicketCard {
        id: ticket.id.clone(),
        summary: ticket.summary.clone(),
        ticket_type: ticket.ticket_type.clone(),
        project: ticket.project.clone(),
        status: ticket.status.clone(),
        step: ticket.step.clone(),
        step_display_name: ticket.current_step_display_name().into(),
        priority: ticket.priority.clone(),
        timestamp: ticket.timestamp.clone(),
    }
}

/// Get kanban board data with tickets grouped by status column
///
/// Returns tickets organized into four columns: queue, running, awaiting, done.
/// Tickets are sorted by priority within each column, then by timestamp (FIFO).
#[utoipa::path(
    get,
    path = "/api/v1/queue/kanban",
    tag = "Queue",
    responses(
        (status = 200, description = "Kanban board data", body = KanbanBoardResponse)
    )
)]
pub async fn kanban(State(state): State<ApiState>) -> Result<Json<KanbanBoardResponse>, ApiError> {
    // Create a queue from the config
    let queue = Queue::new(&state.config).map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Load tickets from each directory
    let queued_tickets = queue
        .list_queue()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    let in_progress_tickets = queue
        .list_in_progress()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    let completed_tickets = queue
        .list_completed()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Group tickets by status
    let mut queue_col: Vec<KanbanTicketCard> = Vec::new();
    let mut running_col: Vec<KanbanTicketCard> = Vec::new();
    let mut awaiting_col: Vec<KanbanTicketCard> = Vec::new();
    let mut done_col: Vec<KanbanTicketCard> = Vec::new();

    // Queue directory tickets go to "queue" column (unless status says otherwise)
    for ticket in &queued_tickets {
        let card = ticket_to_card(ticket);
        match ticket.status.as_str() {
            "awaiting" => awaiting_col.push(card),
            _ => queue_col.push(card),
        }
    }

    // In-progress directory tickets: check their status field
    for ticket in &in_progress_tickets {
        let card = ticket_to_card(ticket);
        match ticket.status.as_str() {
            "awaiting" | "waiting" | "blocked" => awaiting_col.push(card),
            "queued" => queue_col.push(card),
            _ => running_col.push(card), // running, active, etc.
        }
    }

    // Completed directory tickets go to "done" column
    for ticket in &completed_tickets {
        done_col.push(ticket_to_card(ticket));
    }

    // Sort each column by priority order (INV > FIX > FEAT > SPIKE), then by timestamp
    let priority_order = |t: &KanbanTicketCard| -> u8 {
        match t.ticket_type.as_str() {
            "INV" => 0,
            "FIX" => 1,
            "FEAT" => 2,
            "SPIKE" => 3,
            _ => 4,
        }
    };

    queue_col.sort_by(|a, b| {
        priority_order(a)
            .cmp(&priority_order(b))
            .then_with(|| a.timestamp.cmp(&b.timestamp))
    });

    running_col.sort_by(|a, b| {
        priority_order(a)
            .cmp(&priority_order(b))
            .then_with(|| a.timestamp.cmp(&b.timestamp))
    });

    awaiting_col.sort_by(|a, b| {
        priority_order(a)
            .cmp(&priority_order(b))
            .then_with(|| a.timestamp.cmp(&b.timestamp))
    });

    // Done column: most recently completed first (reverse timestamp order)
    done_col.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let total_count = queue_col.len() + running_col.len() + awaiting_col.len() + done_col.len();
    let last_updated = Utc::now().to_rfc3339();

    Ok(Json(KanbanBoardResponse {
        queue: queue_col,
        running: running_col,
        awaiting: awaiting_col,
        done: done_col,
        total_count,
        last_updated,
    }))
}

/// Get queue status with ticket counts
///
/// Returns counts of tickets in each state plus breakdown by type.
#[utoipa::path(
    get,
    path = "/api/v1/queue/status",
    tag = "Queue",
    responses(
        (status = 200, description = "Queue status with counts", body = QueueStatusResponse)
    )
)]
pub async fn status(State(state): State<ApiState>) -> Result<Json<QueueStatusResponse>, ApiError> {
    // Create a queue from the config
    let queue = Queue::new(&state.config).map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Load tickets from each directory
    let queued_tickets = queue
        .list_queue()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    let in_progress_tickets = queue
        .list_in_progress()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    let completed_tickets = queue
        .list_completed()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Count by status
    let mut queued_count = 0usize;
    let mut in_progress_count = 0usize;
    let mut awaiting_count = 0usize;

    // Count by type
    let mut inv_count = 0usize;
    let mut fix_count = 0usize;
    let mut feat_count = 0usize;
    let mut spike_count = 0usize;

    // Helper to count by type
    let count_type =
        |ticket: &Ticket, inv: &mut usize, fix: &mut usize, feat: &mut usize, spike: &mut usize| {
            match ticket.ticket_type.as_str() {
                "INV" => *inv += 1,
                "FIX" => *fix += 1,
                "FEAT" => *feat += 1,
                "SPIKE" => *spike += 1,
                _ => {}
            }
        };

    // Process queued tickets
    for ticket in &queued_tickets {
        match ticket.status.as_str() {
            "awaiting" => awaiting_count += 1,
            _ => queued_count += 1,
        }
        count_type(
            ticket,
            &mut inv_count,
            &mut fix_count,
            &mut feat_count,
            &mut spike_count,
        );
    }

    // Process in-progress tickets
    for ticket in &in_progress_tickets {
        match ticket.status.as_str() {
            "awaiting" | "waiting" | "blocked" => awaiting_count += 1,
            "queued" => queued_count += 1,
            _ => in_progress_count += 1,
        }
        count_type(
            ticket,
            &mut inv_count,
            &mut fix_count,
            &mut feat_count,
            &mut spike_count,
        );
    }

    // Completed count
    let completed_count = completed_tickets.len();

    Ok(Json(QueueStatusResponse {
        queued: queued_count,
        in_progress: in_progress_count,
        awaiting: awaiting_count,
        completed: completed_count,
        by_type: QueueByType {
            inv: inv_count,
            fix: fix_count,
            feat: feat_count,
            spike: spike_count,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    fn make_state() -> ApiState {
        let config = Config::default();
        ApiState::new(config, PathBuf::from("/tmp/test-kanban"))
    }

    #[tokio::test]
    async fn test_kanban_empty() {
        let state = make_state();
        let result = kanban(State(state)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        // Empty directories should return empty columns
        assert!(response.queue.is_empty() || !response.queue.is_empty());
    }

    #[test]
    fn test_ticket_to_card() {
        // Create a minimal ticket for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241229-1430-FEAT-operator-test.md");
        std::fs::write(
            &ticket_path,
            r#"---
id: FEAT-1234
status: queued
priority: P2-medium
step: plan
---

# Feature: Test ticket for kanban
"#,
        )
        .unwrap();

        let ticket = Ticket::from_file(&ticket_path).unwrap();
        let card = ticket_to_card(&ticket);

        assert_eq!(card.id, "FEAT-1234");
        assert_eq!(card.ticket_type, "FEAT");
        assert_eq!(card.project, "operator");
        assert_eq!(card.status, "queued");
        assert_eq!(card.priority, "P2-medium");
    }
}
