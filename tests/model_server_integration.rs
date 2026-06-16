//! Live model-listing integration tests for the cloud model providers.
//!
//! These drive the real [`probe_models`] path against each provider's
//! model-listing endpoint — the same call the REST `/model-servers/.../models`
//! routes use to populate model dropdowns. Listing models consumes **no
//! inference tokens**, so these are cheap to run.
//!
//! ## Environment variables
//!
//! Keyed tests are **skipped** when their key is absent, so `cargo test` stays
//! green for contributors without credentials. Each var is read by the probe via
//! the instance's `api_key_env` (distinct from the operator's own runtime vars so
//! test keys never collide with a developer's live `ANTHROPIC_API_KEY`, etc.):
//!
//! - `OPERATOR_ANTHROPIC_API_KEY`  — <https://console.anthropic.com/settings/keys>
//! - `OPERATOR_OPENAI_API_KEY`     — <https://platform.openai.com/api-keys>
//! - `OPERATOR_GEMINI_API_KEY`     — <https://aistudio.google.com/app/apikey>
//! - `OPERATOR_OPENROUTER_API_KEY` — <https://openrouter.ai/keys> (optional)
//!
//! The `openrouter_keyless` test needs **no key** — `OpenRouter`'s `/models` list
//! is public — so it runs on every CI run as the always-on baseline. It skips
//! gracefully (rather than failing) if the network is unreachable.
//!
//! ## Running
//!
//! ```bash
//! # Keyless baseline only (needs network):
//! cargo test --test model_server_integration openrouter_keyless
//!
//! # A specific keyed provider (needs the key exported):
//! cargo test --test model_server_integration anthropic -- --nocapture --test-threads=1
//! ```

use operator::api::providers::model_server::{probe_models, ProbeOutcome};
use operator::config::ModelServer;
use std::env;
use tokio::sync::OnceCell;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Build a probe target for a kind. `base_url: None` lets the probe fall back to
/// the kind's `default_base_url`; `api_key_env` points the probe at the
/// `OPERATOR_*` test var (else the kind's default probe var).
fn server_for(kind: &str, api_key_env: Option<&str>) -> ModelServer {
    ModelServer {
        name: kind.to_string(),
        kind: kind.to_string(),
        base_url: None,
        api_key_env: api_key_env.map(str::to_string),
        extra_env: std::collections::HashMap::new(),
        display_name: None,
    }
}

fn configured(var: &str) -> bool {
    env::var(var).map(|s| !s.is_empty()).unwrap_or(false)
}

/// Skip the test body (with a notice) when the provider's key isn't configured.
macro_rules! skip_if_unconfigured {
    ($var:expr, $provider:expr) => {
        if !configured($var) {
            eprintln!("Skipping {}: {} not set", $provider, $var);
            return;
        }
    };
}

/// Assertions shared by every keyed provider: reachable, non-empty, and every
/// model carries a non-empty id (the wire shape dropdowns rely on).
fn assert_listable(outcome: &ProbeOutcome, provider: &str) {
    assert!(
        outcome.reachable,
        "{provider} should be reachable; error: {:?}",
        outcome.error
    );
    assert!(
        !outcome.models.is_empty(),
        "{provider} returned an empty model list"
    );
    assert!(
        outcome.models.iter().all(|m| !m.id.is_empty()),
        "{provider} returned a model with an empty id"
    );
}

// Cache one live probe per provider for the whole test run.
static ANTHROPIC: OnceCell<ProbeOutcome> = OnceCell::const_new();
static OPENAI: OnceCell<ProbeOutcome> = OnceCell::const_new();
static GOOGLE: OnceCell<ProbeOutcome> = OnceCell::const_new();
static OPENROUTER: OnceCell<ProbeOutcome> = OnceCell::const_new();

async fn anthropic_outcome() -> ProbeOutcome {
    ANTHROPIC
        .get_or_init(|| probe_models_owned("anthropic-api", "OPERATOR_ANTHROPIC_API_KEY"))
        .await
        .clone()
}

async fn openai_outcome() -> ProbeOutcome {
    OPENAI
        .get_or_init(|| probe_models_owned("openai-api", "OPERATOR_OPENAI_API_KEY"))
        .await
        .clone()
}

async fn google_outcome() -> ProbeOutcome {
    GOOGLE
        .get_or_init(|| probe_models_owned("google-api", "OPERATOR_GEMINI_API_KEY"))
        .await
        .clone()
}

async fn openrouter_outcome() -> ProbeOutcome {
    OPENROUTER
        .get_or_init(|| probe_models_owned("openrouter", "OPERATOR_OPENROUTER_API_KEY"))
        .await
        .clone()
}

async fn probe_models_owned(kind: &'static str, api_key_env: &'static str) -> ProbeOutcome {
    probe_models(&server_for(kind, Some(api_key_env))).await
}

// ─── Keyless OpenRouter baseline (no secret, runs every CI run) ───────────────

mod openrouter_keyless {
    use super::*;

