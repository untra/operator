//! Premium licence verification, end to end through the public API.
//!
//! Verification is offline and the keys are compiled in, so these tests build a
//! [`Verifier`] over a throwaway Ed25519 key and drive the same
//! `status_with` / `install_with` entry points the bundled path uses. Nothing
//! here depends on how the shipped binary was configured.

use std::collections::BTreeMap;

use base64::{engine::general_purpose::STANDARD, Engine};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use ring::signature::{Ed25519KeyPair, KeyPair};
use uuid::Uuid;

use operator::config::Config;
use operator::licensing::{
    install_with, status_with, LicenseStatus, LicenseTerms, Verifier, LICENSE_AUDIENCE,
    LICENSE_VERSION, PREMIUM_TIER,
};

const KID: &str = "test-key";
const ISSUER: &str = "operator-licensing-test";

/// A configuration rooted in `directory`, bound to `profile`.
fn config_for(directory: &std::path::Path, profile: Uuid) -> Config {
    let mut config = Config::default();
    config.paths.state = directory.to_string_lossy().into_owned();
    config.paths.tickets = directory.join("tickets").to_string_lossy().into_owned();
    config.profile.id = profile;
    config
}

struct Issuer {
    verifier: Verifier,
    encoding: EncodingKey,
}

impl Issuer {
    fn new() -> Self {
        let document = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(document.as_ref()).unwrap();
        let keys = BTreeMap::from([(KID.to_string(), STANDARD.encode(pair.public_key().as_ref()))]);
        Self {
            verifier: Verifier::from_keys(keys, ISSUER.to_string()),
            encoding: EncodingKey::from_ed_der(document.as_ref()),
        }
    }

    fn sign(&self, terms: &LicenseTerms) -> String {
        self.sign_with(terms, Algorithm::EdDSA, Some(KID.to_string()))
    }

    fn sign_with(&self, terms: &LicenseTerms, alg: Algorithm, kid: Option<String>) -> String {
        let mut header = Header::new(alg);
        header.kid = kid;
        STANDARD.encode(jsonwebtoken::encode(&header, terms, &self.encoding).unwrap())
    }
}

fn terms_for(profile: Uuid) -> LicenseTerms {
    LicenseTerms {
        version: LICENSE_VERSION,
        iss: ISSUER.to_string(),
        aud: LICENSE_AUDIENCE.to_string(),
        sub: "customer@example.test".to_string(),
        jti: Uuid::new_v4().to_string(),
        profile_id: profile,
        tier: PREMIUM_TIER.to_string(),
        iat: 1_000,
        nbf: 1_000,
        exp: 2_000,
    }
}

/// Install `key`, then report the status the configuration ends up with.
fn install_then_status(issuer: &Issuer, key: &str, now: i64) -> (bool, LicenseStatus) {
    let directory = tempfile::tempdir().unwrap();
    let profile = Uuid::new_v4();
    let config = config_for(directory.path(), profile);
    let accepted = install_with(&config, &issuer.verifier, now, key).is_ok();
    let status = status_with(&config, &issuer.verifier, now).status;
    (accepted, status)
}

#[test]
fn a_valid_licence_installs_and_grants_premium() {
    let issuer = Issuer::new();
    let directory = tempfile::tempdir().unwrap();
    let profile = Uuid::new_v4();
    let config = config_for(directory.path(), profile);
    let terms = terms_for(profile);

    let installed = install_with(&config, &issuer.verifier, 1_500, &issuer.sign(&terms)).unwrap();

    assert_eq!(installed.status, LicenseStatus::Valid);
    assert!(installed.premium);
    let reported = installed.terms.expect("verified terms are reported");
    assert_eq!(reported.sub, terms.sub);
    assert_eq!(reported.jti, terms.jti);
    assert_eq!(reported.profile_id, profile);
}

#[test]
fn a_licence_read_never_returns_the_raw_key() {
    let issuer = Issuer::new();
    let directory = tempfile::tempdir().unwrap();
    let profile = Uuid::new_v4();
    let config = config_for(directory.path(), profile);
    let key = issuer.sign(&terms_for(profile));
    install_with(&config, &issuer.verifier, 1_500, &key).unwrap();

    let response = status_with(&config, &issuer.verifier, 1_500);
    let json = serde_json::to_string(&response).unwrap();

    assert!(
        !json.contains(&key),
        "the licence key must never appear in a read response"
    );
}

#[test]
fn expiry_and_start_dates_are_evaluated_against_the_clock() {
    let issuer = Issuer::new();
    let directory = tempfile::tempdir().unwrap();
    let profile = Uuid::new_v4();
    let config = config_for(directory.path(), profile);
    let terms = terms_for(profile);
    install_with(&config, &issuer.verifier, terms.nbf, &issuer.sign(&terms)).unwrap();

    for (now, expected) in [
        (terms.nbf - 1, LicenseStatus::NotYetValid),
        (terms.nbf, LicenseStatus::Valid),
        (terms.exp - 1, LicenseStatus::Valid),
        (terms.exp, LicenseStatus::Expired),
    ] {
        let response = status_with(&config, &issuer.verifier, now);
        assert_eq!(response.status, expected, "at now={now}");
        assert_eq!(response.premium, expected == LicenseStatus::Valid);
    }
}

