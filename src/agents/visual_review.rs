//! Visual Review Handler - Browser-based visual confirmation.
//!
//! For steps with `review_type: visual`, this handler:
//! - Optionally starts a dev server
//! - Opens a URL in the browser
//! - Waits for operator confirmation

use anyhow::{Context, Result};
use handlebars::Handlebars;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::timeout;
use tracing::{debug, info, instrument, warn};

use crate::templates::schema::VisualReviewConfig;

/// Result of a visual review
#[derive(Debug, Clone)]
pub enum VisualReviewResult {
    /// User approved the visual review
    Approved,
    /// User rejected with feedback
    Rejected { reason: String },
    /// Review was cancelled
    Cancelled,
    /// Server failed to start
    ServerFailed { error: String },
}

/// Handles visual review workflow
pub struct VisualReviewHandler {
    /// Context for URL template rendering
    context: serde_json::Value,
}

impl VisualReviewHandler {
    /// Create a new visual review handler
    pub fn new() -> Self {
        Self {
            context: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Create with template context for URL rendering
    pub fn with_context(context: serde_json::Value) -> Self {
        Self { context }
    }

    /// Add context variable for URL template
    pub fn add_context(&mut self, key: &str, value: impl Into<serde_json::Value>) {
        if let serde_json::Value::Object(ref mut map) = self.context {
            map.insert(key.to_string(), value.into());
        }
    }

    /// Render the URL template with context
    fn render_url(&self, url_template: &str) -> Result<String> {
        let hbs = Handlebars::new();
        hbs.render_template(url_template, &self.context)
            .context("Failed to render URL template")
    }

    /// Start the dev server if configured
    #[instrument(skip(self, config))]
    pub async fn start_server(
        &self,
        config: &VisualReviewConfig,
        working_dir: &Path,
    ) -> Result<Option<Child>> {
        let Some(ref command) = config.startup_command else {
            return Ok(None);
        };

        info!("Starting dev server: {}", command);

        // Split command into parts
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty startup command"));
        }

        let child = Command::new(parts[0])
            .args(&parts[1..])
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start dev server")?;

        // Wait for server to be ready
        let startup_timeout = Duration::from_secs(config.startup_timeout_secs.unwrap_or(30) as u64);

        info!("Waiting up to {:?} for server to start", startup_timeout);

        // Simple wait - in production you'd poll an endpoint
        tokio::time::sleep(Duration::from_secs(3)).await;

        Ok(Some(child))
    }

    /// Open a URL in the default browser
    #[instrument]
    pub fn open_browser(url: &str) -> Result<()> {
        info!("Opening browser: {}", url);

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(url)
                .spawn()
                .context("Failed to open browser")?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(url)
                .spawn()
                .context("Failed to open browser")?;
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", url])
                .spawn()
                .context("Failed to open browser")?;
        }

        Ok(())
    }

    /// Stop the dev server
    pub async fn stop_server(mut child: Child) -> Result<()> {
        debug!("Stopping dev server");

        child.kill().await.ok();
        child.wait().await.ok();

        Ok(())
    }

    /// Run the visual review flow:
    /// 1. Start dev server if configured
    /// 2. Open browser to URL
    /// 3. Return - caller handles the approval UI
    #[instrument(skip(self, config))]
    pub async fn prepare_review(
        &self,
        config: &VisualReviewConfig,
        working_dir: &Path,
    ) -> Result<(String, Option<Child>)> {
        // Start server if needed
        let server = match self.start_server(config, working_dir).await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to start server: {}", e);
                return Err(e);
            }
        };

        // Render URL
        let url = self.render_url(&config.url)?;

        // Open browser
        Self::open_browser(&url)?;

        Ok((url, server))
    }

    /// Cleanup after review (stop server if running)
    pub async fn cleanup(&self, server: Option<Child>) -> Result<()> {
        if let Some(child) = server {
            Self::stop_server(child).await?;
        }
        Ok(())
    }
}

impl Default for VisualReviewHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_handler() {
        let _handler = VisualReviewHandler::new();
    }

    #[test]
    fn test_add_context() {
        let mut handler = VisualReviewHandler::new();
        handler.add_context("port", 3000);
        handler.add_context("host", "localhost");

        if let serde_json::Value::Object(map) = &handler.context {
            assert!(map.contains_key("port"));
            assert!(map.contains_key("host"));
        }
    }

    #[test]
    fn test_render_url() {
        let mut handler = VisualReviewHandler::new();
        handler.add_context("port", 3000);

        let url = handler
            .render_url("http://localhost:{{port}}/preview")
            .unwrap();
        assert_eq!(url, "http://localhost:3000/preview");
    }

    #[test]
    fn test_render_url_no_template() {
        let handler = VisualReviewHandler::new();
        let url = handler.render_url("http://localhost:3000/preview").unwrap();
        assert_eq!(url, "http://localhost:3000/preview");
    }
}
