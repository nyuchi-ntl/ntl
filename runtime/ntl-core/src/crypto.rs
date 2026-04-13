//! Pluggable cryptographic module interface.
//!
//! NTL treats cryptography as a swappable module, not a foundational
//! dependency. This enables post-quantum readiness and crypto agility.

use crate::Result;

/// Public key bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(pub Vec<u8>);

/// Private key bytes.
#[derive(Debug, Clone)]
pub struct PrivateKey(pub Vec<u8>);

/// Cryptographic signature bytes.
#[derive(Debug, Clone)]
pub struct Signature(pub Vec<u8>);

/// Shared secret derived from key exchange.
#[derive(Debug, Clone)]
pub struct SharedSecret(pub Vec<u8>);

/// Hash output bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash(pub Vec<u8>);

/// The pluggable cryptographic module interface.
///
/// All cryptographic operations in NTL go through this trait.
/// Implementations can be swapped at runtime.
pub trait CryptoModule: Send + Sync {
    /// Unique identifier for this module (e.g., "pq-v1", "classical-v1").
    fn module_id(&self) -> &str;

    /// Generate a new keypair.
    fn generate_keypair(&self) -> Result<(PublicKey, PrivateKey)>;

    /// Sign data with a private key.
    fn sign(&self, data: &[u8], key: &PrivateKey) -> Result<Signature>;

    /// Verify a signature against a public key.
    fn verify(&self, data: &[u8], signature: &Signature, key: &PublicKey) -> Result<bool>;

    /// Encrypt data for a recipient's public key.
    fn encrypt(&self, data: &[u8], recipient_key: &PublicKey) -> Result<Vec<u8>>;

    /// Decrypt data with a private key.
    fn decrypt(&self, data: &[u8], key: &PrivateKey) -> Result<Vec<u8>>;

    /// Perform key exchange to derive a shared secret.
    fn key_exchange(
        &self,
        local_private: &PrivateKey,
        remote_public: &PublicKey,
    ) -> Result<SharedSecret>;

    /// Compute a cryptographic hash.
    fn hash(&self, data: &[u8]) -> Hash;
}

/// BLAKE3-based hashing (used across all crypto modules).
pub fn blake3_hash(data: &[u8]) -> Hash {
    Hash(blake3::hash(data).as_bytes().to_vec())
}

/// Derive a NodeId from a public key.
pub fn node_id_from_public_key(key: &PublicKey) -> crate::signal::NodeId {
    let hash = blake3_hash(&key.0);
    crate::signal::NodeId(hash.0)
}

// TODO: Implement PostQuantumModule (Dilithium + Kyber)
// TODO: Implement ClassicalModule (Ed25519 + X25519)
// TODO: Implement HybridModule (dual signatures)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake3_hash_deterministic() {
        let data = b"hello NTL";
        let h1 = blake3_hash(data);
        let h2 = blake3_hash(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn blake3_hash_different_inputs() {
        let h1 = blake3_hash(b"hello");
        let h2 = blake3_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn node_id_from_key_deterministic() {
        let key = PublicKey(vec![42u8; 32]);
        let id1 = node_id_from_public_key(&key);
        let id2 = node_id_from_public_key(&key);
        assert_eq!(id1, id2);
    }
}
