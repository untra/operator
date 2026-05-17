//! Model Context Protocol (MCP) integration for Operator.
//!
//! Provides an MCP server bridge that exposes Operator's REST API as
//! read-only MCP tools. Includes a descriptor endpoint for client discovery,
//! tool definitions, and an SSE transport for JSON-RPC communication.

pub mod client_configs;
pub mod descriptor;
pub mod handler;
pub mod resources;
pub mod stdio;
pub mod tickets;
pub mod tools;
pub mod transport;
