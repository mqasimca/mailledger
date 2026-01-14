//! PKCE (Proof Key for Code Exchange) implementation for `OAuth2`.
//!
//! PKCE (RFC 7636) enhances security for public clients by preventing
//! authorization code interception attacks.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use sha2::{Digest, Sha256};

/// PKCE code challenge and verifier pair.
#[derive(Debug, Clone)]
pub struct PkceChallenge {
    /// Code verifier (random string).
    pub verifier: String,
    /// Code challenge (SHA256 hash of verifier).
    pub challenge: String,
    /// Challenge method (always S256).
    pub method: String,
}

impl PkceChallenge {
    /// Generates a new PKCE challenge.
    ///
    /// Creates a random 43-character verifier and its SHA256 challenge.
    #[must_use]
    pub fn generate() -> Self {
        let verifier = Self::generate_verifier();
        let challenge = Self::compute_challenge(&verifier);

        Self {
            verifier,
            challenge,
            method: "S256".to_string(),
        }
    }

    /// Generates a random code verifier (43-128 characters).
    fn generate_verifier() -> String {
        let random_bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().r#gen::<u8>()).collect();
        URL_SAFE_NO_PAD.encode(random_bytes)
    }

    /// Computes the code challenge from a verifier using SHA256.
    fn compute_challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        URL_SAFE_NO_PAD.encode(hash)
    }

    /// Returns the verifier.
    #[must_use]
    pub fn verifier(&self) -> &str {
        &self.verifier
    }

    /// Returns the challenge.
    #[must_use]
    pub fn challenge(&self) -> &str {
        &self.challenge
    }

    /// Returns the method.
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let pkce = PkceChallenge::generate();
        assert!(!pkce.verifier.is_empty());
        assert!(!pkce.challenge.is_empty());
        assert_eq!(pkce.method, "S256");
        assert_ne!(pkce.verifier, pkce.challenge);
    }

    #[test]
    fn test_verifier_length() {
        let pkce = PkceChallenge::generate();
        assert!(pkce.verifier.len() >= 43);
        assert!(pkce.verifier.len() <= 128);
    }

    #[test]
    fn test_challenge_computation() {
        let verifier = "test_verifier_string";
        let challenge = PkceChallenge::compute_challenge(verifier);
        assert!(!challenge.is_empty());

        // Same verifier should produce same challenge
        let challenge2 = PkceChallenge::compute_challenge(verifier);
        assert_eq!(challenge, challenge2);
    }

    #[test]
    fn test_multiple_generations_unique() {
        let pkce1 = PkceChallenge::generate();
        let pkce2 = PkceChallenge::generate();
        assert_ne!(pkce1.verifier, pkce2.verifier);
        assert_ne!(pkce1.challenge, pkce2.challenge);
    }
}
