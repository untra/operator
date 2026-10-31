//! Loopback client for this workspace's own REST API.
//!
//! The TUI and the CLI create tickets through the same HTTP endpoints as the
//! web UI and the VS Code extension, authenticated with the local-unlock token
//! (see [`crate::auth::local`]), so every surface shares one contract.

use std::time::Duration;

use reqwest::{Method, RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::config::Config;
use crate::rest::dto::{CreateTicketRequest, CreateTicketResponse};
use crate::rest::error::ErrorResponse;

const LOOPBACK_HOST: &str = "127.0.0.1";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum LocalApiError {
    /// Nothing answered: the API server is not running on this port.
    #[error("the operator API is not reachable at {url}: {message}")]
    Unreachable { url: String, message: String },
    /// The API answered with an error.
    #[error("{message}")]
    Api { status: StatusCode, message: String },
    #[error("unexpected response from the operator API: {0}")]
    Decode(String),
}

#[derive(Debug, Clone)]
pub struct LocalApiClient {
    base_url: String,
    token: Option<String>,
    http: reqwest::Client,
}

impl LocalApiClient {
    pub fn for_config(config: &Config) -> Result<Self, LocalApiError> {
        let base_url = format!("http://{LOOPBACK_HOST}:{}", config.rest_api.port);
        let http = crate::http_client::client_builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| LocalApiError::Unreachable {
                url: base_url.clone(),
                message: e.to_string(),
            })?;
        Ok(Self {
            token: crate::auth::local::read(&config.auth_state_path()),
            base_url,
            http,
        })
    }

    pub async fn create_ticket(
        &self,
        request: &CreateTicketRequest,
    ) -> Result<CreateTicketResponse, LocalApiError> {
        self.send(Method::POST, "/api/v1/tickets", Some(request))
            .await
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let builder = self
            .http
            .request(method, format!("{}{path}", self.base_url));
        match &self.token {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        }
    }

    async fn send<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, LocalApiError> {
        let mut builder = self.request(method, path);
        if let Some(body) = body {
            builder = builder.json(body);
        }
        let response = builder
            .send()
            .await
            .map_err(|e| LocalApiError::Unreachable {
                url: self.base_url.clone(),
                message: e.to_string(),
            })?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            let message =
                serde_json::from_str::<ErrorResponse>(&text).map_or(text, |error| error.message);
            return Err(LocalApiError::Api { status, message });
        }
        response
            .json()
            .await
            .map_err(|e| LocalApiError::Decode(e.to_string()))
    }
}
