//! OIDC provider access tokens accepted as bearer credentials.
//!
//! Operator acts as an OAuth protected resource (RFC 9728): clients discover the
//! provider from the published metadata, obtain a token there, and present it
//! here. Only users already provisioned and approved through browser sign-in
//! are resolved; this path never provisions.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{ensure, Context, Result};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

use super::identity::OidcConnection;
use super::scope::Principal;
use super::store::AuthStore;
use crate::rest::dto::auth::{PrincipalKind, ProtectedResourceMetadata};

pub const METADATA_PATH: &str = "/.well-known/oauth-protected-resource";
const ALGORITHMS: [Algorithm; 2] = [Algorithm::RS256, Algorithm::ES256];
const JWKS_TTL: Duration = Duration::from_mins(10);
const JWKS_MIN_REFRESH: Duration = Duration::from_mins(1);
const LEEWAY_SECONDS: u64 = 30;

#[derive(Debug, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub exp: i64,
}

#[derive(Deserialize)]
struct UnverifiedIssuer {
    iss: String,
}

#[derive(Deserialize)]
struct Discovery {
    issuer: String,
    jwks_uri: String,
}

fn normalize(url: &str) -> &str {
    url.trim_end_matches('/')
}

fn both_forms(url: &str) -> [String; 2] {
    let base = normalize(url);
    [base.to_string(), format!("{base}/")]
}

/// Enabled connections that may issue bearer tokens, empty without a public URL.
pub fn active_connections(public: Option<&str>, conns: Vec<OidcConnection>) -> Vec<OidcConnection> {
    if public.is_none() {
        return Vec::new();
    }
    conns.into_iter().filter(|c| !c.disabled).collect()
}

pub fn metadata_url(public: &str) -> String {
    format!("{}{METADATA_PATH}", normalize(public))
}

pub fn protected_resource_metadata(
    public: &str,
    conns: &[OidcConnection],
) -> Option<ProtectedResourceMetadata> {
    (!conns.is_empty()).then(|| ProtectedResourceMetadata {
        resource: normalize(public).to_string(),
        authorization_servers: conns.iter().map(|c| c.issuer.clone()).collect(),
        bearer_methods_supported: vec!["header".into()],
        resource_name: "Operator".into(),
    })
}

/// The `WWW-Authenticate` challenge, naming the metadata document when discovery is active.
pub fn challenge(metadata_url: Option<&str>) -> String {
    match metadata_url {
        Some(url) => {
            format!(
                r#"Bearer realm="operator", resource_metadata="{url}", Cookie realm="operator""#
            )
        }
        None => r#"Bearer realm="operator", Cookie realm="operator""#.into(),
    }
}

/// The `iss` claim, read before the signature is checked to select a provider.
pub fn unverified_issuer(token: &str) -> Option<String> {
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;
    validation.validate_aud = false;
    validation.required_spec_claims.clear();
    jsonwebtoken::decode::<UnverifiedIssuer>(token, &DecodingKey::from_secret(&[]), &validation)
        .ok()
        .map(|data| data.claims.iss)
}

fn key_id(token: &str) -> Option<String> {
    jsonwebtoken::decode_header(token).ok()?.kid
}

/// Verify signature, issuer, audience, and expiry against a provider key set.
pub fn verify(token: &str, keys: &JwkSet, issuer: &str, audience: &str) -> Result<AccessClaims> {
    let header = jsonwebtoken::decode_header(token)?;
    ensure!(
        ALGORITHMS.contains(&header.alg),
        "unsupported token algorithm"
    );
    let kid = header.kid.context("token has no key id")?;
    let jwk = keys.find(&kid).context("unknown signing key")?;
    let key = DecodingKey::from_jwk(jwk)?;
    let mut validation = Validation::new(header.alg);
    validation.set_issuer(&both_forms(issuer));
    validation.set_audience(&both_forms(audience));
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
    validation.leeway = LEEWAY_SECONDS;
    Ok(jsonwebtoken::decode::<AccessClaims>(token, &key, &validation)?.claims)
}

async fn get_json<T: serde::de::DeserializeOwned>(issuer: &url::Url, url: &str) -> Result<T> {
    let request = openidconnect::http::Request::builder()
        .method(openidconnect::http::Method::GET)
        .uri(url)
        .header(openidconnect::http::header::ACCEPT, "application/json")
        .body(Vec::new())?;
    let response = super::oidc::request(issuer, request).await?;
    ensure!(
        response.status().is_success(),
        "OIDC provider returned {}",
        response.status()
    );
    Ok(serde_json::from_slice(response.body())?)
}

