//! Model Context Protocol (MCP) integration for Operator.
//!
//! Provides an MCP server bridge that exposes Operator's REST API as
//! read-only MCP tools. Includes a descriptor endpoint for client discovery,
//! tool definitions, and an SSE transport for JSON-RPC communication.

pub mod descriptor;
pub mod tools;
pub mod transport;
