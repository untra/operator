//! Render methods for each setup step

mod acceptance;
mod collection;
mod confirm;
mod kanban;
mod startup;
mod task_fields;
mod welcome;
mod wrapper;

pub use acceptance::*;
pub use collection::*;
pub use confirm::*;
pub use kanban::*;
pub use startup::*;
pub use task_fields::*;
pub use welcome::*;
pub use wrapper::*;