async fn fetch_keys(issuer: &str) -> Result<JwkSet> {
    let origin = super::oidc::validate_url(issuer)?;
    let discovery: Discovery = get_json(
        &origin,
        &format!("{}/.well-known/openid-configuration", normalize(issuer)),
    )
    .await?;
    ensure!(
        normalize(&discovery.issuer) == normalize(issuer),
        "OIDC discovery issuer mismatch"
    );
    get_json(&origin, &discovery.jwks_uri).await
}

/// Provider key sets, refreshed on expiry or an unknown key id at most once a minute.
#[derive(Default)]
pub struct JwksCache(Mutex<HashMap<String, (Instant, JwkSet)>>);

impl JwksCache {
    fn cached(&self, issuer: &str, kid: &str) -> Option<JwkSet> {
        let entries = self.0.lock().ok()?;
        let (fetched, keys) = entries.get(issuer)?;
        let age = fetched.elapsed();
        let usable = (age < JWKS_TTL && keys.find(kid).is_some()) || age < JWKS_MIN_REFRESH;
        usable.then(|| keys.clone())
    }

    async fn keys(&self, issuer: &str, kid: &str) -> Result<JwkSet> {
        if let Some(keys) = self.cached(issuer, kid) {
            return Ok(keys);
        }
        let keys = fetch_keys(issuer).await?;
        if let Ok(mut entries) = self.0.lock() {
            entries.insert(issuer.to_string(), (Instant::now(), keys.clone()));
        }
        Ok(keys)
    }
}

