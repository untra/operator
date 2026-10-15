use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{ensure, Context, Result};
use openidconnect::core::{CoreClient, CoreProviderMetadata, CoreResponseType};
use openidconnect::OAuth2TokenResponse as _;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl,
};
use url::Url;

use super::identity::OidcConnection;
use super::secret::hash_secret;
use super::store::AuthStore;

const LOGIN_TTL: Duration = Duration::from_mins(5);
const MAX_PENDING_LOGINS: usize = 1000;
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

pub struct PendingLogin {
    connection: OidcConnection,
    metadata: CoreProviderMetadata,
    verifier: PkceCodeVerifier,
    nonce: Nonce,
    cookie_hash: String,
    created: Instant,
}

#[derive(Default)]
pub struct LoginAttempts(Mutex<HashMap<String, PendingLogin>>);

impl LoginAttempts {
    fn insert(&self, state: String, login: PendingLogin) -> Result<()> {
        let mut attempts = self
            .0
            .lock()
            .map_err(|_| anyhow::anyhow!("login state unavailable"))?;
        attempts.retain(|_, p| p.created.elapsed() < LOGIN_TTL);
        ensure!(
            attempts.len() < MAX_PENDING_LOGINS,
            "too many pending logins"
        );
        attempts.insert(state, login);
        Ok(())
    }

    fn take(&self, state: &str, cookie: &str) -> Result<PendingLogin> {
        let mut attempts = self
            .0
            .lock()
            .map_err(|_| anyhow::anyhow!("login state unavailable"))?;
        let login = attempts.get(state).context("invalid login state")?;
        ensure!(
            login.created.elapsed() < LOGIN_TTL && login.cookie_hash == hash_secret(cookie),
            "invalid login state"
        );
        attempts.remove(state).context("invalid login state")
    }
}

pub fn validate_url(raw: &str) -> Result<Url> {
    let url = Url::parse(raw)?;
    ensure!(
        url.username().is_empty() && url.password().is_none() && url.fragment().is_none(),
        "invalid OIDC URL"
    );
    ensure!(url.scheme() == "https", "OIDC requires HTTPS");
    ensure!(url.host_str().is_some(), "OIDC requires a host");
    Ok(url)
}

async fn request(
    issuer: &Url,
    request: openidconnect::HttpRequest,
) -> Result<openidconnect::HttpResponse> {
    let url = validate_url(&request.uri().to_string())?;
    ensure!(
        url.origin() == issuer.origin(),
        "OIDC endpoint must use the configured issuer origin"
    );
    let host = url.host_str().context("missing OIDC host")?;
    let addresses: Vec<_> =
        tokio::net::lookup_host((host, url.port_or_known_default().context("missing port")?))
            .await?
            .collect();
    ensure!(!addresses.is_empty(), "OIDC host has no addresses");
    let policy = super::egress::EgressPolicy {
        allow_loopback: false,
        allow_private: true,
    };
    for address in &addresses {
        super::egress::check_addr(address.ip(), &policy)?;
    }
    let client = crate::http_client::client_builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .timeout(REQUEST_TIMEOUT)
        .resolve_to_addrs(host, &addresses)
        .build()?;
    let (parts, body) = request.into_parts();
    let mut response = client
        .request(parts.method, url)
        .headers(parts.headers)
        .body(body)
        .send()
        .await?;
    let mut output = openidconnect::http::Response::builder().status(response.status());
    *output
        .headers_mut()
        .context("response headers unavailable")? = response.headers().clone();
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            body.len() + chunk.len() <= MAX_RESPONSE_BYTES,
            "OIDC response too large"
        );
        body.extend_from_slice(&chunk);
    }
    Ok(output.body(body)?)
}

struct OidcHttp(Url);

impl<'c> openidconnect::AsyncHttpClient<'c> for OidcHttp {
    type Error = std::io::Error;
    type Future = std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = std::result::Result<openidconnect::HttpResponse, Self::Error>,
                > + Send
                + 'c,
        >,
    >;
    fn call(&'c self, req: openidconnect::HttpRequest) -> Self::Future {
        let issuer = self.0.clone();
        Box::pin(async move { request(&issuer, req).await.map_err(std::io::Error::other) })
    }
}

fn secret(store: &AuthStore, id: &str) -> Result<Option<ClientSecret>> {
    store
        .oidc_secret_file(id)?
        .map(|path| {
            let value = std::fs::read_to_string(path)?;
            ensure!(!value.trim().is_empty(), "empty OIDC client secret");
            Ok(ClientSecret::new(value.trim().to_string()))
        })
        .transpose()
}

pub async fn start(store: &AuthStore, connection: OidcConnection, cookie: &str) -> Result<String> {
    let issuer = validate_url(&connection.issuer)?;
    let http = OidcHttp(issuer);
    let metadata =
        CoreProviderMetadata::discover_async(IssuerUrl::new(connection.issuer.clone())?, &http)
            .await?;
    let client = CoreClient::from_provider_metadata(
        metadata.clone(),
        ClientId::new(connection.client_id.clone()),
        secret(store, &connection.id)?,
    )
    .set_redirect_uri(RedirectUrl::new(connection.redirect_uri.clone())?);
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let (url, state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .set_pkce_challenge(challenge)
        .add_extra_param("max_age", "0")
        .url();
    store.logins.insert(
        hash_secret(state.secret()),
        PendingLogin {
            connection,
            metadata,
            verifier,
            nonce,
            cookie_hash: hash_secret(cookie),
            created: Instant::now(),
        },
    )?;
    Ok(url.to_string())
}

pub async fn complete(
    store: &AuthStore,
    state: &str,
    cookie: &str,
    code: String,
) -> Result<(String, String, chrono::DateTime<chrono::Utc>)> {
    let login = store.logins.take(&hash_secret(state), cookie)?;
    let issuer = validate_url(&login.connection.issuer)?;
    let http = OidcHttp(issuer);
    let client = CoreClient::from_provider_metadata(
        login.metadata,
        ClientId::new(login.connection.client_id.clone()),
        secret(store, &login.connection.id)?,
    )
    .set_redirect_uri(RedirectUrl::new(login.connection.redirect_uri.clone())?);
    let tokens = client
        .exchange_code(AuthorizationCode::new(code))?
        .set_pkce_verifier(login.verifier)
        .request_async(&http)
        .await?;
    let id_token = tokens
        .extra_fields()
        .id_token()
        .context("missing ID token")?;
    let verifier = client.id_token_verifier();
    let claims = id_token.claims(&verifier, &login.nonce)?;
    if let Some(expected) = claims.access_token_hash() {
        let actual = openidconnect::AccessTokenHash::from_token(
            tokens.access_token(),
            id_token.signing_alg()?,
            id_token.signing_key(&verifier)?,
        )?;
        ensure!(actual == *expected, "invalid access token hash");
    }
    let authenticated = claims
        .auth_time()
        .context("provider did not return auth_time")?;
    let now = chrono::Utc::now();
    ensure!(
        authenticated <= now + chrono::Duration::seconds(30)
            && authenticated
                > now - chrono::Duration::seconds(super::identity::RECENT_LOGIN_SECONDS),
        "recent login required"
    );
    let id = store.provision_oidc_user(&login.connection, claims.subject().as_str())?;
    store.oidc_session(&id, authenticated.timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_urls_reject_credentials_plaintext_and_fragments() {
        for url in [
            "http://id.example",
            "https://user:secret@id.example",
            "https://id.example/#secret",
            "file:///etc/passwd",
        ] {
            assert!(validate_url(url).is_err());
        }
        assert!(validate_url("https://id.example/realms/operator").is_ok());
    }
}
