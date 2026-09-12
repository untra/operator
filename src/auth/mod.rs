//! Authentication and authorization.
//!
//! Operator's HTTP surface is authenticated always: there is no configuration
//! flag that turns this off. What varies is how the first credential is
//! obtained - a loopback process gets one issued automatically (see
//! [`local`]), while any other bind must bootstrap an admin password.
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
