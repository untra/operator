use std::collections::BTreeMap;
use std::io::Write;
use std::sync::Mutex;

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::config::{Config, TargetDef, TargetKind};

pub const PREMIUM_TIER: &str = "premium";
pub const LICENSE_AUDIENCE: &str = "operator-license";
pub const LICENSE_VERSION: u32 = 1;
const LICENSE_FILE: &str = "license.key";
const MAX_LICENSE_BYTES: usize = 32 * 1024;
const PUBLIC_KEY_BYTES: usize = 32;
static LICENSE_UPDATE: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum PremiumFeature {
    RemoteTargets,
}

impl std::fmt::Display for PremiumFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RemoteTargets => f.write_str("remote_targets"),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{feature} requires a valid Premium license for this configuration")]
pub struct NotEntitled {
    pub feature: PremiumFeature,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct LicenseTerms {
    pub version: u32,
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub jti: String,
    pub profile_id: Uuid,
    pub tier: String,
    pub iat: i64,
    pub nbf: i64,
    pub exp: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum LicenseStatus {
    Missing,
    Valid,
    Expired,
    NotYetValid,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct LicenseResponse {
    pub status: LicenseStatus,
    pub profile_id: Uuid,
    pub premium: bool,
    pub terms: Option<LicenseTerms>,
    pub purchase_url: Option<String>,
}

impl LicenseResponse {
    /// The free tier, without reading the filesystem. For constructing
    /// fixtures and defaults; real answers come from [`status`].
    #[allow(dead_code)] // Used from test fixtures across both crate targets
    pub fn free(profile_id: Uuid) -> Self {
        Self {
            status: LicenseStatus::Missing,
            profile_id,
            premium: false,
            terms: None,
            purchase_url: None,
        }
    }
}

pub struct Verifier {
    keys: BTreeMap<String, String>,
    issuer: String,
}

impl Verifier {
    /// A verifier over an explicit key set. Enforcement always goes through
    /// [`Verifier::bundled`]; this exists so tests and issuing tools can verify
    /// against a key that is not compiled in.
    #[allow(dead_code)] // Verification seam: used from tests, not the binary
    pub fn from_keys(keys: BTreeMap<String, String>, issuer: String) -> Self {
        Self { keys, issuer }
    }

    pub fn bundled() -> Result<Self> {
        Ok(Self {
            keys: serde_json::from_str(option_env!("OPERATOR_LICENSE_PUBLIC_KEYS").unwrap_or("{}"))
                .context("invalid bundled license verification keys")?,
            issuer: option_env!("OPERATOR_LICENSE_ISSUER")
                .unwrap_or("operator-licensing")
                .to_owned(),
        })
    }

    fn cache_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.issuer.hash(&mut hasher);
        for (key_id, key) in &self.keys {
            key_id.hash(&mut hasher);
            key.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Signature and claim checks. Deliberately time-independent so the result
    /// can be memoised; validity against the clock is [`status_for`].
    fn decode(&self, key: &str, profile_id: Uuid) -> Result<LicenseTerms> {
        anyhow::ensure!(
            !profile_id.is_nil(),
            "configuration identity has not been initialized"
        );
        anyhow::ensure!(key.len() <= MAX_LICENSE_BYTES, "license is too large");
        let bytes = STANDARD
            .decode(key.trim())
            .context("license must be Base64 encoded")?;
        let token = std::str::from_utf8(&bytes).context("license is not a JWT")?;
        let header = jsonwebtoken::decode_header(token).context("invalid license JWT")?;
        anyhow::ensure!(
            header.alg == Algorithm::EdDSA,
            "unsupported license algorithm"
        );
        let encoded = header
            .kid
            .as_ref()
            .and_then(|kid| self.keys.get(kid))
            .context("unknown license signing key")?;
        let public = STANDARD
            .decode(encoded)
            .context("invalid license verification key")?;
        anyhow::ensure!(
            public.len() == PUBLIC_KEY_BYTES,
            "invalid license verification key"
        );
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[LICENSE_AUDIENCE]);
        validation.set_required_spec_claims(&["exp", "iat", "nbf", "iss", "aud", "sub"]);
        validation.validate_exp = false;
        validation.validate_nbf = false;
        let terms = jsonwebtoken::decode::<LicenseTerms>(
            token,
            &DecodingKey::from_ed_der(&public),
            &validation,
        )
        .context("license signature or claims rejected")?
        .claims;
        anyhow::ensure!(
            terms.version == LICENSE_VERSION,
            "unsupported license version"
        );
        anyhow::ensure!(
            terms.profile_id == profile_id,
            "license belongs to another configuration"
        );
        anyhow::ensure!(terms.tier == PREMIUM_TIER, "unsupported license tier");
        anyhow::ensure!(
            !terms.sub.trim().is_empty() && !terms.jti.trim().is_empty(),
            "missing license identity"
        );
        anyhow::ensure!(
            terms.iat >= 0 && terms.nbf >= terms.iat && terms.exp > terms.nbf,
            "invalid license validity interval"
        );
        Ok(terms)
    }

    fn verify(
        &self,
        key: &str,
        profile_id: Uuid,
        now: i64,
    ) -> Result<(LicenseStatus, LicenseTerms)> {
        let terms = self.decode(key, profile_id)?;
        Ok((status_for(&terms, now), terms))
    }
}

/// Where `now` falls relative to the licence's validity interval.
fn status_for(terms: &LicenseTerms, now: i64) -> LicenseStatus {
    if now >= terms.exp {
        LicenseStatus::Expired
    } else if now < terms.nbf || now < terms.iat {
        LicenseStatus::NotYetValid
    } else {
        LicenseStatus::Valid
    }
}

/// What the selected configuration is allowed to do.
///
/// Derived from the licence on every read, so an expiry that passes while the
/// process runs takes effect without a file change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entitlements {
    pub status: LicenseStatus,
    pub premium: bool,
}

impl Entitlements {
    pub fn allows(&self, feature: PremiumFeature) -> bool {
        match feature {
            PremiumFeature::RemoteTargets => self.premium,
        }
    }
}

/// Which configuration's licence a cache entry belongs to. One process can
/// host several configurations, and a licence is pinned to exactly one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    path: std::path::PathBuf,
    profile: Uuid,
    verifier: u64,
}

/// The licence bytes an entry was verified from. A rewrite that changes neither
/// is indistinguishable, which is why writes call [`invalidate`] explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileStamp {
    modified: Option<std::time::SystemTime>,
    len: u64,
}

