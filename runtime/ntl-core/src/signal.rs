//! Signal types and construction for NTL.
//!
//! A signal is the fundamental data unit in NTL, replacing the concept of
//! a "request" or "message" from traditional protocols.

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::propagation::PropagationScope;

/// Unique identifier for a signal, based on ULID for lexicographic time-ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SignalId(Ulid);

impl SignalId {
    /// Generate a new signal ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Ulid::new())
    }

    /// Get the timestamp component of the ID.
    #[must_use]
    pub fn timestamp_ms(&self) -> u64 {
        self.0.timestamp_ms()
    }
}

impl Default for SignalId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SignalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The type of a signal, determining how it's routed and processed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalType {
    /// Carries a data payload.
    Data,
    /// Requests data from the network.
    Query,
    /// Notifies of a state change.
    Event,
    /// Requests an action.
    Command,
    /// Maintains synapse liveness.
    Heartbeat,
    /// Announces node capability.
    Discovery,
    /// Confirms signal receipt.
    Ack,
    /// Application-defined signal type.
    Custom(String),
}

impl SignalType {
    /// Convert to the wire format type byte.
    #[must_use]
    pub fn to_type_byte(&self) -> u8 {
        match self {
            Self::Data => 0,
            Self::Query => 1,
            Self::Event => 2,
            Self::Command => 3,
            Self::Heartbeat => 4,
            Self::Discovery => 5,
            Self::Ack => 6,
            Self::Custom(_) => 15,
        }
    }

    /// Parse from wire format type byte.
    #[must_use]
    pub fn from_type_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Data),
            1 => Some(Self::Query),
            2 => Some(Self::Event),
            3 => Some(Self::Command),
            4 => Some(Self::Heartbeat),
            5 => Some(Self::Discovery),
            6 => Some(Self::Ack),
            _ => None,
        }
    }
}

/// Encoding format for signal payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Encoding {
    /// CBOR — default, compact binary, self-describing.
    Cbor = 0,
    /// Protocol Buffers — when schema is shared.
    Protobuf = 1,
    /// Raw unstructured bytes.
    Raw = 2,
}

/// A node identifier, derived from the node's public key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Vec<u8>);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex: String = self.0.iter().take(8).map(|b| format!("{b:02x}")).collect();
        write!(f, "ntl:{hex}...")
    }
}

/// An NTL signal — the fundamental data unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// Unique signal identifier (ULID).
    pub id: SignalId,

    /// Signal type classification.
    pub signal_type: SignalType,

    /// Protocol version.
    pub version: u8,

    /// Emitting node identity.
    pub origin: NodeId,

    /// Cryptographic signature over the signal body.
    pub signature: Vec<u8>,

    /// Emission timestamp in nanoseconds since Unix epoch.
    pub timestamp: u64,

    /// The data payload.
    pub payload: serde_json::Value,

    /// Payload encoding format.
    pub encoding: Encoding,

    /// Signal weight / priority (0.0 - 1.0).
    pub weight: f32,

    /// Time-to-live in hops.
    pub ttl: u16,

    /// Propagation scope.
    pub scope: PropagationScope,

    /// Links to a related signal for request-response patterns.
    pub correlation_id: Option<SignalId>,

    /// Ordered list of nodes this signal has traversed.
    pub trace: Vec<NodeId>,

    /// Searchable tags.
    pub tags: Vec<String>,
}

/// Builder for constructing signals with a fluent API.
pub struct SignalBuilder {
    signal_type: SignalType,
    topic: String,
    payload: serde_json::Value,
    weight: f32,
    ttl: u16,
    scope: PropagationScope,
    correlation_id: Option<SignalId>,
    tags: Vec<String>,
}

impl Signal {
    /// Create a Data signal builder.
    #[must_use]
    pub fn data(topic: &str) -> SignalBuilder {
        SignalBuilder::new(SignalType::Data, topic)
    }