#[test]
fn every_rejected_licence_shape_is_refused() {
    let issuer = Issuer::new();
    let profile = Uuid::new_v4();
    let base = terms_for(profile);

    let wrong_issuer = LicenseTerms {
        iss: "attacker".to_string(),
        ..base.clone()
    };
    let wrong_audience = LicenseTerms {
        aud: "operator-api".to_string(),
        ..base.clone()
    };
    let unknown_tier = LicenseTerms {
        tier: "enterprise".to_string(),
        ..base.clone()
    };
    let wrong_version = LicenseTerms {
        version: LICENSE_VERSION + 1,
        ..base.clone()
    };
    let other_configuration = LicenseTerms {
        profile_id: Uuid::new_v4(),
        ..base.clone()
    };

    let cases: Vec<(&str, String)> = vec![
        ("malformed base64", "not-a-licence".to_string()),
        ("not a JWT", STANDARD.encode("plain text")),
        ("wrong issuer", issuer.sign(&wrong_issuer)),
        ("wrong audience", issuer.sign(&wrong_audience)),
        ("unknown tier", issuer.sign(&unknown_tier)),
        ("unsupported version", issuer.sign(&wrong_version)),
        ("another configuration", issuer.sign(&other_configuration)),
        (
            "unknown key id",
            issuer.sign_with(&base, Algorithm::EdDSA, Some("nope".to_string())),
        ),
        ("no key id", issuer.sign_with(&base, Algorithm::EdDSA, None)),
    ];

    for (name, key) in cases {
        let (accepted, status) = install_then_status(&issuer, &key, 1_500);
        assert!(!accepted, "{name} must not install");
        assert!(
            matches!(status, LicenseStatus::Missing | LicenseStatus::Invalid),
            "{name} left status {status:?}"
        );
    }
}

/// Algorithm confusion: the verifier pins EdDSA, so a symmetric token whose
/// "signature" the attacker also controls must never be considered.
#[test]
fn a_symmetric_algorithm_is_refused() {
    let issuer = Issuer::new();
    let profile = Uuid::new_v4();
    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some(KID.to_string());
    let hmac = EncodingKey::from_secret(b"not-the-signing-key");
    let token = jsonwebtoken::encode(&header, &terms_for(profile), &hmac).unwrap();

    let (accepted, status) = install_then_status(&issuer, &STANDARD.encode(token), 1_500);

    assert!(!accepted, "an HS256 licence must not install");
    assert_eq!(status, LicenseStatus::Missing);
}

#[test]
fn a_licence_signed_by_another_key_is_refused() {
    let issuer = Issuer::new();
    let attacker = Issuer::new();
    let profile = Uuid::new_v4();
    // Signed by the attacker, presented under the real issuer's key id.
    let forged = attacker.sign(&terms_for(profile));

    let (accepted, status) = install_then_status(&issuer, &forged, 1_500);

    assert!(!accepted, "a foreign signature must not install");
    assert_eq!(status, LicenseStatus::Missing);
}

#[test]
fn a_tampered_payload_is_refused() {
    let issuer = Issuer::new();
    let profile = Uuid::new_v4();
    let signed = issuer.sign(&terms_for(profile));

    // Re-encode the token with one payload byte changed.
    let token = String::from_utf8(STANDARD.decode(&signed).unwrap()).unwrap();
    let mut parts: Vec<String> = token.split('.').map(str::to_string).collect();
    let payload = parts[1].clone();
    parts[1] = payload
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 4 {
                if c == 'A' {
                    'B'
                } else {
                    'A'
                }
            } else {
                c
            }
        })
        .collect();
    let tampered = STANDARD.encode(parts.join("."));

    let (accepted, status) = install_then_status(&issuer, &tampered, 1_500);

    assert!(!accepted, "a tampered payload must not install");
    assert_eq!(status, LicenseStatus::Missing);
}

/// An install validates before it writes, so a bad replacement is not a way to
/// knock out a working licence.
#[test]
fn an_invalid_replacement_preserves_the_installed_licence() {
    let issuer = Issuer::new();
    let directory = tempfile::tempdir().unwrap();
    let profile = Uuid::new_v4();
    let config = config_for(directory.path(), profile);
    let good = issuer.sign(&terms_for(profile));
    install_with(&config, &issuer.verifier, 1_500, &good).unwrap();

    for bad in [
        "garbage".to_string(),
        issuer.sign(&LicenseTerms {
            tier: "enterprise".to_string(),
            ..terms_for(profile)
        }),
        Issuer::new().sign(&terms_for(profile)),
    ] {
        assert!(install_with(&config, &issuer.verifier, 1_500, &bad).is_err());
        let response = status_with(&config, &issuer.verifier, 1_500);
        assert_eq!(response.status, LicenseStatus::Valid);
        assert!(response.premium, "the working licence must survive");
    }
}

/// Renaming a configuration keeps its id, so the licence must keep verifying.
#[test]
fn a_licence_survives_renaming_its_configuration() {
    let issuer = Issuer::new();
    let directory = tempfile::tempdir().unwrap();
    let profile = Uuid::new_v4();
    let mut config = config_for(directory.path(), profile);
    install_with(
        &config,
        &issuer.verifier,
        1_500,
        &issuer.sign(&terms_for(profile)),
    )
    .unwrap();

    config.profile.name = "renamed-workspace".to_string();

    assert!(status_with(&config, &issuer.verifier, 1_500).premium);
}

/// The licence is stored with owner-only permissions: it is a credential, not
/// ordinary configuration.
#[cfg(unix)]
#[test]
fn the_stored_licence_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let issuer = Issuer::new();
    let directory = tempfile::tempdir().unwrap();
    let profile = Uuid::new_v4();
    let config = config_for(directory.path(), profile);
    install_with(
        &config,
        &issuer.verifier,
        1_500,
        &issuer.sign(&terms_for(profile)),
    )
    .unwrap();

    let mode = std::fs::metadata(directory.path().join("license.key"))
        .unwrap()
        .permissions()
        .mode();

    assert_eq!(mode & 0o077, 0, "group and other must have no access");
}
