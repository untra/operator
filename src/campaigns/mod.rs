//! Campaigns: epics of tickets planned into dependency waves and launched together.

pub mod batch;
pub mod model;
pub mod plan;
pub mod runner;
pub mod service;
pub mod store;
pub mod workspace;

pub use store::{members, CampaignStore, NewCampaign};
