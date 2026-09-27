//! The vendored verifier is a generated copy. These two tests are the
//! stand-in for a shared crate: the digest catches a local edit, and the
//! vectors catch an edit that also rewrote the digest.

use serde::Deserialize;
use serde_json::Value;

use operator::trust_verify::{verify_license, LicensePolicy, RootKeyring};

#[test]
fn vendored_verifier_digest_matches_header() {
    let source = include_str!("../src/trust_verify.rs");
    let (header, body) = source.split_once("\n\n").expect("header blank line");
    let declared = header
        .lines()
        .find_map(|line| line.strip_prefix("// sha256: "))
        .expect("sha256 line");
    assert_eq!(declared, hex_sha256(body));
}

fn hex_sha256(body: &str) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;
    let mut encoded = String::with_capacity(64);
    for byte in Sha256::digest(body.as_bytes()) {
        write!(encoded, "{byte:02x}").expect("writing to a string");
    }
    encoded
}

#[derive(Deserialize)]
struct Suite {
    product: String,
    issuer: String,
    audience: String,
    roots: Value,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    license_key: String,
    roots: Option<Value>,
    expect: String,
    claims: Option<Value>,
}

#[test]
fn trust_vectors_match_verify_license() {
    let suite: Suite =
        serde_json::from_str(include_str!("../testdata/trust-vectors.json")).unwrap();
    for case in &suite.cases {
        let roots_json =
            serde_json::to_string(case.roots.as_ref().unwrap_or(&suite.roots)).unwrap();
        let roots = RootKeyring::from_json(&roots_json).unwrap_or_else(|error| {
            panic!("{}: roots failed to parse: {error}", case.name);
        });
        let policy = LicensePolicy {
            product: &suite.product,
            issuer: &suite.issuer,
            audience: &suite.audience,
        };
        match verify_license::<Value>(&case.license_key, &roots, &policy) {
            Ok(verified) => {
                assert_eq!(case.expect, "ok", "{}", case.name);
                assert_eq!(
                    verified.claims,
                    case.claims.clone().expect("ok case carries claims"),
                    "{}",
                    case.name
                );
            }
            Err(error) => assert_eq!(error.code(), case.expect, "{}", case.name),
        }
    }
}