/// The verification outcome, without the time-dependent part.
#[derive(Debug, Clone)]
enum Verified {
    Terms(Box<LicenseTerms>),
    Rejected,
}

type Cache = std::collections::HashMap<CacheKey, (FileStamp, Verified)>;

static CACHE: std::sync::RwLock<Option<Cache>> = std::sync::RwLock::new(None);

/// Decodes performed rather than served from cache; the memoisation is not
/// otherwise observable.
#[cfg(test)]
static DECODES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn cached(key: &CacheKey, stamp: &FileStamp) -> Option<Verified> {
    let guard = CACHE.read().ok()?;
    guard
        .as_ref()?
        .get(key)
        .and_then(|(cached, verified)| (cached == stamp).then(|| verified.clone()))
}

fn store(key: CacheKey, stamp: FileStamp, verified: &Verified) {
    if let Ok(mut guard) = CACHE.write() {
        guard
            .get_or_insert_with(Cache::default)
            .insert(key, (stamp, verified.clone()));
    }
}

/// Drop every memoised verification. Called whenever a licence file is written
/// or removed, because filesystem timestamps are too coarse to rely on for a
/// rewrite that lands in the same tick.
fn invalidate() {
    if let Ok(mut guard) = CACHE.write() {
        *guard = None;
    }
}

/// Entitlements for `config`, using the bundled verification keys.
pub fn entitlements(config: &Config) -> Entitlements {
    let response = status(config);
    Entitlements {
        status: response.status,
        premium: response.premium,
    }
}