/// Resolve a provider access token to an approved user's principal.
///
/// `Ok(None)` for any token that does not verify; `Err` only when the store or provider is unreachable.
pub async fn authenticate(
    store: &AuthStore,
    public: Option<String>,
    profile: Uuid,
    token: &str,
) -> Result<Option<Principal>> {
    let Some(public) = public else {
        return Ok(None);
    };
    let (Some(issuer), Some(kid)) = (unverified_issuer(token), key_id(token)) else {
        return Ok(None);
    };
    let lookup = store.clone();
    let conns = tokio::task::spawn_blocking(move || lookup.oidc_connections()).await??;
    let Some(connection) = active_connections(Some(&public), conns)
        .into_iter()
        .find(|c| normalize(&c.issuer) == normalize(&issuer))
    else {
        return Ok(None);
    };
    let keys = store.jwks.keys(&connection.issuer, &kid).await?;
    let Ok(claims) = verify(token, &keys, &connection.issuer, &public) else {
        return Ok(None);
    };
    let Some(expires) = chrono::DateTime::from_timestamp(claims.exp, 0) else {
        return Ok(None);
    };
    let lookup = store.clone();
    let authority = tokio::task::spawn_blocking(move || {
        lookup.subject_authority(&connection, &claims.sub, profile)
    })
    .await??;
    Ok(authority.map(|authority| Principal {
        kind: PrincipalKind::OidcToken,
        ..authority.principal(expires, None)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;
    use jsonwebtoken::{EncodingKey, Header};
    use ring::rand::SystemRandom;
    use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};

    const ISSUER: &str = "https://identity.example/realms/operator";
    const AUDIENCE: &str = "https://operator.example";

    fn connection(issuer: &str, disabled: bool) -> OidcConnection {
        OidcConnection {
            id: issuer.into(),
            provider: crate::auth::identity::OidcProvider::Keycloak,
            issuer: issuer.into(),
            client_id: "operator".into(),
            redirect_uri: format!("{AUDIENCE}/api/v1/auth/oidc/callback"),
            disabled,
        }
    }

    struct Signer {
        encoding: EncodingKey,
        keys: JwkSet,
    }

    fn make_signer(kid: &str) -> Signer {
        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
        let pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref(), &rng)
            .unwrap();
        let point = pair.public_key().as_ref();
        let keys = serde_json::from_value(serde_json::json!({"keys": [{
            "kty": "EC", "crv": "P-256", "kid": kid, "alg": "ES256", "use": "sig",
            "x": URL_SAFE_NO_PAD.encode(&point[1..33]),
            "y": URL_SAFE_NO_PAD.encode(&point[33..65]),
        }]}))
        .unwrap();
        Signer {
            encoding: EncodingKey::from_ec_der(pkcs8.as_ref()),
            keys,
        }
    }

    fn token(signer: &Signer, kid: &str, claims: serde_json::Value) -> String {
        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(kid.into());
        jsonwebtoken::encode(&header, &claims, &signer.encoding).unwrap()
    }

    fn claims(aud: &str, exp_offset: i64) -> serde_json::Value {
        serde_json::json!({
            "iss": ISSUER, "sub": "subject", "aud": [aud, "account"],
            "exp": chrono::Utc::now().timestamp() + exp_offset,
        })
    }

    #[test]
    fn test_metadata_lists_enabled_issuers_only_with_public_url() {
        let conns = vec![connection(ISSUER, false), connection("https://other", true)];
        assert!(active_connections(None, conns.clone()).is_empty());
        let active = active_connections(Some(AUDIENCE), conns);
        let metadata = protected_resource_metadata(&format!("{AUDIENCE}/"), &active).unwrap();
        assert_eq!(metadata.resource, AUDIENCE);
        assert_eq!(metadata.authorization_servers, vec![ISSUER.to_string()]);
        assert!(protected_resource_metadata(AUDIENCE, &[]).is_none());
    }

    #[test]
    fn test_challenge_names_metadata_only_when_active() {
        let url = metadata_url(&format!("{AUDIENCE}/"));
        assert_eq!(url, format!("{AUDIENCE}{METADATA_PATH}"));
        assert!(challenge(Some(&url)).contains(&format!(r#"resource_metadata="{url}""#)));
        assert_eq!(
            challenge(None),
            r#"Bearer realm="operator", Cookie realm="operator""#
        );
    }

    #[test]
    fn test_verify_accepts_a_valid_provider_token() {
        let signer = make_signer("k1");
        let jwt = token(&signer, "k1", claims(AUDIENCE, 300));
        assert_eq!(unverified_issuer(&jwt).as_deref(), Some(ISSUER));
        let verified = verify(&jwt, &signer.keys, &format!("{ISSUER}/"), AUDIENCE).unwrap();
        assert_eq!(verified.sub, "subject");
    }

    #[test]
    fn test_verify_rejects_wrong_audience_issuer_expiry_and_key() {
        let signer = make_signer("k1");
        let other = make_signer("k1");
        assert!(verify(
            &token(&signer, "k1", claims("https://elsewhere", 300)),
            &signer.keys,
            ISSUER,
            AUDIENCE
        )
        .is_err());
        assert!(verify(
            &token(&signer, "k1", claims(AUDIENCE, 300)),
            &signer.keys,
            "https://identity.example/realms/other",
            AUDIENCE
        )
        .is_err());
        assert!(verify(
            &token(&signer, "k1", claims(AUDIENCE, -600)),
            &signer.keys,
            ISSUER,
            AUDIENCE
        )
        .is_err());
        assert!(verify(
            &token(&signer, "k2", claims(AUDIENCE, 300)),
            &signer.keys,
            ISSUER,
            AUDIENCE
        )
        .is_err());
        assert!(verify(
            &token(&other, "k1", claims(AUDIENCE, 300)),
            &signer.keys,
            ISSUER,
            AUDIENCE
        )
        .is_err());
    }

    #[test]
    fn test_verify_rejects_symmetric_and_unsigned_tokens() {
        let signer = make_signer("k1");
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("k1".into());
        let hs = jsonwebtoken::encode(
            &header,
            &claims(AUDIENCE, 300),
            &EncodingKey::from_secret(b"secret"),
        )
        .unwrap();
        assert!(verify(&hs, &signer.keys, ISSUER, AUDIENCE).is_err());
        let valid = token(&signer, "k1", claims(AUDIENCE, 300));
        let payload = valid.split('.').nth(1).unwrap();
        let unsigned = format!(
            "{}.{payload}.",
            URL_SAFE_NO_PAD.encode(r#"{"alg":"none","kid":"k1"}"#)
        );
        assert!(verify(&unsigned, &signer.keys, ISSUER, AUDIENCE).is_err());
    }

    #[tokio::test]
    async fn test_authenticate_resolves_an_approved_user_from_cached_keys() {
        let (store, owner, request) =
            crate::auth::identity::tests::fixture_in(AuthStore::in_memory().unwrap());
        let issuer = store.oidc_connections().unwrap().remove(0).issuer;
        let signer = make_signer("k1");
        store
            .jwks
            .0
            .lock()
            .unwrap()
            .insert(issuer.clone(), (Instant::now(), signer.keys.clone()));
        let mut good = claims(AUDIENCE, 300);
        good["iss"] = issuer.clone().into();
        let jwt = token(&signer, "k1", good.clone());
        let profile = request.grants.profile_id;

        let principal = authenticate(&store, Some(AUDIENCE.into()), profile, &jwt)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(principal.kind, PrincipalKind::OidcToken);
        assert_eq!(principal.subject, owner.user.id);
        assert!(authenticate(&store, None, profile, &jwt)
            .await
            .unwrap()
            .is_none());

        let mut stranger = good;
        stranger["sub"] = "stranger".into();
        let jwt = token(&signer, "k1", stranger);
        assert!(authenticate(&store, Some(AUDIENCE.into()), profile, &jwt)
            .await
            .unwrap()
            .is_none());
    }
}
