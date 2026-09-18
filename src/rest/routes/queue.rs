//! Queue management endpoints for the REST API.
//!
//! Provides the Kanban board data endpoint for displaying tickets
//! grouped by status columns, and queue control endpoints for
//! pause/resume/sync operations.

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;

use crate::config::Config;
use crate::queue::{Queue, Ticket, TicketStatus};
use crate::rest::dto::{
    KanbanBoardResponse, KanbanSyncResponse, KanbanTicketCard, QueueByType, QueueControlResponse,
    QueueStatusResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::state::State as OperatorState;

/// Convert a Ticket to a `KanbanTicketCard`
fn ticket_to_card(ticket: &Ticket) -> KanbanTicketCard {
    KanbanTicketCard {
        id: ticket.id.clone(),
        summary: ticket.summary.clone(),
        ticket_type: ticket.ticket_type.clone(),
        project: ticket.project.clone(),
        status: TicketStatus::from_frontmatter(&ticket.status),
        step: ticket.step.clone(),
        step_display_name: ticket.current_step_display_name().into(),
        priority: ticket.priority_level(),
        timestamp: ticket.timestamp.clone(),
        filename: ticket.filename.clone(),
    }
}

/// Order one active column the way the launcher orders the queue: issuetype
/// rank from `queue.priority_order`, then the ticket's own priority, then FIFO.
fn sort_active_column(config: &Config, column: &mut [KanbanTicketCard]) {
    column.sort_by(|a, b| {
        config
            .priority_index(&a.ticket_type)
            .cmp(&config.priority_index(&b.ticket_type))
            .then_with(|| a.priority.cmp(&b.priority))
            .then_with(|| a.timestamp.cmp(&b.timestamp))
    });
}

/// Get kanban board data with tickets grouped by status column
///
/// Returns tickets organized into four columns: queue, running, awaiting, done.
/// Active columns follow `queue.priority_order`, then the ticket's `priority:`
/// field, then timestamp (FIFO). The done column is newest first.
#[utoipa::path(
    operation_id = "queue_kanban",
    get,
    path = "/api/v1/queue/kanban",
    tag = "Queue",
    responses(
        (status = 200, description = "Kanban board data", body = KanbanBoardResponse)
    )
)]
pub async fn kanban(State(state): State<ApiState>) -> Result<Json<KanbanBoardResponse>, ApiError> {
    // Create a queue from the config
    let config = state.config();
    let queue = Queue::new(&config).map_err(|e| ApiError::InternalError(e.to_string()))?;

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
        match TicketStatus::parse(&ticket.status) {
            Some(TicketStatus::Awaiting) => awaiting_col.push(card),
            _ => queue_col.push(card),
        }
    }

    // In-progress directory tickets: check their status field. An unrecognised
    // status stays in the running column rather than jumping back to queue.
    for ticket in &in_progress_tickets {
        let card = ticket_to_card(ticket);
        match TicketStatus::parse(&ticket.status) {
            Some(TicketStatus::Awaiting) => awaiting_col.push(card),
            Some(TicketStatus::Queued) => queue_col.push(card),
            _ => running_col.push(card),
        }
    }

    // Completed directory tickets go to "done" column
    for ticket in &completed_tickets {
        done_col.push(ticket_to_card(ticket));
    }

    sort_active_column(&config, &mut queue_col);
    sort_active_column(&config, &mut running_col);
    sort_active_column(&config, &mut awaiting_col);

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
    operation_id = "queue_status",
    get,
    path = "/api/v1/queue/status",
    tag = "Queue",
    responses(
        (status = 200, description = "Queue status with counts", body = QueueStatusResponse)
    )
)]
pub async fn status(State(state): State<ApiState>) -> Result<Json<QueueStatusResponse>, ApiError> {
    // Create a queue from the config
    let config = state.config();
    let queue = Queue::new(&config).map_err(|e| ApiError::InternalError(e.to_string()))?;

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

    // Configured types are always present so the shape is stable; anything a
    // collection defines is added as it is encountered.
    let mut by_type: std::collections::BTreeMap<String, usize> = config
        .queue
        .priority_order
        .iter()
        .map(|t| (t.clone(), 0))
        .collect();
    let mut count_type = |ticket: &Ticket| {
        *by_type.entry(ticket.ticket_type.clone()).or_insert(0) += 1;
    };

    // Process queued tickets
    for ticket in &queued_tickets {
        match TicketStatus::parse(&ticket.status) {
            Some(TicketStatus::Awaiting) => awaiting_count += 1,
            _ => queued_count += 1,
        }
        count_type(ticket);
    }

    // Process in-progress tickets
    for ticket in &in_progress_tickets {
        match TicketStatus::parse(&ticket.status) {
            Some(TicketStatus::Awaiting) => awaiting_count += 1,
            Some(TicketStatus::Queued) => queued_count += 1,
            _ => in_progress_count += 1,
        }
        count_type(ticket);
    }

    // Completed count
    let completed_count = completed_tickets.len();

    Ok(Json(QueueStatusResponse {
        queued: queued_count,
        in_progress: in_progress_count,
        awaiting: awaiting_count,
        completed: completed_count,
        by_type: QueueByType(by_type),
    }))
}