#[allow(dead_code)] // Used by `entitlements_with`, which the binary never calls
fn entitlements_from(response: &LicenseResponse) -> Entitlements {
    Entitlements {
        status: response.status,
        premium: response.premium,
    }
}

pub fn status(config: &Config) -> LicenseResponse {
    match Verifier::bundled() {
        Ok(verifier) => status_with(config, &verifier, chrono::Utc::now().timestamp()),
        // No bundled keys means no licence can ever verify; report that as
        // invalid rather than pretending the file is absent.
        Err(_) => rejected(config),
    }
}

/// [`status`] against an explicit verifier and clock. The verification tests and issuing tools; enforcement always uses [`status`].
pub fn status_with(config: &Config, verifier: &Verifier, now: i64) -> LicenseResponse {
    let path = config.state_path().join(LICENSE_FILE);
    let metadata = match std::fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return missing(config),
        Err(_) => return rejected(config),
    };
    let cache_key = CacheKey {
        path: path.clone(),
        profile: config.profile.id,
        verifier: verifier.cache_key(),
    };
    let stamp = FileStamp {
        modified: metadata.modified().ok(),
        len: metadata.len(),
    };

    let verified = match cached(&cache_key, &stamp) {
        Some(verified) => verified,
        None => {
            #[cfg(test)]
            DECODES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let verified = match std::fs::read_to_string(&path) {
                Ok(key) => match verifier.decode(&key, config.profile.id) {
                    Ok(terms) => Verified::Terms(Box::new(terms)),
                    Err(_) => Verified::Rejected,
                },
                Err(_) => Verified::Rejected,
            };
            store(cache_key, stamp, &verified);
            verified
        }
    };

    match verified {
        Verified::Rejected => rejected(config),
        Verified::Terms(terms) => {
            let status = status_for(&terms, now);
            LicenseResponse {
                status,
                profile_id: config.profile.id,
                premium: status == LicenseStatus::Valid,
                terms: Some(*terms),
                purchase_url: purchase_url(),
            }
        }
    }
}

/// Entitlements against an explicit verifier and clock.
#[allow(dead_code)] // Verification seam: used from tests, not the binary
pub fn entitlements_with(config: &Config, verifier: &Verifier, now: i64) -> Entitlements {
    entitlements_from(&status_with(config, verifier, now))
}

fn purchase_url() -> Option<String> {
    option_env!("OPERATOR_PURCHASE_URL")
        .filter(|url| !url.is_empty())
        .map(str::to_owned)
}

fn missing(config: &Config) -> LicenseResponse {
    LicenseResponse {
        status: LicenseStatus::Missing,
        profile_id: config.profile.id,
        premium: false,
        terms: None,
        purchase_url: purchase_url(),
    }
}

fn rejected(config: &Config) -> LicenseResponse {
    LicenseResponse {
        status: LicenseStatus::Invalid,
        ..missing(config)
    }
}

pub fn install(config: &Config, key: &str) -> Result<LicenseResponse> {
    install_with(
        config,
        &Verifier::bundled()?,
        chrono::Utc::now().timestamp(),
        key,
    )
}

/// [`install`] against an explicit verifier and clock.
///
/// The replacement is verified before anything is written, so a rejected key
/// leaves the installed licence untouched.
pub fn install_with(
    config: &Config,
    verifier: &Verifier,
    now: i64,
    key: &str,
) -> Result<LicenseResponse> {
    let _guard = LICENSE_UPDATE
        .lock()
        .map_err(|_| anyhow::anyhow!("license update lock unavailable"))?;
    let (status, _) = verifier.verify(key, config.profile.id, now)?;
    anyhow::ensure!(
        status == LicenseStatus::Valid,
        "license is not currently valid"
    );
    persist(config, key.trim())?;
    invalidate();
    Ok(status_with(config, verifier, now))
}

