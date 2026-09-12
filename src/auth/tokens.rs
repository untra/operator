//! Access-token minting and verification (`EdDSA` / Ed25519).
//!
//! Access tokens are deliberately **not revocable individually** - checking a
//! revocation list on every request would put a database read in the hot path.
//! Their blast radius is bounded by a short expiry instead, which is why
//! [`ACCESS_TOKEN_TTL`] is 15 minutes and why anything longer-lived (sessions,
//! refresh tokens, access keys) is opaque and database-backed so it *can* be
//! revoked.

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde::{Deserialize, Serialize};

/// Length of a raw Ed25519 public key.
const ED25519_PUBLIC_KEY_LEN: usize = 32;

use crate::rest::dto::auth::Scope;

/// Lifetime of a normal access token.
pub const ACCESS_TOKEN_TTL: Duration = Duration::minutes(15);

/// Token issuer, and the audience for ordinary API access.
pub const ISSUER: &str = "operator";
/// Audience for tokens used against the REST API and MCP.
pub const AUDIENCE_API: &str = "operator-api";
/// Audience for single-purpose agent step-completion callbacks.
pub const AUDIENCE_CALLBACK: &str = "opr8r-callback";

/// Registered and Operator-specific claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Issuer.
    pub iss: String,
    /// Audience.
    pub aud: String,
    /// Subject - the account the token acts as.
    pub sub: String,
    /// Space-separated scopes, per OAuth convention.
    pub scope: String,
    /// Expiry (seconds since epoch).
    pub exp: i64,
    /// Issued at (seconds since epoch).
    pub iat: i64,
    /// Token id, so a specific token can be named in an audit record.
    pub jti: String,
    /// Ticket a callback token is pinned to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket_id: Option<String>,
    /// Step a callback token is pinned to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// Agent session a callback token is pinned to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl Claims {
    /// Parse the space-separated `scope` claim, ignoring unknown entries so a
    /// token minted by a newer build does not fail closed on an unknown scope.
    pub fn scopes(&self) -> Vec<Scope> {
        self.scope
            .split_whitespace()
            .filter_map(|s| s.parse::<Scope>().ok())
            .collect()
    }
}

/// An Ed25519 keypair used to sign and verify tokens.
///
/// Stored as DER rather than PEM because that is exactly what the underlying
/// signer wants: `jsonwebtoken`'s `EdDSA` path hands the private key straight to
/// `ring::signature::Ed25519KeyPair::from_pkcs8_maybe_unchecked`, and the
/// verifier wants the bare 32-byte public key. Keeping DER end-to-end avoids
/// hand-rolling ASN.1 to convert between the two.
pub struct SigningKey {
    /// Key id, carried in the JWT header so a rotation can still verify tokens issued under the previous key.
    pub kid: String,
    /// PKCS#8 DER of the private key.
    private_der: Vec<u8>,
    /// Raw 32-byte public key.
    public_raw: Vec<u8>,
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl std::fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print key material, even accidentally through a derived Debug.
        f.debug_struct("SigningKey")
            .field("kid", &self.kid)
            .finish_non_exhaustive()
    }
}

impl SigningKey {
    /// Generate a fresh Ed25519 keypair.
    pub fn generate(kid: impl Into<String>) -> Result<Self> {
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|_| anyhow!("generating Ed25519 keypair failed"))?;
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref())
            .map_err(|_| anyhow!("freshly generated Ed25519 keypair did not parse"))?;

        Self::from_der(
            kid,
            pkcs8.as_ref().to_vec(),
            pair.public_key().as_ref().to_vec(),
        )
    }

    /// Rebuild from the DER material stored in the auth database.
    pub fn from_der(
        kid: impl Into<String>,
        private_der: Vec<u8>,
        public_raw: Vec<u8>,
    ) -> Result<Self> {
        // Fail here rather than at first sign/verify, so a corrupt row surfaces
        // at startup instead of as a mysterious 401 later.
        Ed25519KeyPair::from_pkcs8_maybe_unchecked(&private_der)
            .map_err(|_| anyhow!("stored Ed25519 private key is not valid PKCS#8"))?;
        if public_raw.len() != ED25519_PUBLIC_KEY_LEN {
            return Err(anyhow!(
                "stored Ed25519 public key must be {ED25519_PUBLIC_KEY_LEN} bytes, got {}",
                public_raw.len()
            ));
        }

        Ok(Self {
            kid: kid.into(),
            encoding: EncodingKey::from_ed_der(&private_der),
            decoding: DecodingKey::from_ed_der(&public_raw),
            private_der,
            public_raw,
        })
    }

    /// PKCS#8 DER of the private key, for persistence. Handle as a secret.
    pub fn private_der(&self) -> &[u8] {
        &self.private_der
    }

    /// Raw public key bytes, for persistence.
    pub fn public_raw(&self) -> &[u8] {
        &self.public_raw
    }

    /// Mint a signed token.
    pub fn sign(&self, claims: &Claims) -> Result<String> {
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(self.kid.clone());
        jsonwebtoken::encode(&header, claims, &self.encoding).context("signing access token")
    }

    /// Verify a token's signature, issuer, audience, and expiry.
    ///
    /// `audience` is required rather than optional: verifying without pinning
    /// it would let a narrowly scoped agent callback token be replayed against
    /// the full REST API.
    pub fn verify(&self, token: &str, audience: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[ISSUER]);
        validation.set_audience(&[audience]);
        validation.validate_exp = true;

        jsonwebtoken::decode::<Claims>(token, &self.decoding, &validation)
            .map(|data| data.claims)
            .map_err(|e| anyhow!("token rejected: {e}"))
    }
}