/// Pause queue processing
///
/// Sets the queue paused state to true, stopping automatic ticket launches.
#[utoipa::path(
    operation_id = "queue_pause",
    post,
    path = "/api/v1/queue/pause",
    tag = "Queue",
    responses(
        (status = 200, description = "Queue paused successfully", body = QueueControlResponse)
    )
)]
pub async fn pause(State(state): State<ApiState>) -> Result<Json<QueueControlResponse>, ApiError> {
    let mut operator_state = OperatorState::load(&state.config())
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    operator_state
        .set_paused(true)
        .map_err(|e| ApiError::InternalError(format!("Failed to pause queue: {e}")))?;

    Ok(Json(QueueControlResponse {
        paused: true,
        message: "Queue processing paused".to_string(),
    }))
}

/// Resume queue processing
///
/// Sets the queue paused state to false, resuming automatic ticket launches.
#[utoipa::path(
    operation_id = "queue_resume",
    post,
    path = "/api/v1/queue/resume",
    tag = "Queue",
    responses(
        (status = 200, description = "Queue resumed successfully", body = QueueControlResponse)
    )
)]
pub async fn resume(State(state): State<ApiState>) -> Result<Json<QueueControlResponse>, ApiError> {
    let mut operator_state = OperatorState::load(&state.config())
        .map_err(|e| ApiError::InternalError(format!("Failed to load state: {e}")))?;

    operator_state
        .set_paused(false)
        .map_err(|e| ApiError::InternalError(format!("Failed to resume queue: {e}")))?;

    Ok(Json(QueueControlResponse {
        paused: false,
        message: "Queue processing resumed".to_string(),
    }))
}

/// Sync kanban collections
///
/// Fetches issues from configured external kanban providers (Jira, Linear, etc.)
/// and creates local tickets in the queue.
#[utoipa::path(
    operation_id = "queue_sync",
    post,
    path = "/api/v1/queue/sync",
    tag = "Queue",
    responses(
        (status = 200, description = "Kanban sync completed", body = KanbanSyncResponse)
    )
)]
pub async fn sync(State(state): State<ApiState>) -> Result<Json<KanbanSyncResponse>, ApiError> {
    use crate::services::KanbanSyncService;

    let sync_service = KanbanSyncService::new(&state.config());

    let result = sync_service
        .sync_all()
        .await
        .map_err(|e| ApiError::InternalError(format!("Kanban sync failed: {e}")))?;

    Ok(Json(KanbanSyncResponse {
        created: result.created,
        skipped: result.skipped,
        errors: result.errors,
        total_processed: result.total_processed,
    }))
}