fn persist(config: &Config, key: &str) -> Result<()> {
    let directory = config.state_path();
    std::fs::create_dir_all(&directory)?;
    let temporary = directory.join(format!(".license-{}.tmp", Uuid::new_v4()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> Result<()> {
        let mut file = options.open(&temporary)?;
        file.write_all(key.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&temporary, directory.join(LICENSE_FILE))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

pub fn remove(config: &Config) -> Result<LicenseResponse> {
    let _guard = LICENSE_UPDATE
        .lock()
        .map_err(|_| anyhow::anyhow!("license update lock unavailable"))?;
    match std::fs::remove_file(config.state_path().join(LICENSE_FILE)) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    invalidate();
    Ok(status(config))
}

pub fn require_premium(config: &Config, feature: PremiumFeature) -> Result<(), NotEntitled> {
    if entitlements(config).allows(feature) {
        Ok(())
    } else {
        Err(NotEntitled { feature })
    }
}

pub fn require_target(config: &Config, target: &TargetDef) -> Result<(), NotEntitled> {
    match target.kind {
        TargetKind::Ssh(_) | TargetKind::Coder(_) => {
            require_premium(config, PremiumFeature::RemoteTargets)
        }
        TargetKind::Local | TargetKind::Docker(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The verification cache and its decode counter are process-wide, so the
    /// tests that observe them run one at a time.
    static SERIAL: Mutex<()> = Mutex::new(());

    fn serial() -> std::sync::MutexGuard<'static, ()> {
        SERIAL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    use jsonwebtoken::{EncodingKey, Header};
    use ring::signature::{Ed25519KeyPair, KeyPair};

    fn fixture() -> (Verifier, EncodingKey, LicenseTerms) {
        let document = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(document.as_ref()).unwrap();
        let verifier = Verifier {
            keys: BTreeMap::from([("test".into(), STANDARD.encode(pair.public_key().as_ref()))]),
            issuer: "test-issuer".into(),
        };
        let terms = LicenseTerms {
            version: LICENSE_VERSION,
            iss: verifier.issuer.clone(),
            aud: LICENSE_AUDIENCE.into(),
            sub: "customer".into(),
            jti: Uuid::new_v4().to_string(),
            profile_id: Uuid::new_v4(),
            tier: PREMIUM_TIER.into(),
            iat: 100,
            nbf: 100,
            exp: 200,
        };
        (verifier, EncodingKey::from_ed_der(document.as_ref()), terms)
    }

    fn sign(encoding: &EncodingKey, terms: &LicenseTerms) -> String {
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some("test".into());
        STANDARD.encode(jsonwebtoken::encode(&header, terms, encoding).unwrap())
    }

    #[test]
    fn verifies_signature_identity_and_time_boundaries() {
        let (verifier, encoding, terms) = fixture();
        let key = sign(&encoding, &terms);
        assert_eq!(
            verifier.verify(&key, terms.profile_id, 100).unwrap().0,
            LicenseStatus::Valid
        );
        assert_eq!(
            verifier.verify(&key, terms.profile_id, 99).unwrap().0,
            LicenseStatus::NotYetValid
        );
        assert_eq!(
            verifier.verify(&key, terms.profile_id, 200).unwrap().0,
            LicenseStatus::Expired
        );
        assert!(verifier.verify(&key, Uuid::new_v4(), 150).is_err());
        let (_, wrong_key, _) = fixture();
        assert!(verifier
            .verify(&sign(&wrong_key, &terms), terms.profile_id, 150)
            .is_err());
    }

    #[test]
    fn rejects_wrong_domain_tier_version_and_malformed_input() {
        let (verifier, encoding, terms) = fixture();
        for modified in [
            LicenseTerms {
                aud: crate::auth::tokens::AUDIENCE_API.into(),
                ..terms.clone()
            },
            LicenseTerms {
                iss: "attacker".into(),
                ..terms.clone()
            },
            LicenseTerms {
                tier: "unknown".into(),
                ..terms.clone()
            },
            LicenseTerms {
                version: LICENSE_VERSION + 1,
                ..terms.clone()
            },
        ] {
            assert!(verifier
                .verify(&sign(&encoding, &modified), terms.profile_id, 150)
                .is_err());
        }
        assert!(verifier.verify("not-a-key", terms.profile_id, 150).is_err());
    }

    /// Pins today's default: a source build carries no verification keys, so
    /// every licence is rejected and Premium is unreachable. `build.rs` keeps
    /// that state out of release artifacts; this keeps it from being a mystery
    /// when someone hits it locally.
    #[test]
    fn a_build_with_no_bundled_keys_rejects_every_licence() {
        let bundled = Verifier::bundled().expect("bundled key set must parse");
        let (_, encoding, terms) = fixture();
        let signed = sign(&encoding, &terms);
        let outcome = bundled.decode(&signed, terms.profile_id);

        if bundled.keys.is_empty() {
            let error = outcome.expect_err("no keys means nothing can verify");
            assert!(
                error.to_string().contains("unknown license signing key"),
                "unexpected rejection: {error}"
            );
        } else {
            // A build configured with real keys still must not accept a
            // licence signed by this test's throwaway key.
            assert!(outcome.is_err(), "a foreign key must never verify");
        }
    }

    /// The launch path calls this once per gate, seven times per launch; the
    /// signature must be verified once, not seven times.
    #[test]
    fn repeated_reads_verify_the_signature_once() {
        use std::sync::atomic::Ordering;
        let _serial = serial();

        let (verifier, encoding, terms) = fixture();
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        config.profile.id = terms.profile_id;
        install_with(&config, &verifier, terms.nbf, &sign(&encoding, &terms)).unwrap();

        invalidate();
        let before = DECODES.load(Ordering::Relaxed);
        for _ in 0..8 {
            assert!(entitlements_with(&config, &verifier, terms.nbf).premium);
        }
        assert_eq!(
            DECODES.load(Ordering::Relaxed) - before,
            1,
            "eight reads must decode the licence once"
        );
    }

    /// A licence replaced on disk must take effect immediately: the cache keys
    /// on the file, not on the process.
    #[test]
    fn entitlement_is_recomputed_when_the_licence_changes_on_disk() {
        let _serial = serial();
        let (verifier, encoding, terms) = fixture();
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        config.profile.id = terms.profile_id;

        let key = sign(&encoding, &terms);
        assert!(install_with(&config, &verifier, 150, &key).is_ok());
        assert!(entitlements_with(&config, &verifier, 150).premium);

        remove(&config).unwrap();
        assert!(!entitlements_with(&config, &verifier, 150).premium);
        assert_eq!(
            status_with(&config, &verifier, 150).status,
            LicenseStatus::Missing
        );
    }

    #[test]
    fn cached_verification_is_scoped_to_the_verifier() {
        let _serial = serial();
        let (verifier, encoding, terms) = fixture();
        let (other_verifier, _, _) = fixture();
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        config.profile.id = terms.profile_id;

        install_with(&config, &verifier, 150, &sign(&encoding, &terms)).unwrap();
        assert!(entitlements_with(&config, &verifier, 150).premium);
        assert_eq!(
            status_with(&config, &other_verifier, 150).status,
            LicenseStatus::Invalid
        );
    }

    /// The cache stores verified terms, never the derived boolean: an expiry
    /// that passes while the process runs must stop granting entitlement even
    /// though the file never changed.
    #[test]
    fn a_cached_valid_licence_stops_granting_entitlement_once_it_expires() {
        let _serial = serial();
        let (verifier, encoding, terms) = fixture();
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        config.profile.id = terms.profile_id;

        let key = sign(&encoding, &terms);
        install_with(&config, &verifier, terms.nbf, &key).unwrap();
        assert!(entitlements_with(&config, &verifier, terms.nbf).premium);

        let expired = entitlements_with(&config, &verifier, terms.exp);
        assert!(
            !expired.premium,
            "an expired licence must not grant premium"
        );
        assert_eq!(expired.status, LicenseStatus::Expired);
    }

    #[test]
    fn missing_license_allows_local_but_denies_remote() {
        let _serial = serial();
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        assert!(require_target(&config, &TargetDef::local()).is_ok());
        assert!(require_premium(&config, PremiumFeature::RemoteTargets).is_err());
        assert!(install(&config, "bad-key").is_err());
        assert_eq!(status(&config).status, LicenseStatus::Missing);
    }
}
