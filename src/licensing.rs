use std::io::Write;
use std::sync::Mutex;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::config::{Config, TargetDef, TargetKind};

pub const PREMIUM_TIER: &str = "premium";
pub const LICENSE_AUDIENCE: &str = "operator-license";
/// Claim version shared with `license`. The envelope, not this number, refuses a bare JWT.
pub const LICENSE_VERSION: u32 = 1;
const LICENSE_FILE: &str = "license.key";
const MAX_LICENSE_BYTES: usize = 32 * 1024;
const PRODUCT_ID: &str = "operator";
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
    roots: crate::trust_verify::RootKeyring,
    issuer: String,
}

/// Chain result plus the attestation window. The window stays off
/// [`LicenseTerms`] because that type is public API.
#[derive(Debug, Clone)]
struct DecodedLicense {
    terms: LicenseTerms,
    attestation_not_before: i64,
    attestation_expires_at: i64,
}

impl DecodedLicense {
    fn effective_exp(&self) -> i64 {
        crate::trust_verify::VerifiedLicense {
            claims: (),
            signing_kid: String::new(),
            attestation_not_before: self.attestation_not_before,
            attestation_expires_at: self.attestation_expires_at,
        }
        .effective_expiry(self.terms.exp)
    }
}

impl Verifier {
    /// A verifier over an explicit root keyring. Enforcement always goes through
    /// [`Verifier::bundled`]; this exists so tests can verify against a root
    /// that is not compiled in.
    #[allow(dead_code)] // Verification seam: used from tests, not the binary
    pub fn from_keys(roots: crate::trust_verify::RootKeyring, issuer: String) -> Self {
        Self { roots, issuer }
    }

    pub fn bundled() -> Result<Self> {
        let roots = crate::trust_verify::RootKeyring::from_json(
            option_env!("OPERATOR_LICENSE_ROOT_KEYS").unwrap_or("{}"),
        )
        .context("invalid bundled license root keys")?;
        Ok(Self {
            roots,
            issuer: option_env!("OPERATOR_LICENSE_ISSUER")
                .unwrap_or("operator-licensing")
                .to_owned(),
        })
    }

    fn cache_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.issuer.hash(&mut hasher);
        self.roots.hash(&mut hasher);
        hasher.finish()
    }

    /// Signature and claim checks. Deliberately time-independent so the result
    /// can be memoised; validity against the clock is [`status_for`].
    fn decode(&self, key: &str, profile_id: Uuid) -> Result<DecodedLicense> {
        anyhow::ensure!(
            !profile_id.is_nil(),
            "configuration identity has not been initialized"
        );
        anyhow::ensure!(key.len() <= MAX_LICENSE_BYTES, "license is too large");
        let verified = crate::trust_verify::verify_license::<LicenseTerms>(
            key,
            &self.roots,
            &crate::trust_verify::LicensePolicy {
                product: PRODUCT_ID,
                issuer: &self.issuer,
                audience: LICENSE_AUDIENCE,
            },
        )?;
        let terms = verified.claims;
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
        Ok(DecodedLicense {
            terms,
            attestation_not_before: verified.attestation_not_before,
            attestation_expires_at: verified.attestation_expires_at,
        })
    }

    fn verify(
        &self,
        key: &str,
        profile_id: Uuid,
        now: i64,
    ) -> Result<(LicenseStatus, LicenseTerms)> {
        let decoded = self.decode(key, profile_id)?;
        let status = status_for(&decoded, now);
        Ok((status, decoded.terms))
    }
}

