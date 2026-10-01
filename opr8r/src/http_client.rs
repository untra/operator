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
pub fn client_builder() -> reqwest::ClientBuilder {
    ensure_crypto_provider();
    reqwest::Client::builder()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_builds_with_ring_provider() {
        assert!(client_builder().build().is_ok());
        assert!(rustls::crypto::CryptoProvider::get_default().is_some());
    }
}