    /// Create a Query signal builder.
    #[must_use]
    pub fn query(topic: &str) -> SignalBuilder {
        SignalBuilder::new(SignalType::Query, topic)
    }

    /// Create an Event signal builder.
    #[must_use]
    pub fn event(topic: &str) -> SignalBuilder {
        SignalBuilder::new(SignalType::Event, topic)
    }

    /// Create a Command signal builder.
    #[must_use]
    pub fn command(topic: &str) -> SignalBuilder {
        SignalBuilder::new(SignalType::Command, topic)
    }

    /// Create a Discovery signal builder.
    #[must_use]
    pub fn discovery() -> SignalBuilder {
        SignalBuilder::new(SignalType::Discovery, "discovery")
    }

    /// Create a Heartbeat signal builder.
    #[must_use]
    pub fn heartbeat() -> SignalBuilder {
        SignalBuilder::new(SignalType::Heartbeat, "heartbeat")
    }

    /// Validate this signal according to the NTL specification.
    pub fn validate(&self) -> crate::Result<()> {
        if !(0.0..=1.0).contains(&self.weight) {
            return Err(crate::Error::InvalidSignal(format!(
                "weight {} out of range [0.0, 1.0]",
                self.weight
            )));
        }

        if self.ttl == 0 {
            return Err(crate::Error::TtlExpired(self.id.to_string()));
        }

        if self.signature.is_empty() {
            return Err(crate::Error::InvalidSignal(
                "missing signature".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if this signal has exceeded its TTL.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.ttl == 0
    }

    /// Decrement TTL and add a node to the trace.
    pub fn hop(&mut self, node_id: NodeId) {
        self.ttl = self.ttl.saturating_sub(1);
        if self.trace.len() < 64 {
            self.trace.push(node_id);
        }
    }

    /// Attenuate the signal weight by a factor.
    pub fn attenuate(&mut self, factor: f32) {
        self.weight *= factor;
        self.weight = self.weight.clamp(0.0, 1.0);
    }

    /// Check if a node ID appears in the trace (loop detection).
    #[must_use]
    pub fn has_visited(&self, node_id: &NodeId) -> bool {
        self.trace.contains(node_id)
    }

    /// Get the wire format size estimate in bytes.
    #[must_use]
    pub fn estimated_size(&self) -> usize {
        // Header (8) + approximate CBOR body
        8 + 16 // id
          + self.origin.0.len()
          + self.signature.len()
          + 8  // timestamp
          + 4  // weight
          + 2  // ttl
          + self.payload.to_string().len()
          + self.trace.len() * 32
          + self.tags.iter().map(String::len).sum::<usize>()
    }

    /// Maximum allowed signal size in bytes.
    pub const MAX_SIZE: usize = 1_048_576; // 1 MB
}

impl SignalBuilder {
    fn new(signal_type: SignalType, topic: &str) -> Self {
        Self {
            signal_type,
            topic: topic.to_string(),
            payload: serde_json::Value::Null,
            weight: 0.5,
            ttl: 10,
            scope: PropagationScope::default(),
            correlation_id: None,
            tags: Vec::new(),
        }
    }

    /// Set the signal payload.
    #[must_use]
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Set the signal weight (0.0 - 1.0).
    #[must_use]
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Set the time-to-live in hops.
    #[must_use]
    pub fn with_ttl(mut self, ttl: u16) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set the propagation scope.
    #[must_use]
    pub fn with_scope(mut self, scope: PropagationScope) -> Self {
        self.scope = scope;
        self
    }

    /// Set a correlation ID for request-response patterns.
    #[must_use]
    pub fn with_correlation(mut self, id: SignalId) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Set searchable tags.
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(String::from).collect();
        self
    }

    /// Build the signal (without emitting — used for testing).
    ///
    /// Note: In production, use `.emit(&node)` instead, which handles
    /// signing and network emission.
    #[must_use]
    pub fn build_unsigned(self, origin: NodeId) -> Signal {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut tags = self.tags;
        if !self.topic.is_empty() {
            tags.insert(0, self.topic);
        }

        Signal {
            id: SignalId::new(),
            signal_type: self.signal_type,
            version: 1,
            origin,
            signature: Vec::new(), // Unsigned — must be signed before emission
            timestamp: now,
            payload: self.payload,
            encoding: Encoding::Cbor,
            weight: self.weight,
            ttl: self.ttl,
            scope: self.scope,
            correlation_id: self.correlation_id,
            trace: Vec::new(),
            tags,
        }
    }

    // TODO: pub async fn emit(self, node: &Node) -> crate::Result<Signal>
    // This will be implemented when Node is complete — it signs the signal
    // with the node's crypto module and emits it into the network.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_id_is_unique() {
        let a = SignalId::new();
        let b = SignalId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn signal_id_is_time_ordered() {
        let a = SignalId::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = SignalId::new();
        assert!(b.timestamp_ms() >= a.timestamp_ms());
    }

    #[test]
    fn signal_builder_defaults() {
        let origin = NodeId(vec![0u8; 32]);
        let signal = Signal::data("test").build_unsigned(origin);

        assert_eq!(signal.signal_type, SignalType::Data);
        assert!((signal.weight - 0.5).abs() < f32::EPSILON);
        assert_eq!(signal.ttl, 10);
        assert!(signal.tags.contains(&"test".to_string()));
    }

    #[test]
    fn signal_weight_clamping() {
        let origin = NodeId(vec![0u8; 32]);
        let signal = Signal::data("test")
            .with_weight(1.5)
            .build_unsigned(origin);
        assert!((signal.weight - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn signal_hop_decrements_ttl() {
        let origin = NodeId(vec![0u8; 32]);
        let mut signal = Signal::data("test")
            .with_ttl(5)
            .build_unsigned(origin);

        let hop_node = NodeId(vec![1u8; 32]);
        signal.hop(hop_node.clone());

        assert_eq!(signal.ttl, 4);
        assert!(signal.has_visited(&hop_node));
    }

    #[test]
    fn signal_attenuation() {
        let origin = NodeId(vec![0u8; 32]);
        let mut signal = Signal::data("test")
            .with_weight(1.0)
            .build_unsigned(origin);

        signal.attenuate(0.9);
        assert!((signal.weight - 0.9).abs() < f32::EPSILON);

        signal.attenuate(0.9);
        assert!((signal.weight - 0.81).abs() < 0.001);
    }

    #[test]
    fn signal_validation_rejects_invalid_weight() {
        let origin = NodeId(vec![0u8; 32]);
        let mut signal = Signal::data("test").build_unsigned(origin);
        signal.weight = 1.5;
        signal.signature = vec![1]; // Non-empty to pass sig check
        assert!(signal.validate().is_err());
    }

    #[test]
    fn signal_type_byte_roundtrip() {
        let types = vec![
            SignalType::Data,
            SignalType::Query,
            SignalType::Event,
            SignalType::Command,
            SignalType::Heartbeat,
            SignalType::Discovery,
            SignalType::Ack,
        ];
        for t in types {
            let byte = t.to_type_byte();
            let parsed = SignalType::from_type_byte(byte).unwrap();
            assert_eq!(t, parsed);
        }
    }

    #[test]
    fn signal_loop_detection() {
        let origin = NodeId(vec![0u8; 32]);
        let mut signal = Signal::data("test").build_unsigned(origin.clone());

        let node_a = NodeId(vec![1u8; 32]);
        let node_b = NodeId(vec![2u8; 32]);

        signal.hop(node_a.clone());
        signal.hop(node_b.clone());

        assert!(signal.has_visited(&node_a));
        assert!(signal.has_visited(&node_b));
        assert!(!signal.has_visited(&origin));
    }
}
