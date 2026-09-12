//! Outbound request destination validation (SSRF guard).
//!
//! Several Operator features fetch a URL that configuration or a request body
//! supplies. The sharpest is the model-server probe: a caller sets a base URL,
//! triggers a probe, and Operator makes the request **with the provider's API
//! key attached**. Pointed at `169.254.169.254`, that reads cloud instance
//! credentials; pointed at an internal address, it is a port scanner with a
//! bearer token.
//!
//! Authentication and scopes are the first control - probing needs `execute`,
//! changing a URL needs `admin`. This module is the second: even an authorized
//! caller cannot aim Operator at the loopback interface, link-local space, or
//! the cloud metadata endpoint.
//!
//! Redirects are re-validated. Every existing call site uses reqwest's default
//! redirect policy, so validating only the initial URL would let an allowed
//! host bounce the request to a forbidden one.

use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use anyhow::{anyhow, Result};
use url::Url;

/// The cloud instance metadata address, blocked on every major provider.
const CLOUD_METADATA_V4: Ipv4Addr = Ipv4Addr::new(169, 254, 169, 254);

/// Schemes Operator will speak. `file://`, `ftp://`, and friends have no legitimate use here and are a classic SSRF escape hatch.
const ALLOWED_SCHEMES: &[&str] = &["http", "https"];

/// What an outbound request is allowed to reach.
#[derive(Debug, Clone)]
pub struct EgressPolicy {
    /// Permit loopback destinations.
    ///
    /// On by default because Operator's normal local workflow talks to `localhost` model servers - Ollama, LM Studio, an OpenAI-compatible proxy.
    /// It is turned **off** in a published deployment, where loopback means the container's own interfaces rather than the user's laptop.
    pub allow_loopback: bool,
    /// Permit RFC 1918 / unique-local addresses, for a self-hosted provider on the same network.
    pub allow_private: bool,
}

impl Default for EgressPolicy {
    fn default() -> Self {
        Self {
            allow_loopback: true,
            allow_private: true,
        }
    }
}

impl EgressPolicy {
    /// The policy for a published deployment: no loopback, no private ranges.
    pub fn hardened() -> Self {
        Self {
            allow_loopback: false,
            allow_private: false,
        }
    }

    /// Derive the policy from configuration.
    ///
    /// A deployment that has declared a public URL is reachable from outside,
    /// so its outbound reach is narrowed to match.
    pub fn from_config(config: &crate::config::Config) -> Self {
        if config.rest_api.public_base_url().is_some() {
            Self::hardened()
        } else {
            Self::default()
        }
    }
}

/// Whether an address is in a range that must never be reachable.
///
/// These are unconditional: no configuration turns them on, because none of
/// them is a destination Operator has any business reaching.
fn is_always_forbidden(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4 == CLOUD_METADATA_V4
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.is_documentation()
                // 0.0.0.0/8 "this network"
                || v4.octets()[0] == 0
        }
        IpAddr::V6(v6) => {
            v6.is_multicast()
                || v6.is_unspecified()
                // fe80::/10 link-local
                || (v6.segments()[0] & 0xffc0) == 0xfe80
                // IPv4-mapped addresses re-enter the v4 rules; without this a forbidden v4 address could be smuggled in as ::ffff:169.254.169.254
                || v6.to_ipv4_mapped().is_some_and(|v4| is_always_forbidden(IpAddr::V4(v4)))
        }
    }
}

/// Whether an address is in a private range.
fn is_private(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private(),
        // fc00::/7 unique local
        IpAddr::V6(v6) => (v6.segments()[0] & 0xfe00) == 0xfc00,
    }
}

/// Check one resolved address against the policy.
pub fn check_addr(ip: IpAddr, policy: &EgressPolicy) -> Result<()> {
    if is_always_forbidden(ip) {
        return Err(anyhow!(
            "destination {ip} is in a reserved range (link-local, multicast, or cloud metadata) \
             and is never permitted"
        ));
    }
    if ip.is_loopback() && !policy.allow_loopback {
        return Err(anyhow!(
            "destination {ip} is loopback, which this deployment does not permit"
        ));
    }
    if is_private(ip) && !policy.allow_private {
        return Err(anyhow!(
            "destination {ip} is a private address, which this deployment does not permit"
        ));
    }
    Ok(())
}

/// Validate a URL's scheme and, when the host is an IP literal, its address.
///
/// A hostname is *not* resolved here. DNS resolution followed by a separate
/// connection is a time-of-check/time-of-use gap (DNS rebinding), so the
/// authoritative check is [`check_addr`] applied to the address actually
/// connected to - see [`validated_client`].
pub fn check_url(url: &Url, policy: &EgressPolicy) -> Result<()> {
    if !ALLOWED_SCHEMES.contains(&url.scheme()) {
        return Err(anyhow!(
            "scheme `{}` is not permitted; use http or https",
            url.scheme()
        ));
    }

    // Match on the parsed host rather than the string. `host_str()` renders an
    // IPv6 literal in its bracketed form (`[::1]`), which does not parse as an
    // `IpAddr` - so string-parsing silently treated every IPv6 literal as a hostname and skipped the address checks entirely.
    match url.host() {
        Some(url::Host::Ipv4(v4)) => check_addr(IpAddr::V4(v4), policy),
        Some(url::Host::Ipv6(v6)) => check_addr(IpAddr::V6(v6), policy),
        // A name is judged at connect time, not here; see the doc comment.
        Some(url::Host::Domain(name)) if !name.is_empty() => Ok(()),
        _ => Err(anyhow!("destination URL has no host")),
    }
}

