//! Authentication and authorization.
//!
//! Operator's HTTP surface is authenticated always: there is no configuration
//! flag that turns this off. What varies is how the first credential is
//! obtained. Local mode supports owner-only credentials, and hosted mode uses OIDC.
//!
//! This module lives in the library, not the binary, because `src/rest` does
//! and depends on it. It must not reference bin-only `crate::ui`.

pub mod callback;
pub mod egress;
pub mod local;
pub mod password;
pub mod schema;
pub mod scope;
pub mod secret;
pub mod store;
pub mod tokens;

pub mod audit;

pub mod identity;
pub mod oidc;
pub mod oidc_bearer;
pub mod policy;
pub mod provision;
pub mod usage;

pub mod limits;
pub mod metrics;
