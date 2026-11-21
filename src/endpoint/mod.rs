//! Endpoint clients: machines (e.g. a Windows gaming PC) that pair with this
//! server, report capabilities, tunnel local model servers, and run launches.
//!
//! [`protocol`] is the single source of the uplink wire format; the C# client
//! consumes bindings generated from it by [`csharp`].

// Binding generators: reached only through the lib's `generate_types` binary.
#[allow(dead_code)]
pub mod bindings;
#[allow(dead_code)]
pub mod csharp;
pub mod protocol;
pub mod registry;
pub mod uplink;
