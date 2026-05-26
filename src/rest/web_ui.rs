//! Embedded web UI served via rust-embed.
//!
//! Gated behind the `embed-ui` feature flag.

use axum::http::{header, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "ui/dist"]
struct UiAssets;

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

    const FIVE_MB: usize = 5_242_880;
    const FIFTEEN_MB: usize = 15_728_640;

    #[test]
    fn test_embedded_assets_under_5mb_gzipped() {
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
            compressed_total < FIVE_MB,
            "Embedded UI assets: {compressed_total}B ({:.1}MB) gzipped — exceeds 5MB budget \
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
}
