//! Argon2id password hashing for the single admin account.
//!
//! Argon2id (rather than Argon2i or Argon2d) is the OWASP recommendation: it
//! resists both GPU cracking and side-channel attacks. Parameters come from the
//! `argon2` crate's defaults, which track the OWASP guidance; pinning our own
//! numbers here would mean maintaining them by hand as hardware moves.

use anyhow::{anyhow, Result};
use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;

/// Shortest password accepted. Length is the only rule enforced: composition
/// rules (a digit, a symbol) push people toward predictable substitutions
/// without adding real entropy, which is why OWASP dropped them.
pub const MIN_PASSWORD_LENGTH: usize = 12;

/// Longest password accepted. Bounded so a very large body cannot be used to
/// make the server do unbounded KDF work.
pub const MAX_PASSWORD_LENGTH: usize = 1024;

/// Reject passwords that are too short or too long.
pub fn validate_password(password: &str) -> Result<()> {
    let len = password.chars().count();
    if len < MIN_PASSWORD_LENGTH {
        return Err(anyhow!(
            "password must be at least {MIN_PASSWORD_LENGTH} characters"
        ));
    }
    if len > MAX_PASSWORD_LENGTH {
        return Err(anyhow!(
            "password must be at most {MAX_PASSWORD_LENGTH} characters"
        ));
    }
    Ok(())
}

/// Hash a password with a fresh random salt, returning a PHC string that
/// carries the algorithm, parameters, and salt alongside the digest.
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow!("hashing password failed: {e}"))
}

/// Verify a password against a stored PHC hash.
///
/// Returns `Ok(false)` for a wrong password and `Err` only when the stored hash
/// is unreadable — the caller must not treat a corrupt hash as a failed login,
/// because that would silently lock the account instead of surfacing the fault.
pub fn verify_password(password: &str, phc: &str) -> Result<bool> {
    let parsed =
        PasswordHash::new(phc).map_err(|e| anyhow!("stored password hash is invalid: {e}"))?;
    match Argon2::default().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(anyhow!("verifying password failed: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "correct horse battery staple";

    #[test]
    fn test_hash_verifies_against_its_own_password() {
        let phc = hash_password(GOOD).unwrap();
        assert!(verify_password(GOOD, &phc).unwrap());
    }

    #[test]
    fn test_wrong_password_is_rejected_without_erroring() {
        let phc = hash_password(GOOD).unwrap();
        assert!(!verify_password("wrong horse battery staple", &phc).unwrap());
    }

    #[test]
    fn test_hash_is_argon2id_and_salted() {
        let phc = hash_password(GOOD).unwrap();
        assert!(
            phc.starts_with("$argon2id$"),
            "must be Argon2id, not argon2i/argon2d: {phc}"
        );
        // A fresh salt per hash means identical passwords hash differently,
        // so a stolen database cannot be scanned for shared passwords.
        assert_ne!(phc, hash_password(GOOD).unwrap());
    }

    #[test]
    fn test_hash_never_contains_the_plaintext() {
        let phc = hash_password(GOOD).unwrap();
        assert!(!phc.contains(GOOD));
        assert!(!phc.contains("correct"));
    }

    #[test]
    fn test_corrupt_stored_hash_errors_rather_than_reading_as_a_bad_password() {
        // Returning Ok(false) here would present a storage fault as a wrong
        // password, locking the operator out with a misleading message.
        assert!(verify_password(GOOD, "not-a-phc-string").is_err());
        assert!(verify_password(GOOD, "").is_err());
    }

    #[test]
    fn test_password_length_bounds() {
        assert!(validate_password(&"a".repeat(MIN_PASSWORD_LENGTH)).is_ok());
        assert!(validate_password(&"a".repeat(MIN_PASSWORD_LENGTH - 1)).is_err());
        assert!(validate_password(&"a".repeat(MAX_PASSWORD_LENGTH)).is_ok());
        assert!(validate_password(&"a".repeat(MAX_PASSWORD_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_length_is_measured_in_characters_not_bytes() {
        // A 12-character passphrase of multi-byte characters is 12 characters,
        // not 36 bytes; counting bytes would accept a shorter one.
        let emoji = "🔐".repeat(MIN_PASSWORD_LENGTH - 1);
        assert!(validate_password(&emoji).is_err());
        assert!(validate_password(&"🔐".repeat(MIN_PASSWORD_LENGTH)).is_ok());
    }
}