/// Where `now` falls relative to the licence and its attestation.
fn status_for(decoded: &DecodedLicense, now: i64) -> LicenseStatus {
    if now >= decoded.effective_exp() {
        LicenseStatus::Expired
    } else if now < decoded.terms.nbf
        || now < decoded.terms.iat
        || now < decoded.attestation_not_before
    {
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
    Terms(Box<DecodedLicense>),
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
                    Ok(decoded) => Verified::Terms(Box::new(decoded)),
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
        Verified::Terms(decoded) => {
            let status = status_for(&decoded, now);
            LicenseResponse {
                status,
                profile_id: config.profile.id,
                premium: status == LicenseStatus::Valid,
                terms: Some(decoded.terms),
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
    use base64::{engine::general_purpose::STANDARD, Engine};
    use jsonwebtoken::{Algorithm, EncodingKey, Header};
    use ring::signature::{Ed25519KeyPair, KeyPair};

    const ROOT_KID: &str = "root-test";
    const SIGNING_KID: &str = "test";
    const ATTESTATION_NBF: i64 = 2_000;
    const ATTESTATION_EXP: i64 = 3_000;
    const NOW: i64 = 2_500;

    struct Fixture {
        verifier: Verifier,
        license_key: EncodingKey,
        attestation: String,
        terms: LicenseTerms,
    }

    fn generate_ed25519() -> (Ed25519KeyPair, Vec<u8>) {
        let document = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let bytes = document.as_ref().to_vec();
        let pair = Ed25519KeyPair::from_pkcs8(&bytes).unwrap();
        (pair, bytes)
    }

    fn mint() -> Fixture {
        let (root, root_pkcs8) = generate_ed25519();
        let (signing, signing_pkcs8) = generate_ed25519();
        let root_b64 = STANDARD.encode(root.public_key().as_ref());
        let roots = crate::trust_verify::RootKeyring::from_json(&format!(
            r#"{{"{ROOT_KID}":"{root_b64}"}}"#
        ))
        .unwrap();
        let verifier = Verifier::from_keys(roots, "test-issuer".into());
        let attestation = sign_attestation(&root_pkcs8, signing.public_key().as_ref());
        let terms = LicenseTerms {
            version: LICENSE_VERSION,
            iss: verifier.issuer.clone(),
            aud: LICENSE_AUDIENCE.into(),
            sub: "customer".into(),
            jti: Uuid::new_v4().to_string(),
            profile_id: Uuid::new_v4(),
            tier: PREMIUM_TIER.into(),
            iat: ATTESTATION_NBF,
            nbf: ATTESTATION_NBF,
            exp: ATTESTATION_EXP,
        };
        Fixture {
            verifier,
            license_key: EncodingKey::from_ed_der(&signing_pkcs8),
            attestation,
            terms,
        }
    }

    fn sign_attestation(root_pkcs8: &[u8], signing_public: &[u8]) -> String {
        let claims = crate::trust_verify::AttestationClaims {
            version: crate::trust_verify::ATTESTATION_VERSION,
            iss: crate::trust_verify::ATTESTATION_ISSUER.into(),
            aud: crate::trust_verify::ATTESTATION_AUDIENCE.into(),
            sub: PRODUCT_ID.into(),
            purpose: crate::trust_verify::PURPOSE_LICENSE_SIGNING.into(),
            kid: SIGNING_KID.into(),
            public_key: STANDARD.encode(signing_public),
            iat: ATTESTATION_NBF,
            nbf: ATTESTATION_NBF,
            exp: ATTESTATION_EXP,
        };
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(ROOT_KID.into());
        jsonwebtoken::encode(&header, &claims, &EncodingKey::from_ed_der(root_pkcs8)).unwrap()
    }

    fn sign(fixture: &Fixture, terms: &LicenseTerms) -> String {
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(SIGNING_KID.into());
        let license = jsonwebtoken::encode(&header, terms, &fixture.license_key).unwrap();
        crate::trust_verify::Envelope::new(license, fixture.attestation.clone()).encode()
    }

    #[test]
    fn verifies_signature_identity_and_time_boundaries() {
        let fixture = mint();
        let key = sign(&fixture, &fixture.terms);
        assert_eq!(
            fixture
                .verifier
                .verify(&key, fixture.terms.profile_id, ATTESTATION_NBF)
                .unwrap()
                .0,
            LicenseStatus::Valid
        );
        assert_eq!(
            fixture
                .verifier
                .verify(&key, fixture.terms.profile_id, ATTESTATION_NBF - 1)
                .unwrap()
                .0,
            LicenseStatus::NotYetValid
        );
        assert_eq!(
            fixture
                .verifier
                .verify(&key, fixture.terms.profile_id, ATTESTATION_EXP)
                .unwrap()
                .0,
            LicenseStatus::Expired
        );
        assert!(fixture.verifier.verify(&key, Uuid::new_v4(), NOW).is_err());
        let other = mint();
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(SIGNING_KID.into());
        let forged_license =
            jsonwebtoken::encode(&header, &fixture.terms, &other.license_key).unwrap();
        let forged =
            crate::trust_verify::Envelope::new(forged_license, fixture.attestation.clone())
                .encode();
        assert!(fixture
            .verifier
            .verify(&forged, fixture.terms.profile_id, NOW)
            .is_err());
    }

    #[test]
    fn rejects_wrong_domain_tier_version_and_malformed_input() {
        let fixture = mint();
        for modified in [
            LicenseTerms {
                aud: crate::auth::tokens::AUDIENCE_API.into(),
                ..fixture.terms.clone()
            },
            LicenseTerms {
                iss: "attacker".into(),
                ..fixture.terms.clone()
            },
            LicenseTerms {
                tier: "unknown".into(),
                ..fixture.terms.clone()
            },
            LicenseTerms {
                version: LICENSE_VERSION + 1,
                ..fixture.terms.clone()
            },
        ] {
            assert!(fixture
                .verifier
                .verify(&sign(&fixture, &modified), fixture.terms.profile_id, NOW)
                .is_err());
        }
        assert!(fixture
            .verifier
            .verify("not-a-key", fixture.terms.profile_id, NOW)
            .is_err());
    }

    /// Pins today's default: a source build carries no verification keys, so
    /// every licence is rejected and Premium is unreachable. `build.rs` keeps
    /// that state out of release artifacts; this keeps it from being a mystery
    /// when someone hits it locally.
    #[test]
    fn a_build_with_no_bundled_keys_rejects_every_licence() {
        let bundled = Verifier::bundled().expect("bundled key set must parse");
        let fixture = mint();
        let signed = sign(&fixture, &fixture.terms);
        let outcome = bundled.decode(&signed, fixture.terms.profile_id);

        if bundled.roots.is_empty() {
            let error = outcome.expect_err("no roots means nothing can verify");
            assert!(
                error
                    .to_string()
                    .contains("license attestation was signed by an untrusted root"),
                "unexpected rejection: {error}"
            );
        } else {
            // A build configured with real roots still must not accept a
            // licence signed under this test's throwaway root.
            assert!(outcome.is_err(), "a foreign root must never verify");
        }
    }

    #[test]
    fn a_bare_base64_jwt_is_refused() {
        let fixture = mint();
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(SIGNING_KID.into());
        let jwt = jsonwebtoken::encode(&header, &fixture.terms, &fixture.license_key).unwrap();
        let legacy = STANDARD.encode(jwt);
        let error = fixture
            .verifier
            .decode(&legacy, fixture.terms.profile_id)
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("license is not a valid envelope"),
            "{error}"
        );
    }

    #[test]
    fn an_hmac_license_inside_a_valid_envelope_is_refused() {
        let fixture = mint();
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some(SIGNING_KID.into());
        let hmac = EncodingKey::from_secret(b"not-the-signing-key");
        let license = jsonwebtoken::encode(&header, &fixture.terms, &hmac).unwrap();
        let key = crate::trust_verify::Envelope::new(license, fixture.attestation).encode();
        let error = fixture
            .verifier
            .verify(&key, fixture.terms.profile_id, NOW)
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("unsupported signature algorithm"),
            "{error}"
        );
    }

    #[test]
    fn attestation_window_bounds_the_license_clock() {
        let fixture = mint();
        let terms = LicenseTerms {
            iat: 1_000,
            nbf: 1_000,
            exp: 5_000,
            ..fixture.terms.clone()
        };
        let key = sign(&fixture, &terms);
        assert_eq!(
            fixture
                .verifier
                .verify(&key, terms.profile_id, NOW)
                .unwrap()
                .0,
            LicenseStatus::Valid
        );
        assert_eq!(
            fixture
                .verifier
                .verify(&key, terms.profile_id, 1_500)
                .unwrap()
                .0,
            LicenseStatus::NotYetValid
        );
        assert_eq!(
            fixture
                .verifier
                .verify(&key, terms.profile_id, 3_500)
                .unwrap()
                .0,
            LicenseStatus::Expired
        );
        let (status, reported) = fixture
            .verifier
            .verify(&key, terms.profile_id, NOW)
            .unwrap();
        assert_eq!(status, LicenseStatus::Valid);
        assert_eq!(reported.exp, 5_000, "terms.exp stays the license exp");
    }

    /// The launch path calls this once per gate, seven times per launch; the
    /// signature must be verified once, not seven times.
    #[test]
    fn repeated_reads_verify_the_signature_once() {
        use std::sync::atomic::Ordering;
        let _serial = serial();

        let fixture = mint();
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        config.profile.id = fixture.terms.profile_id;
        install_with(
            &config,
            &fixture.verifier,
            fixture.terms.nbf,
            &sign(&fixture, &fixture.terms),
        )
        .unwrap();

        invalidate();
        let before = DECODES.load(Ordering::Relaxed);
        for _ in 0..8 {
            assert!(entitlements_with(&config, &fixture.verifier, fixture.terms.nbf).premium);
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
        let fixture = mint();
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        config.profile.id = fixture.terms.profile_id;

        let key = sign(&fixture, &fixture.terms);
        assert!(install_with(&config, &fixture.verifier, NOW, &key).is_ok());
        assert!(entitlements_with(&config, &fixture.verifier, NOW).premium);

        remove(&config).unwrap();
        assert!(!entitlements_with(&config, &fixture.verifier, NOW).premium);
        assert_eq!(
            status_with(&config, &fixture.verifier, NOW).status,
            LicenseStatus::Missing
        );
    }

    #[test]
    fn cached_verification_is_scoped_to_the_verifier() {
        let _serial = serial();
        let fixture = mint();
        let other = mint();
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        config.profile.id = fixture.terms.profile_id;

        install_with(
            &config,
            &fixture.verifier,
            NOW,
            &sign(&fixture, &fixture.terms),
        )
        .unwrap();
        assert!(entitlements_with(&config, &fixture.verifier, NOW).premium);
        assert_eq!(
            status_with(&config, &other.verifier, NOW).status,
            LicenseStatus::Invalid
        );
    }

    /// The cache stores verified terms, never the derived boolean: an expiry
    /// that passes while the process runs must stop granting entitlement even
    /// though the file never changed.
    #[test]
    fn a_cached_valid_licence_stops_granting_entitlement_once_it_expires() {
        let _serial = serial();
        let fixture = mint();
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        config.profile.id = fixture.terms.profile_id;

        let key = sign(&fixture, &fixture.terms);
        install_with(&config, &fixture.verifier, fixture.terms.nbf, &key).unwrap();
        assert!(entitlements_with(&config, &fixture.verifier, fixture.terms.nbf).premium);

        let expired = entitlements_with(&config, &fixture.verifier, fixture.terms.exp);
        assert!(
            !expired.premium,
            "an expired licence must not grant premium"
        );
        assert_eq!(expired.status, LicenseStatus::Expired);
    }

    #[test]
    fn a_rotated_root_keyring_is_not_served_from_the_memo_cache() {
        use std::sync::atomic::Ordering;
        let _serial = serial();
        let fixture = mint();
        let directory = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = directory.path().to_string_lossy().into_owned();
        config.profile.id = fixture.terms.profile_id;
        let key = sign(&fixture, &fixture.terms);
        install_with(&config, &fixture.verifier, fixture.terms.nbf, &key).unwrap();
        invalidate();
        let before = DECODES.load(Ordering::Relaxed);
        assert!(entitlements_with(&config, &fixture.verifier, fixture.terms.nbf).premium);
        assert_eq!(DECODES.load(Ordering::Relaxed) - before, 1);

        let rotated = mint();
        let after = DECODES.load(Ordering::Relaxed);
        let again = status_with(&config, &rotated.verifier, fixture.terms.nbf);
        assert_eq!(again.status, LicenseStatus::Invalid);
        assert!(!again.premium);
        assert_eq!(
            DECODES.load(Ordering::Relaxed) - after,
            1,
            "rotated roots must recompute"
        );
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