/// Sync a specific kanban collection
///
/// Fetches issues from a single provider/project combination and creates
/// local tickets in the queue.
#[utoipa::path(
    operation_id = "queue_sync_collection",
    post,
    path = "/api/v1/queue/sync/{provider}/{project_key}",
    tag = "Queue",
    params(
        ("provider" = String, Path, description = "Provider name (jira or linear)"),
        ("project_key" = String, Path, description = "Project/team key"),
    ),
    responses(
        (status = 200, description = "Collection sync completed", body = KanbanSyncResponse)
    )
)]
pub async fn sync_collection(
    State(state): State<ApiState>,
    Path((provider, project_key)): Path<(String, String)>,
) -> Result<Json<KanbanSyncResponse>, ApiError> {
    use crate::services::KanbanSyncService;

    let sync_service = KanbanSyncService::new(&state.config());

    let result = sync_service
        .sync_collection(&provider, &project_key)
        .await
        .map_err(|e| ApiError::InternalError(format!("Kanban sync failed: {e}")))?;

    Ok(Json(KanbanSyncResponse {
        created: result.created,
        skipped: result.skipped,
        errors: result.errors,
        total_processed: result.total_processed,
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

    /// Points the handler's `Queue` at `dir`; `make_state` alone does not,
    /// because the config keeps its default relative tickets path.
    fn make_state_in(dir: &std::path::Path) -> ApiState {
        let mut config = Config::default();
        config.paths.tickets = dir.to_string_lossy().into_owned();
        ApiState::new(config, dir.to_path_buf())
    }

    fn state_with_order(dir: &std::path::Path, order: &[&str]) -> ApiState {
        let mut config = Config::default();
        config.paths.tickets = dir.to_string_lossy().into_owned();
        config.queue.priority_order = order.iter().map(|t| (*t).to_string()).collect();
        ApiState::new(config, dir.to_path_buf())
    }

    /// Writes `<tickets>/<column>/<timestamp>-<type>-operator-<slug>.md`.
    fn write_ticket(
        root: &std::path::Path,
        column: &str,
        timestamp: &str,
        ticket_type: &str,
        id: &str,
        priority: &str,
        status: &str,
    ) {
        let dir = root.join(column);
        std::fs::create_dir_all(&dir).unwrap();
        let slug = id.to_lowercase();
        std::fs::write(
            dir.join(format!("{timestamp}-{ticket_type}-operator-{slug}.md")),
            format!(
                "---\nid: {id}\nstatus: {status}\npriority: {priority}\nstep: plan\n---\n\n# {id}\n"
            ),
        )
        .unwrap();
    }

    fn ids(column: &[KanbanTicketCard]) -> Vec<&str> {
        column.iter().map(|c| c.id.as_str()).collect()
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

    /// `TASK` is third in the default `queue.priority_order`, so it must not
    /// sort below `FEAT` on the board.
    #[tokio::test]
    async fn test_kanban_orders_task_above_feat() {
        let temp = tempfile::tempdir().unwrap();
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1100",
            "FEAT",
            "FEAT-1",
            "P2-medium",
            "queued",
        );
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1200",
            "TASK",
            "TASK-1",
            "P2-medium",
            "queued",
        );

        let board = kanban(State(make_state_in(temp.path()))).await.unwrap();
        assert_eq!(ids(&board.queue), vec!["TASK-1", "FEAT-1"]);
    }

    /// The ticket's own `priority:` breaks ties within a type, ahead of FIFO.
    #[tokio::test]
    async fn test_kanban_p0_sorts_above_p3_of_same_type() {
        let temp = tempfile::tempdir().unwrap();
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1100",
            "FEAT",
            "FEAT-LOW",
            "P3-low",
            "queued",
        );
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1200",
            "FEAT",
            "FEAT-HOT",
            "P0-critical",
            "queued",
        );

        let board = kanban(State(make_state_in(temp.path()))).await.unwrap();
        assert_eq!(ids(&board.queue), vec!["FEAT-HOT", "FEAT-LOW"]);
    }

    /// Collection-defined types are absent from `priority_order`, so they sort
    /// last - but must still order among themselves by priority, not tie.
    #[tokio::test]
    async fn test_kanban_unknown_issuetype_sorts_last_and_breaks_ties_by_priority() {
        let temp = tempfile::tempdir().unwrap();
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1000",
            "CHORE",
            "CHORE-LOW",
            "P3-low",
            "queued",
        );
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1100",
            "CHORE",
            "CHORE-HOT",
            "P0-critical",
            "queued",
        );
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1200",
            "FEAT",
            "FEAT-1",
            "P3-low",
            "queued",
        );

        let board = kanban(State(make_state_in(temp.path()))).await.unwrap();
        assert_eq!(ids(&board.queue), vec!["FEAT-1", "CHORE-HOT", "CHORE-LOW"]);
    }

    #[tokio::test]
    async fn test_kanban_priority_order_follows_config() {
        let temp = tempfile::tempdir().unwrap();
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1100",
            "FEAT",
            "FEAT-1",
            "P2-medium",
            "queued",
        );
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1200",
            "SPIKE",
            "SPIKE-1",
            "P2-medium",
            "queued",
        );

        let state = state_with_order(temp.path(), &["SPIKE", "FEAT"]);
        let board = kanban(State(state)).await.unwrap();
        assert_eq!(ids(&board.queue), vec!["SPIKE-1", "FEAT-1"]);
    }

    /// The done column is recency-ordered, not priority-ordered.
    #[tokio::test]
    async fn test_kanban_done_column_is_newest_first() {
        let temp = tempfile::tempdir().unwrap();
        write_ticket(
            temp.path(),
            "completed",
            "20240101-1100",
            "INV",
            "INV-OLD",
            "P0-critical",
            "completed",
        );
        write_ticket(
            temp.path(),
            "completed",
            "20240101-1200",
            "FEAT",
            "FEAT-NEW",
            "P3-low",
            "completed",
        );

        let board = kanban(State(make_state_in(temp.path()))).await.unwrap();
        assert_eq!(ids(&board.done), vec!["FEAT-NEW", "INV-OLD"]);
    }

    /// Every issuetype is counted, not just the four that used to be hardcoded.
    #[tokio::test]
    async fn test_queue_status_counts_task_and_custom_types() {
        let temp = tempfile::tempdir().unwrap();
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1100",
            "TASK",
            "TASK-1",
            "P2-medium",
            "queued",
        );
        write_ticket(
            temp.path(),
            "queue",
            "20240101-1200",
            "CHORE",
            "CHORE-1",
            "P2-medium",
            "queued",
        );

        let response = status(State(make_state_in(temp.path()))).await.unwrap();
        assert_eq!(response.by_type.0.get("TASK"), Some(&1));
        assert_eq!(response.by_type.0.get("CHORE"), Some(&1));
        // Configured types are always present, even at zero.
        assert_eq!(response.by_type.0.get("FEAT"), Some(&0));
    }

    #[test]
    fn test_ticket_to_card() {
        // Create a minimal ticket for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241229-1430-FEAT-operator-test.md");
        std::fs::write(
            &ticket_path,
            r"---
id: FEAT-1234
status: queued
priority: P2-medium
step: plan
---

# Feature: Test ticket for kanban
",
        )
        .unwrap();

        let ticket = Ticket::from_file(&ticket_path).unwrap();
        let card = ticket_to_card(&ticket);

        assert_eq!(card.id, "FEAT-1234");
        assert_eq!(card.ticket_type, "FEAT");
        assert_eq!(card.project, "operator");
        assert_eq!(card.status, TicketStatus::Queued);
        assert_eq!(card.priority, crate::queue::TicketPriority::P2Medium);
    }

    /// A status the schema does not know keeps the ticket in its directory's
    /// column rather than snapping back to the queue.
    #[tokio::test]
    async fn test_kanban_buckets_unknown_in_progress_status_as_running() {
        let temp = tempfile::tempdir().unwrap();
        write_ticket(
            temp.path(),
            "in-progress",
            "20240101-1100",
            "FEAT",
            "FEAT-1",
            "P2-medium",
            "gibberish",
        );

        let board = kanban(State(make_state_in(temp.path()))).await.unwrap();
        assert_eq!(ids(&board.running), vec!["FEAT-1"]);
        assert!(board.queue.is_empty());
    }
}