/// Build the claims for an ordinary API access token.
pub fn api_claims(subject: &str, scopes: &[Scope], now: DateTime<Utc>, jti: String) -> Claims {
    Claims {
        iss: ISSUER.to_string(),
        aud: AUDIENCE_API.to_string(),
        sub: subject.to_string(),
        scope: scopes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        exp: (now + ACCESS_TOKEN_TTL).timestamp(),
        iat: now.timestamp(),
        jti,
        ticket_id: None,
        step: None,
        session_id: None,
    }
}

/// Build the claims for an agent step-completion callback token.
///
/// The lifetime is the step's, not [`ACCESS_TOKEN_TTL`]: a step may legitimately
/// run for hours, and a callback that expired mid-run would strand the agent
/// with completed work it cannot report. What bounds this token is not time but
/// its claims - it carries only `execute`, is pinned to one ticket, step, and
/// session, and is issued for the `opr8r-callback` audience, so it is useless
/// against any other route.
pub fn callback_claims(
    subject: &str,
    ticket_id: &str,
    step: &str,
    session_id: &str,
    ttl: Duration,
    now: DateTime<Utc>,
    jti: String,
) -> Claims {
    Claims {
        iss: ISSUER.to_string(),
        aud: AUDIENCE_CALLBACK.to_string(),
        sub: subject.to_string(),
        scope: Scope::Execute.as_str().to_string(),
        exp: (now + ttl).timestamp(),
        iat: now.timestamp(),
        jti,
        ticket_id: Some(ticket_id.to_string()),
        step: Some(step.to_string()),
        session_id: Some(session_id.to_string()),
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::SigningKey;

    /// A freshly generated keypair for tests. Generating beats pinning PEM
    /// literals: the test then exercises the same code path production uses to
    /// create a key on first initialization.
    pub fn key() -> SigningKey {
        SigningKey::generate("test-kid").expect("generating a test keypair")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_claim_uses_space_separated_oauth_form() {
        let claims = api_claims(
            "admin",
            &[Scope::Read, Scope::Execute],
            Utc::now(),
            "jti".to_string(),
        );
        assert_eq!(claims.scope, "read execute");
        assert_eq!(claims.scopes(), vec![Scope::Read, Scope::Execute]);
    }

    #[test]
    fn test_unknown_scopes_in_a_claim_are_ignored_not_fatal() {
        // A token minted by a newer build may name a scope this build does not
        // know. Dropping it is correct; failing the whole token is not.
        let claims = Claims {
            scope: "read future-scope execute".to_string(),
            ..api_claims("admin", &[], Utc::now(), "jti".to_string())
        };
        assert_eq!(claims.scopes(), vec![Scope::Read, Scope::Execute]);
    }

    #[test]
    fn test_api_token_expires_in_fifteen_minutes() {
        let now = Utc::now();
        let claims = api_claims("admin", &[Scope::Read], now, "jti".to_string());
        assert_eq!(claims.exp - claims.iat, 15 * 60);
    }

    #[test]
    fn test_callback_claims_are_pinned_and_execute_only() {
        let claims = callback_claims(
            "admin",
            "FEAT-1",
            "build",
            "sess-9",
            Duration::hours(24),
            Utc::now(),
            "jti".to_string(),
        );
        assert_eq!(claims.aud, AUDIENCE_CALLBACK);
        assert_eq!(claims.scope, "execute");
        assert_eq!(claims.ticket_id.as_deref(), Some("FEAT-1"));
        assert_eq!(claims.step.as_deref(), Some("build"));
        assert_eq!(claims.session_id.as_deref(), Some("sess-9"));
        assert_eq!(claims.scopes(), vec![Scope::Execute]);
    }

    #[test]
    fn test_signing_key_debug_never_prints_key_material() {
        let key = test_support::key();
        let rendered = format!("{key:?}");
        assert!(rendered.contains("test-kid"));
        assert!(!rendered.contains("PRIVATE"));
        assert!(!rendered.contains("BEGIN"));
    }

    #[test]
    fn test_round_trip_sign_and_verify() {
        let key = test_support::key();
        let claims = api_claims(
            "admin",
            &[Scope::Read, Scope::Write],
            Utc::now(),
            "jti-1".to_string(),
        );
        let token = key.sign(&claims).unwrap();

        let verified = key.verify(&token, AUDIENCE_API).unwrap();
        assert_eq!(verified.sub, "admin");
        assert_eq!(verified.jti, "jti-1");
        assert_eq!(verified.scopes(), vec![Scope::Read, Scope::Write]);
    }

    #[test]
    fn test_header_carries_kid_and_eddsa() {
        let key = test_support::key();
        let token = key
            .sign(&api_claims("admin", &[], Utc::now(), "j".to_string()))
            .unwrap();
        let header = jsonwebtoken::decode_header(&token).unwrap();
        assert_eq!(header.alg, Algorithm::EdDSA);
        assert_eq!(header.kid.as_deref(), Some("test-kid"));
    }

    #[test]
    fn test_callback_token_is_rejected_against_the_api_audience() {
        // The whole point of pinning `aud`: a narrowly scoped callback token
        // must not be replayable against the full REST API.
        let key = test_support::key();
        let token = key
            .sign(&callback_claims(
                "admin",
                "FEAT-1",
                "build",
                "sess",
                Duration::hours(1),
                Utc::now(),
                "j".to_string(),
            ))
            .unwrap();

        assert!(key.verify(&token, AUDIENCE_CALLBACK).is_ok());
        assert!(
            key.verify(&token, AUDIENCE_API).is_err(),
            "a callback token must not authenticate ordinary API requests"
        );
    }

    #[test]
    fn test_expired_token_is_rejected() {
        let key = test_support::key();
        let past = Utc::now() - Duration::hours(2);
        let token = key
            .sign(&api_claims("admin", &[Scope::Read], past, "j".to_string()))
            .unwrap();
        assert!(key.verify(&token, AUDIENCE_API).is_err());
    }

    #[test]
    fn test_tampered_token_fails_signature_verification() {
        let key = test_support::key();
        let token = key
            .sign(&api_claims(
                "admin",
                &[Scope::Read],
                Utc::now(),
                "j".to_string(),
            ))
            .unwrap();

        // Flip a character in the payload segment.
        let mut parts: Vec<&str> = token.split('.').collect();
        let payload = parts[1].to_string();
        let mutated = match payload.strip_prefix('e') {
            Some(rest) => format!("f{rest}"),
            None => format!("e{}", &payload[1..]),
        };
        parts[1] = &mutated;
        let tampered = parts.join(".");

        assert!(key.verify(&tampered, AUDIENCE_API).is_err());
    }

    #[test]
    fn test_key_survives_a_persistence_round_trip() {
        // The database stores DER; a token signed before a restart must still
        // verify after one, or every issued credential dies on restart.
        let original = SigningKey::generate("kid-1").unwrap();
        let token = original
            .sign(&api_claims(
                "admin",
                &[Scope::Admin],
                Utc::now(),
                "j".to_string(),
            ))
            .unwrap();

        let reloaded = SigningKey::from_der(
            "kid-1",
            original.private_der().to_vec(),
            original.public_raw().to_vec(),
        )
        .unwrap();

        let claims = reloaded.verify(&token, AUDIENCE_API).unwrap();
        assert_eq!(claims.sub, "admin");
    }

    #[test]
    fn test_each_generated_key_is_distinct() {
        let a = SigningKey::generate("a").unwrap();
        let b = SigningKey::generate("b").unwrap();
        assert_ne!(a.public_raw(), b.public_raw());
        assert_ne!(a.private_der(), b.private_der());
    }

    #[test]
    fn test_a_token_does_not_verify_under_a_different_key() {
        // Re-bootstrapping generates a new key, which must invalidate every
        // token issued under the old one.
        let old_key = SigningKey::generate("old").unwrap();
        let new_key = SigningKey::generate("new").unwrap();
        let token = old_key
            .sign(&api_claims(
                "admin",
                &[Scope::Read],
                Utc::now(),
                "j".to_string(),
            ))
            .unwrap();

        assert!(old_key.verify(&token, AUDIENCE_API).is_ok());
        assert!(new_key.verify(&token, AUDIENCE_API).is_err());
    }

    #[test]
    fn test_corrupt_stored_key_material_is_rejected_at_load() {
        // Surfacing this at startup beats a mysterious 401 on first request.
        assert!(SigningKey::from_der("kid", vec![0u8; 16], vec![0u8; 32]).is_err());

        let good = SigningKey::generate("kid").unwrap();
        assert!(
            SigningKey::from_der("kid", good.private_der().to_vec(), vec![0u8; 31]).is_err(),
            "a public key of the wrong length must be rejected"
        );
    }

    #[test]
    fn test_token_from_a_different_issuer_is_rejected() {
        let key = test_support::key();
        let mut claims = api_claims("admin", &[Scope::Read], Utc::now(), "j".to_string());
        claims.iss = "somebody-else".to_string();
        let token = key.sign(&claims).unwrap();
        assert!(key.verify(&token, AUDIENCE_API).is_err());
    }
}
