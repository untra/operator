//! Generating and comparing opaque credentials.
//!
//! Two distinct jobs live here, and conflating them is a classic mistake:
//!
//! * **Passwords** are low-entropy and human-chosen, so they need a slow,
//!   salted KDF (Argon2id — see [`super::password`]).
//! * **Opaque tokens** (session cookies, refresh tokens, device codes, access
//!   keys) are 256-bit random values *we* generate. They need only a fast
//!   pre-image-resistant hash; Argon2 on a lookup path would add latency for
//!   no security benefit, because there is nothing to brute-force.
//!
//! Both lookups compare in constant time.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use rand::TryRngCore;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Bytes of entropy in a generated credential. 256 bits — well beyond any
/// offline search, and the reason a fast hash suffices for storage.
const SECRET_BYTES: usize = 32;

/// Characters used for the human-typed device user code.
///
/// Digits and letters that are hard to confuse aloud or in a terminal font are
/// excluded: no `0`/`O`, no `1`/`I`/`L`, no `U` (misheard as "you").
const USER_CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTVWXYZ23456789";

/// Generate a URL-safe opaque secret with 256 bits of entropy.
///
/// Uses the OS CSPRNG and propagates failure rather than falling back to a
/// weaker source: a silently non-random credential is worse than no credential.
pub fn generate_secret() -> anyhow::Result<String> {
    let mut bytes = [0u8; SECRET_BYTES];
    rand::rngs::OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|e| anyhow::anyhow!("OS random number generator unavailable: {e}"))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// Generate a prefixed secret, e.g. `opk_<random>` for an access key. The
/// prefix makes a leaked credential recognizable in logs and secret scanners.
pub fn generate_prefixed_secret(prefix: &str) -> anyhow::Result<String> {
    Ok(format!("{prefix}_{}", generate_secret()?))
}

/// Generate a short, human-typed device code formatted `XXXX-XXXX`.
pub fn generate_user_code() -> anyhow::Result<String> {
    let mut bytes = [0u8; 8];
    rand::rngs::OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|e| anyhow::anyhow!("OS random number generator unavailable: {e}"))?;

    let n = USER_CODE_ALPHABET.len();
    let chars: Vec<char> = bytes
        .iter()
        // Modulo bias across a 30-character alphabet from a 256-value byte is
        // at most ~2%, which is immaterial for a code that lives for minutes,
        // is rate-limited, and is single-use.
        .map(|b| USER_CODE_ALPHABET[usize::from(*b) % n] as char)
        .collect();

    Ok(format!(
        "{}-{}",
        chars[..4].iter().collect::<String>(),
        chars[4..].iter().collect::<String>()
    ))
}

/// Hash an opaque secret for storage. Never store the secret itself.
pub fn hash_secret(secret: &str) -> String {
    let digest = Sha256::digest(secret.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

/// Constant-time comparison of two hashes.
///
/// Lookups are by hash equality, so a short-circuiting `==` would leak the
/// matching prefix length through timing.
pub fn hashes_equal(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_generated_secrets_are_unique_and_url_safe() {
        let mut seen = HashSet::new();
        for _ in 0..256 {
            let s = generate_secret().unwrap();
            assert!(seen.insert(s.clone()), "generated a duplicate secret");
            assert!(
                s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                "secret must be URL-safe so it can ride in a cookie or header: {s}"
            );
        }
    }

    #[test]
    fn test_secret_carries_full_entropy() {
        // 32 bytes base64url without padding is 43 characters.
        assert_eq!(generate_secret().unwrap().len(), 43);
    }

    #[test]
    fn test_prefixed_secret_is_recognizable() {
        let key = generate_prefixed_secret("opk").unwrap();
        assert!(key.starts_with("opk_"));
        assert_eq!(key.len(), 4 + 43);
    }

    #[test]
    fn test_user_code_avoids_visually_ambiguous_characters() {
        for _ in 0..128 {
            let code = generate_user_code().unwrap();
            assert_eq!(code.len(), 9, "expected XXXX-XXXX: {code}");
            assert_eq!(&code[4..5], "-");
            for c in code.chars().filter(|c| *c != '-') {
                assert!(
                    !"O01ILU".contains(c),
                    "user code must avoid characters confused when read aloud: {code}"
                );
                assert!(USER_CODE_ALPHABET.contains(&(c as u8)));
            }
        }
    }

    #[test]
    fn test_hash_is_deterministic_and_hides_the_secret() {
        let secret = generate_secret().unwrap();
        let hash = hash_secret(&secret);
        assert_eq!(hash, hash_secret(&secret));
        assert!(!hash.contains(&secret));
        assert_ne!(hash, secret);
    }

    #[test]
    fn test_distinct_secrets_hash_differently() {
        let a = hash_secret(&generate_secret().unwrap());
        let b = hash_secret(&generate_secret().unwrap());
        assert_ne!(a, b);
    }

    #[test]
    fn test_hashes_equal_matches_only_identical_input() {
        let hash = hash_secret("token");
        assert!(hashes_equal(&hash, &hash_secret("token")));
        assert!(!hashes_equal(&hash, &hash_secret("token ")));
        // Differing lengths must compare false rather than panic.
        assert!(!hashes_equal(&hash, "short"));
    }
}
