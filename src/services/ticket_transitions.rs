//! The one write path for moving a ticket between board columns.
//!
//! Every surface that moves a ticket - the MCP tools, the REST API, the web
//! board - goes through [`move_ticket`] so a move is always mirrored to the
//! external board the ticket came from. Bypassing this (renaming the file
//! directly) leaves a synced ticket stranded in its old column upstream.

use crate::queue::{Queue, Ticket, TicketColumn};
use crate::rest::state::ApiState;

/// Mirror a completed local move to the upstream kanban board.
///
/// Fire-and-forget: a provider that is unreachable must not fail the local
/// move. No-op unless bidirectional sync is configured (`state.kanban_sync`).
fn push_kanban_transition(state: &ApiState, ticket: &Ticket, column: TicketColumn) {
    let Some(ks) = state.kanban_sync.clone() else {
        return;
    };
    let ticket = ticket.clone();
    tokio::spawn(async move {
        match column {
            TicketColumn::InProgress => ks.on_ticket_claimed(&ticket).await,
            TicketColumn::Completed => ks.on_ticket_completed(&ticket).await,
            TicketColumn::Queue => ks.on_ticket_requeued(&ticket).await,
        }
    });
}

/// Move a ticket into `column` and mirror the move upstream.
///
/// The local move is authoritative: it completes before the upstream push is
/// spawned, and a push failure never rolls it back.
pub async fn move_ticket(
    state: &ApiState,
    ticket: &Ticket,
    column: TicketColumn,
) -> Result<(), String> {
    let config = (*state.config()).clone();
    let moved = ticket.clone();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let queue = Queue::new(&config).map_err(|e| e.to_string())?;
        queue.move_ticket(&moved, column).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    push_kanban_transition(state, ticket, column);
    Ok(())
}
