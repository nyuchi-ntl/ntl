//! Error types for the Nyuchi Transfer Layer.

use thiserror::Error;

/// Errors that can occur in NTL operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Signal validation failed.
    #[error("invalid signal: {0}")]
    InvalidSignal(String),

    /// Signal payload is malformed or missing required fields.
    #[error("invalid payload: {0}")]
    InvalidPayload(String),

    /// Cryptographic operation failed.
    #[error("crypto error: {0}")]
    Crypto(String),

    /// Signature verification failed.
    #[error("signature verification failed for signal {signal_id} from node {origin}")]
    SignatureVerification {
        /// The signal that failed verification.
        signal_id: String,
        /// The claimed origin node.
        origin: String,
    },

    /// Synapse operation failed.
    #[error("synapse error: {0}")]
    Synapse(String),

    /// Synapse handshake failed.
    #[error("handshake failed with {remote}: {reason}")]
    HandshakeFailed {
        /// The remote node.
        remote: String,
        /// Reason for failure.
        reason: String,
    },

    /// No compatible crypto module between peers.
    #[error("no compatible crypto module with peer {0}")]
    NoCryptoModule(String),

    /// Propagation error.
    #[error("propagation error: {0}")]
    Propagation(String),

    /// Signal TTL expired.
    #[error("signal TTL expired: {0}")]
    TtlExpired(String),

    /// Duplicate signal detected.
    #[error("duplicate signal: {0}")]
    DuplicateSignal(String),

    /// Node configuration error.
    #[error("config error: {0}")]
    Config(String),

    /// Adapter error.
    #[error("adapter error: {0}")]
    Adapter(String),

    /// Correlation timeout — no response signal received.
    #[error("correlation timeout after {timeout_ms}ms for signal {signal_id}")]
    CorrelationTimeout {
        /// The signal awaiting correlation.
        signal_id: String,
        /// Timeout duration in milliseconds.
        timeout_ms: u64,
    },

    /// Node capacity exceeded.
    #[error("capacity exceeded: {0}")]
    CapacityExceeded(String),

    /// Transport-level error.
    #[error("transport error: {0}")]
    Transport(#[from] std::io::Error),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// The node is shutting down.
    #[error("node is shutting down")]
    Shutdown,
}
