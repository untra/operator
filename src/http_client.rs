//! Single construction point for outbound HTTP clients.
//!
//! reqwest is built with `rustls-no-provider` so the tree stays on ring
//! (no aws-lc C build). rustls then needs a process-default `CryptoProvider`
//! installed before any client is built; every client goes through here.

use std::sync::Once;

static INSTALL_PROVIDER: Once = Once::new();

fn ensure_crypto_provider() {
    INSTALL_PROVIDER.call_once(|| {
        // Err means another caller already installed a provider, which is fine.
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

#[allow(clippy::disallowed_methods)]
pub fn default_client() -> reqwest::Client {
    ensure_crypto_provider();
    reqwest::Client::new()
}

#[allow(clippy::disallowed_methods)]
pub fn client_builder() -> reqwest::ClientBuilder {
    ensure_crypto_provider();
    reqwest::Client::builder()
}

#[allow(clippy::disallowed_methods)]
pub fn blocking_client_builder() -> reqwest::blocking::ClientBuilder {
    ensure_crypto_provider();
    reqwest::blocking::Client::builder()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_client_builds_with_ring_provider() {
        assert!(client_builder().build().is_ok());
    }

    #[test]
    fn blocking_client_builds_with_ring_provider() {
        assert!(blocking_client_builder().build().is_ok());
    }

    #[test]
    fn default_client_installs_ring_provider() {
        let _client = default_client();
        assert!(rustls::crypto::CryptoProvider::get_default().is_some());
    }

    #[test]
    fn repeated_builds_do_not_reinstall() {
        assert!(client_builder().build().is_ok());
        assert!(client_builder().build().is_ok());
        assert!(rustls::crypto::CryptoProvider::get_default().is_some());
    }
}
