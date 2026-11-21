//! Read-only MCP tools over health, issue types, collections and skills.

use crate::rest::state::RequestState as State;
use axum::extract::{Path, Query};
use serde_json::Value;

use super::args::{json_value, optional_string, required_str};
use crate::rest::routes;
use crate::rest::routes::issuetypes::CollectionQuery;
use crate::rest::state::ApiState;

pub async fn health(_args: Value, state: &ApiState) -> Result<Value, String> {
    json_value(routes::health::health(State(state.clone())).await)
}

pub async fn status(_args: Value, state: &ApiState) -> Result<Value, String> {
    json_value(routes::health::status(State(state.clone())).await)
}

pub async fn list_issue_types(args: Value, state: &ApiState) -> Result<Value, String> {
    let collection = optional_string(&args, "collection");
    routes::issuetypes::list(State(state.clone()), Query(CollectionQuery { collection }))
        .await
        .map_err(|_| "Unknown collection".to_string())
        .and_then(json_value)
}

pub async fn get_issue_type(args: Value, state: &ApiState) -> Result<Value, String> {
    let key = required_str(&args, "key")?;
    let collection = optional_string(&args, "collection");
    routes::issuetypes::get_one(
        State(state.clone()),
        Path(key.to_string()),
        Query(CollectionQuery { collection }),
    )
    .await
    .map_err(|_| format!("Issue type '{key}' not found"))
    .and_then(json_value)
}

pub async fn list_collections(_args: Value, state: &ApiState) -> Result<Value, String> {
    json_value(routes::collections::list(State(state.clone())).await)
}

pub async fn get_collection(args: Value, state: &ApiState) -> Result<Value, String> {
    let name = required_str(&args, "name")?;
    routes::collections::get_one(State(state.clone()), Path(name.to_string()))
        .await
        .map_err(|_| format!("Collection '{name}' not found"))
        .and_then(json_value)
}

pub async fn list_skills(_args: Value, state: &ApiState) -> Result<Value, String> {
    json_value(routes::skills::list(State(state.clone())).await)
}
