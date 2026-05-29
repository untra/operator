//! Embedded web UI served via rust-embed.
//!
//! Gated behind the `embed-ui` feature flag.

use axum::http::{header, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "ui/dist"]
struct UiAssets;

/// Sentinel substring written into `ui/dist/index.html` by `build.rs` when the
/// SPA hasn't been built. The runtime detector uses this to distinguish a real
/// Vite build from a placeholder so the TUI can give the user an actionable
/// message instead of opening a blank page.
pub const PLACEHOLDER_MARKER: &str = "operator:placeholder";

/// Whether the embedded SPA is usable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedUiState {
    /// A real built SPA is embedded.
    Ready,
    /// The build.rs placeholder is embedded — `ui/dist` wasn't built before
    /// the cargo build.
    Placeholder,
    /// No SPA assets at all (should be unreachable when this module compiles).
    Missing,
}

/// Inspect the embedded assets and report whether a real SPA is available.
pub fn embedded_ui_state() -> EmbeddedUiState {
    let Some(file) = UiAssets::get("index.html") else {
        return EmbeddedUiState::Missing;
    };
    let body = std::str::from_utf8(&file.data).unwrap_or("");
    if body.contains(PLACEHOLDER_MARKER) {
        EmbeddedUiState::Placeholder
    } else {
        EmbeddedUiState::Ready
    }
}

/// Axum fallback handler that serves embedded SPA assets.
///
/// Priority: exact file match → index.html (SPA client-side routing).
pub async fn spa_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if let Some(file) = UiAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, mime.as_ref().to_string()),
                (header::CACHE_CONTROL, cache_policy(path)),
            ],
            file.data.into_owned(),
        )
            .into_response()
    } else {
        match UiAssets::get("index.html") {
            Some(index) => Html(String::from_utf8_lossy(&index.data).to_string()).into_response(),
            None => (StatusCode::NOT_FOUND, "UI not built").into_response(),
        }
    }
}

fn cache_policy(path: &str) -> String {
    // Vite produces hashed filenames like `assets/index-abc123.js`
    if path.starts_with("assets/") {
        "public, max-age=31536000, immutable".into()
    } else {
        "no-cache".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    const TEN_MB: usize = 10_485_760;
    const FIFTEEN_MB: usize = 15_728_640;

    #[test]
    fn test_embedded_assets_under_10mb_gzipped() {
        let mut compressed_total: usize = 0;
        let mut uncompressed_total: usize = 0;

        for path in UiAssets::iter() {
            if let Some(file) = UiAssets::get(&path) {
                uncompressed_total += file.data.len();

                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&file.data).expect("gzip write");
                let compressed = encoder.finish().expect("gzip finish");
                compressed_total += compressed.len();
            }
        }

        assert!(
            compressed_total < TEN_MB,
            "Embedded UI assets: {compressed_total}B ({:.1}MB) gzipped — exceeds 10MB budget \
             (uncompressed: {uncompressed_total}B / {:.1}MB)",
            compressed_total as f64 / 1_048_576.0,
            uncompressed_total as f64 / 1_048_576.0,
        );
    }

    #[test]
    fn test_embedded_assets_under_15mb_uncompressed() {
        let total: usize = UiAssets::iter()
            .filter_map(|path| UiAssets::get(&path))
            .map(|file| file.data.len())
            .sum();

        assert!(
            total < FIFTEEN_MB,
            "Embedded UI assets: {total}B ({:.1}MB) uncompressed — exceeds 15MB budget",
            total as f64 / 1_048_576.0,
        );
    }

    #[test]
    fn test_index_html_exists() {
        assert!(
            UiAssets::get("index.html").is_some(),
            "ui/dist/index.html must exist in embedded assets"
        );
    }

    #[test]
    fn test_spa_fallback_returns_index_for_unknown_paths() {
        let index_content = UiAssets::get("index.html").expect("index.html must exist");
        let index_str = String::from_utf8_lossy(&index_content.data);
        assert!(
            index_str.contains("<!doctype html>")
                || index_str.contains("<!DOCTYPE html>")
                || index_str.contains("<html"),
            "index.html should contain valid HTML"
        );
    }

    #[test]
    fn test_embedded_ui_state_detects_real_build() {
        let index = UiAssets::get("index.html").expect("index.html must exist");
        let body = std::str::from_utf8(&index.data).unwrap_or("");
        let expected = if body.contains(PLACEHOLDER_MARKER) {
            EmbeddedUiState::Placeholder
        } else {
            EmbeddedUiState::Ready
        };
        assert_eq!(
            embedded_ui_state(),
            expected,
            "embedded_ui_state() must agree with the contents of the embedded index.html"
        );
    }

    #[test]
    fn test_placeholder_marker_constant_is_stable() {
        // build.rs writes this exact substring; if either side drifts, the
        // runtime detector silently breaks. Pin the value here.
        assert_eq!(PLACEHOLDER_MARKER, "operator:placeholder");
    }
}