/// Parse and validate a destination URL.
pub fn validate(raw: &str, policy: &EgressPolicy) -> Result<Url> {
    let url = Url::parse(raw).map_err(|e| anyhow!("invalid URL {raw:?}: {e}"))?;
    check_url(&url, policy)?;
    Ok(url)
}

/// Build an HTTP client that enforces `policy` on the initial request and on
/// every redirect hop.
///
/// The redirect policy is where this earns its keep: an allowed host can answer
/// `302 Location: http://169.254.169.254/...`, and reqwest follows redirects by
/// default at every existing call site.
pub fn validated_client(policy: EgressPolicy, timeout: Duration) -> Result<reqwest::Client> {
    let redirect_policy = reqwest::redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() >= 10 {
            return attempt.error("too many redirects");
        }
        match check_url(attempt.url(), &policy) {
            Ok(()) => attempt.follow(),
            Err(e) => attempt.error(e),
        }
    });

    reqwest::Client::builder()
        .timeout(timeout)
        .redirect(redirect_policy)
        .build()
        .map_err(|e| anyhow!("building HTTP client: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv6Addr;

    fn permissive() -> EgressPolicy {
        EgressPolicy::default()
    }

    #[test]
    fn test_cloud_metadata_is_never_reachable() {
        // The single most valuable SSRF target: it serves instance credentials to anything that asks, with no authentication.
        for policy in [EgressPolicy::default(), EgressPolicy::hardened()] {
            assert!(validate("http://169.254.169.254/latest/meta-data/", &policy).is_err());
            // IPv4-mapped IPv6 must not be a way around it.
            assert!(validate("http://[::ffff:169.254.169.254]/", &policy).is_err());
        }
    }

    #[test]
    fn test_link_local_and_multicast_are_never_reachable() {
        for url in [
            "http://169.254.1.1/",
            "http://224.0.0.1/",
            "http://[ff02::1]/",
            "http://[fe80::1]/",
            "http://0.0.0.0/",
        ] {
            assert!(
                validate(url, &permissive()).is_err(),
                "{url} must be refused even under the permissive policy"
            );
        }
    }

    #[test]
    fn test_non_http_schemes_are_refused() {
        for url in [
            "file:///etc/passwd",
            "ftp://example.com/",
            "gopher://example.com/",
        ] {
            assert!(
                validate(url, &permissive()).is_err(),
                "{url} must be refused"
            );
        }
    }

    #[test]
    fn test_loopback_follows_the_policy() {
        // Local model servers are the normal case on a developer machine...
        assert!(validate("http://127.0.0.1:11434/api/tags", &permissive()).is_ok());
        assert!(validate("http://[::1]:11434/", &permissive()).is_ok());

        // ...but in a published deployment loopback is the container itself.
        assert!(validate("http://127.0.0.1:11434/", &EgressPolicy::hardened()).is_err());
        assert!(validate("http://[::1]:11434/", &EgressPolicy::hardened()).is_err());
    }

    #[test]
    fn test_private_ranges_follow_the_policy() {
        for url in [
            "http://10.1.2.3/",
            "http://192.168.1.5/",
            "http://172.16.0.9/",
        ] {
            assert!(validate(url, &permissive()).is_ok(), "{url} under default");
            assert!(
                validate(url, &EgressPolicy::hardened()).is_err(),
                "{url} under hardened"
            );
        }
    }

    #[test]
    fn test_ordinary_public_destinations_are_allowed() {
        for url in [
            "https://api.anthropic.com/v1/messages",
            "https://api.openai.com/v1/models",
            "http://example.com:8080/path?q=1",
        ] {
            assert!(validate(url, &EgressPolicy::hardened()).is_ok(), "{url}");
        }
    }

    #[test]
    fn test_hostnames_pass_url_validation_and_are_judged_at_connect_time() {
        // Resolving here and connecting later is a rebinding gap, so a name is deliberately not resolved at this stage.
        assert!(validate(
            "https://ollama.internal/api/tags",
            &EgressPolicy::hardened()
        )
        .is_ok());
    }

    #[test]
    fn test_a_url_with_no_parseable_host_is_refused() {
        // Note what the `url` crate does here: `http:///nohost` normalizes to
        // `http://nohost/`, so the empty authority becomes a hostname rather
        // than an absent host. A genuinely host-less http URL does not parse.
        assert_eq!(
            Url::parse("http:///nohost").unwrap().as_str(),
            "http://nohost/"
        );
        assert!(Url::parse("http://").is_err());
        assert!(validate("http://", &permissive()).is_err());
        assert!(validate("not a url at all", &permissive()).is_err());
    }

    #[test]
    fn test_hardened_policy_is_selected_for_a_published_deployment() {
        let mut config = crate::config::Config::default();
        assert!(EgressPolicy::from_config(&config).allow_loopback);

        config.rest_api.public_url = Some("https://operator.example.com".to_string());
        let policy = EgressPolicy::from_config(&config);
        assert!(!policy.allow_loopback);
        assert!(!policy.allow_private);
    }

    #[test]
    fn test_check_addr_rejects_reserved_and_accepts_public() {
        assert!(check_addr(IpAddr::V4(CLOUD_METADATA_V4), &permissive()).is_err());
        assert!(check_addr(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), &permissive()).is_ok());
        assert!(check_addr(
            IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0, 0, 0, 0, 0, 1)),
            &permissive()
        )
        .is_ok());
    }

    #[test]
    fn test_validated_client_builds() {
        assert!(validated_client(EgressPolicy::hardened(), Duration::from_secs(5)).is_ok());
    }
}
