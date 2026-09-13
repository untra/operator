//! Render methods for each setup step

mod acceptance;
mod admin_password;
mod collection;
mod confirm;
mod git;
mod hosted;
mod kanban;
mod model_server;
mod startup;
mod target;
mod task_fields;
mod welcome;
mod wrapper;

pub use acceptance::*;
pub use collection::*;
pub use confirm::*;
pub use git::*;
pub use hosted::*;
pub use kanban::*;
pub use model_server::*;
pub use startup::*;
pub use target::*;
pub use task_fields::*;
pub use welcome::*;
pub use wrapper::*;