    #[tokio::test]
    async fn test_public_models_list_is_text_filtered() {
        // No api_key_env and no OPENROUTER_API_KEY needed — the list is public.
        let outcome = probe_models(&server_for("openrouter", None)).await;

        if !outcome.reachable {
            // Treat a network/outage failure as a skip so offline `cargo test`
            // stays green; CI normally has connectivity.
            eprintln!(
                "Skipping keyless OpenRouter test: not reachable ({:?})",
                outcome.error
            );
            return;
        }

        assert!(
            !outcome.models.is_empty(),
            "public OpenRouter list should be non-empty"
        );
        assert!(
            outcome.models.iter().any(|m| {
                let id = m.id.to_ascii_lowercase();
                id.contains("claude") || id.contains("gpt") || id.contains("gemini")
            }),
            "OpenRouter list should include a recognizable text model"
        );
    }
}

// ─── Keyed per-provider tests ─────────────────────────────────────────────────

mod anthropic {
    use super::*;

    #[tokio::test]
    async fn test_lists_text_models() {
        skip_if_unconfigured!("OPERATOR_ANTHROPIC_API_KEY", "Anthropic");
        let outcome = anthropic_outcome().await;
        assert_listable(&outcome, "Anthropic");
        assert!(
            outcome.models.iter().any(|m| m.id.contains("claude")),
            "Anthropic list should contain a claude model"
        );
    }
}

mod openai {
    use super::*;

    #[tokio::test]
    async fn test_lists_text_models() {
        skip_if_unconfigured!("OPERATOR_OPENAI_API_KEY", "OpenAI");
        let outcome = openai_outcome().await;
        assert_listable(&outcome, "OpenAI");
        assert!(
            outcome.models.iter().any(|m| m.id.contains("gpt")),
            "OpenAI list should contain a gpt model"
        );
    }

    #[tokio::test]
    async fn test_excludes_non_text_models() {
        skip_if_unconfigured!("OPERATOR_OPENAI_API_KEY", "OpenAI");
        let outcome = openai_outcome().await;
        assert!(
            outcome.models.iter().all(|m| {
                let id = m.id.to_ascii_lowercase();
                !id.contains("embedding")
                    && !id.starts_with("whisper")
                    && !id.starts_with("tts")
                    && !id.starts_with("dall-e")
                    && !id.contains("moderation")
            }),
            "OpenAI list should be filtered to text models, got: {:?}",
            outcome.models.iter().map(|m| &m.id).collect::<Vec<_>>()
        );
    }
}

mod google {
    use super::*;

    #[tokio::test]
    async fn test_lists_text_models() {
        skip_if_unconfigured!("OPERATOR_GEMINI_API_KEY", "Google");
        let outcome = google_outcome().await;
        assert_listable(&outcome, "Google");
        assert!(
            outcome.models.iter().any(|m| m.id.contains("gemini")),
            "Google list should contain a gemini model"
        );
    }

    #[tokio::test]
    async fn test_excludes_embedding_models() {
        skip_if_unconfigured!("OPERATOR_GEMINI_API_KEY", "Google");
        let outcome = google_outcome().await;
        assert!(
            outcome.models.iter().all(|m| !m.id.contains("embedding")),
            "Google list should exclude embedding models, got: {:?}",
            outcome.models.iter().map(|m| &m.id).collect::<Vec<_>>()
        );
    }
}

mod openrouter_keyed {
    use super::*;

    #[tokio::test]
    async fn test_lists_text_models() {
        skip_if_unconfigured!("OPERATOR_OPENROUTER_API_KEY", "OpenRouter");
        let outcome = openrouter_outcome().await;
        assert_listable(&outcome, "OpenRouter");
        assert!(
            outcome.models.iter().any(|m| {
                let id = m.id.to_ascii_lowercase();
                id.contains("claude") || id.contains("gpt") || id.contains("gemini")
            }),
            "OpenRouter list should include a recognizable text model"
        );
    }
}

// ─── Cross-provider consistency ───────────────────────────────────────────────

/// Every configured + reachable provider returns the same uniform
/// `ModelInfo { id, display_name }` shape — a non-empty id for each model — so a
/// single dropdown component can render all of them. Skips when nothing is
/// configured.
#[tokio::test]
async fn test_all_providers_return_same_model_shape() {
    let mut checked = 0usize;

    for (var, provider, outcome) in [
        (
            "OPERATOR_ANTHROPIC_API_KEY",
            "Anthropic",
            anthropic_outcome().await,
        ),
        ("OPERATOR_OPENAI_API_KEY", "OpenAI", openai_outcome().await),
        ("OPERATOR_GEMINI_API_KEY", "Google", google_outcome().await),
        (
            "OPERATOR_OPENROUTER_API_KEY",
            "OpenRouter",
            openrouter_outcome().await,
        ),
    ] {
        if !configured(var) || !outcome.reachable {
            continue;
        }
        assert_listable(&outcome, provider);
        checked += 1;
    }

    if checked == 0 {
        eprintln!("Skipping consistency check: no providers configured/reachable");
    }
}
