//! Minting the single-purpose credential an agent uses to report step completion.
//!
//! The `opr8r` client calls `POST /api/v1/tickets/{id}/steps/{step}/complete`,
//! which launches processes and advances a workflow. Before authentication that
//! endpoint was reachable by anything that could open a socket to the port.
//!
//! The token minted here is deliberately narrow rather than long-lived-and-broad:
//! it carries only `execute`, is issued for the `opr8r-callback` audience so it
//! cannot authenticate any ordinary API route, and pins the ticket and step the
//! handler then matches against the request path.

use anyhow::{Context, Result};
use chrono::{Duration, Utc};

use crate::auth::store::{AuthStore, ADMIN_SUBJECT};
use crate::auth::tokens::callback_claims;
use crate::config::Config;

/// Lifetime of a callback token.
///
/// Deliberately far longer than the 15-minute access-token TTL. A step may legitimately run for hours, and a credential that expired mid-run would
/// strand an agent holding completed work it cannot report - turning a security control into a reliability bug. The token is bounded by its claims instead of by the clock.
const CALLBACK_TTL: Duration = Duration::hours(24);

/// Mint a callback token for one ticket, step, and agent session.
///
/// Opens the auth database rather than holding a handle: launches are infrequent, `Launcher` is constructed from a bare `Config` in the CLI,
/// the TUI, and the REST API alike, and threading an auth handle through all three would be a large change for a per-launch cost that is already dominated by spawning a process.
pub fn mint(config: &Config, ticket_id: &str, step: &str, session_id: &str) -> Result<String> {
    let store = AuthStore::open(&config.state_path()).context("opening the auth store")?;
    let key = store
        .load_or_create_signing_key()
        .context("loading the token signing key")?;

    let claims = callback_claims(
        ADMIN_SUBJECT,
        ticket_id,
        step,
        session_id,
        CALLBACK_TTL,
        Utc::now(),
        uuid::Uuid::new_v4().to_string(),
    );
    key.sign(&claims).context("signing the callback token")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::tokens::{AUDIENCE_API, AUDIENCE_CALLBACK};
    use crate::rest::dto::auth::Scope;

    fn config_in(dir: &std::path::Path) -> Config {
        let mut config = Config::default();
        config.paths.state = dir.to_string_lossy().to_string();
        config
    }

    #[test]
    fn test_minted_token_verifies_and_is_pinned_to_its_step() {
        let dir = tempfile::tempdir().unwrap();
        let config = config_in(dir.path());

        let token = mint(&config, "FEAT-42", "build", "sess-1").unwrap();

        let store = AuthStore::open(&config.state_path()).unwrap();
        let key = store.load_or_create_signing_key().unwrap();
        let claims = key.verify(&token, AUDIENCE_CALLBACK).unwrap();

        assert_eq!(claims.ticket_id.as_deref(), Some("FEAT-42"));
        assert_eq!(claims.step.as_deref(), Some("build"));
        assert_eq!(claims.session_id.as_deref(), Some("sess-1"));
        assert_eq!(claims.scopes(), vec![Scope::Execute]);
    }

    #[test]
    fn test_callback_token_cannot_authenticate_ordinary_api_routes() {
        // The audience is the boundary: an agent that leaks its callback token
        // has not leaked API access.
        let dir = tempfile::tempdir().unwrap();
        let config = config_in(dir.path());
        let token = mint(&config, "FEAT-42", "build", "sess-1").unwrap();

        let store = AuthStore::open(&config.state_path()).unwrap();
        let key = store.load_or_create_signing_key().unwrap();
        assert!(key.verify(&token, AUDIENCE_API).is_err());
    }

    #[test]
    fn test_tokens_minted_across_calls_share_the_persisted_key() {
        // Minting must not rotate the signing key; doing so would invalidate
        // every other credential on every launch.
        let dir = tempfile::tempdir().unwrap();
        let config = config_in(dir.path());

        let first = mint(&config, "FEAT-1", "build", "s1").unwrap();
        let second = mint(&config, "FEAT-2", "review", "s2").unwrap();

        let store = AuthStore::open(&config.state_path()).unwrap();
        let key = store.load_or_create_signing_key().unwrap();
        assert!(key.verify(&first, AUDIENCE_CALLBACK).is_ok());
        assert!(key.verify(&second, AUDIENCE_CALLBACK).is_ok());
    }

    #[test]
    fn test_token_outlives_a_long_running_step() {
        let dir = tempfile::tempdir().unwrap();
        let config = config_in(dir.path());
        let token = mint(&config, "FEAT-1", "build", "s1").unwrap();

        let store = AuthStore::open(&config.state_path()).unwrap();
        let key = store.load_or_create_signing_key().unwrap();
        let claims = key.verify(&token, AUDIENCE_CALLBACK).unwrap();

        // A multi-hour step must not lose the ability to report completion.
        assert!(
            claims.exp - claims.iat >= 8 * 60 * 60,
            "callback tokens must outlive a long step"
        );
    }
}
