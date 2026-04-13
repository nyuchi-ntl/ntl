//! Synapse types and lifecycle management for NTL.
//!
//! A synapse is a persistent, weighted connection between two NTL nodes.
//! Synapses strengthen with use and weaken with inactivity.

use serde::{Deserialize, Serialize};

use crate::signal::NodeId;

/// Unique identifier for a synapse.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SynapseId(pub String);

impl std::fmt::Display for SynapseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "syn:{}", &self.0[..8.min(self.0.len())])
    }
}

/// The lifecycle state of a synapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynapseState {
    /// Handshake in progress.
    Forming,
    /// Actively transmitting signals.
    Active,
    /// Weight below active threshold, still connected.
    Weakening,
    /// Weight below dormancy threshold, connection idle.
    Dormant,
    /// Connection terminated, state archived.
    Pruned,
}

/// The underlying transport for a synapse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    /// QUIC — default, multiplexed, encrypted.
    Quic,
    /// TCP — fallback, widely supported.
    Tcp,
    /// Unix domain socket — same-machine nodes.
    Unix,
    /// Bluetooth Low Energy — proximity mesh.
    BluetoothLe,
    /// Application-defined transport.
    Custom(String),
}

impl Default for Transport {
    fn default() -> Self {
        Self::Quic
    }
}

/// An NTL synapse — a persistent, weighted connection between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    /// Unique synapse identifier.
    pub id: SynapseId,

    /// Local node in this synapse.
    pub local_node: NodeId,

    /// Remote node in this synapse.
    pub remote_node: NodeId,

    /// Current synapse weight (0.0 - 1.0).
    pub weight: f32,

    /// Current lifecycle state.
    pub state: SynapseState,

    /// Underlying transport mechanism.
    pub transport: Transport,

    /// Timestamp when synapse was established (ns since epoch).
    pub established_at: u64,

    /// Timestamp of last signal activity (ns since epoch).
    pub last_active: u64,

    /// Total signals transmitted through this synapse.
    pub signals_transmitted: u64,

    /// Total signals received through this synapse.
    pub signals_received: u64,

    /// Average round-trip latency in nanoseconds.
    pub avg_latency_ns: u64,

    /// Ratio of failed signal transmissions.
    pub error_rate: f32,

    /// Maximum weight this synapse can reach.
    pub max_weight: f32,

    /// Weight decay rate per decay interval.
    pub decay_rate: f32,

    /// Weight attenuation factor for signals passing through.
    pub attenuation_factor: f32,

    /// Historical affinity for signal types (type -> success count).
    pub type_affinity: std::collections::HashMap<String, u64>,
}

/// Configuration for synapse behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapseConfig {
    /// Initial weight for new synapses.
    pub initial_weight: f32,
    /// Maximum weight a synapse can reach.
    pub max_weight: f32,
    /// Weight decay rate per hour.
    pub decay_rate: f32,
    /// Weight threshold below which synapse becomes dormant.
    pub dormancy_threshold: f32,
    /// Hours in dormant state before pruning.
    pub prune_after_hours: u64,
    /// Maximum number of synapses per node.
    pub max_synapses: u32,
    /// Preferred transport.
    pub preferred_transport: Transport,
    /// Fallback transport.
    pub fallback_transport: Transport,
    /// Default attenuation factor.
    pub attenuation_factor: f32,
}

impl Default for SynapseConfig {
    fn default() -> Self {
        Self {
            initial_weight: 0.1,
            max_weight: 1.0,
            decay_rate: 0.01,
            dormancy_threshold: 0.01,
            prune_after_hours: 168, // 7 days
            max_synapses: 1000,
            preferred_transport: Transport::Quic,
            fallback_transport: Transport::Tcp,
            attenuation_factor: 0.9,
        }
    }
}

impl Synapse {
    /// Create a new synapse in the Forming state.
    #[must_use]
    pub fn new(local: NodeId, remote: NodeId, config: &SynapseConfig) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            id: SynapseId(ulid::Ulid::new().to_string()),
            local_node: local,
            remote_node: remote,
            weight: config.initial_weight,
            state: SynapseState::Forming,
            transport: config.preferred_transport.clone(),
            established_at: now,
            last_active: now,
            signals_transmitted: 0,
            signals_received: 0,
            avg_latency_ns: 0,
            error_rate: 0.0,
            max_weight: config.max_weight,
            decay_rate: config.decay_rate,
            attenuation_factor: config.attenuation_factor,
            type_affinity: std::collections::HashMap::new(),
        }
    }

    /// Strengthen the synapse after a successful signal transmission.
    pub fn strengthen(&mut self, signal_weight: f32, strengthen_factor: f32) {
        let delta = signal_weight * strengthen_factor;
        self.weight = (self.weight + delta).min(self.max_weight);
        self.update_state();
    }

    /// Weaken the synapse after a failed transmission.
    pub fn weaken_failure(&mut self) {
        self.weight *= 0.9;
        self.error_rate = (self.error_rate * 0.9) + 0.1;
        self.update_state();
    }

    /// Apply time-based weight decay.
    pub fn decay(&mut self) {
        self.weight *= 1.0 - self.decay_rate;
        self.update_state();
    }

    /// Record a successful signal transmission.
    pub fn record_transmission(&mut self, latency_ns: u64, signal_type: &str) {
        self.signals_transmitted += 1;
        self.last_active = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Running average latency
        if self.avg_latency_ns == 0 {
            self.avg_latency_ns = latency_ns;
        } else {
            self.avg_latency_ns = (self.avg_latency_ns * 9 + latency_ns) / 10;
        }

        // Update type affinity
        *self.type_affinity.entry(signal_type.to_string()).or_insert(0) += 1;

        self.error_rate *= 0.99; // Decay error rate on success
    }

    /// Record a received signal.
    pub fn record_reception(&mut self) {
        self.signals_received += 1;
        self.last_active = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
    }

    /// Get the affinity score for a specific signal type.
    #[must_use]
    pub fn affinity_for(&self, signal_type: &str) -> f32 {
        let total: u64 = self.type_affinity.values().sum();
        if total == 0 {
            return 0.0;
        }
        let count = self.type_affinity.get(signal_type).copied().unwrap_or(0);
        count as f32 / total as f32
    }

    /// Activate a dormant synapse.
    pub fn reactivate(&mut self, config: &SynapseConfig) {
        if self.state == SynapseState::Dormant {
            self.weight = config.initial_weight;
            self.state = SynapseState::Active;
        }
    }

    /// Update state based on current weight.
    fn update_state(&mut self) {
        if self.state == SynapseState::Pruned {
            return; // Terminal state
        }

        if self.weight >= 0.1 {
            self.state = SynapseState::Active;
        } else if self.weight >= 0.01 {
            self.state = SynapseState::Weakening;
        } else {
            self.state = SynapseState::Dormant;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SynapseConfig {
        SynapseConfig::default()
    }

    fn test_nodes() -> (NodeId, NodeId) {
        (NodeId(vec![0u8; 32]), NodeId(vec![1u8; 32]))
    }

    #[test]
    fn new_synapse_starts_forming() {
        let (local, remote) = test_nodes();
        let synapse = Synapse::new(local, remote, &test_config());
        assert_eq!(synapse.state, SynapseState::Forming);
        assert!((synapse.weight - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn strengthen_increases_weight() {
        let (local, remote) = test_nodes();
        let mut synapse = Synapse::new(local, remote, &test_config());
        synapse.state = SynapseState::Active;

        let initial = synapse.weight;
        synapse.strengthen(0.5, 0.01);
        assert!(synapse.weight > initial);
    }

    #[test]
    fn weight_respects_max() {
        let (local, remote) = test_nodes();
        let mut synapse = Synapse::new(local, remote, &test_config());
        synapse.state = SynapseState::Active;
        synapse.weight = 0.99;
        synapse.strengthen(1.0, 0.1);
        assert!(synapse.weight <= synapse.max_weight);
    }

    #[test]
    fn decay_reduces_weight() {
        let (local, remote) = test_nodes();
        let mut synapse = Synapse::new(local, remote, &test_config());
        synapse.state = SynapseState::Active;
        synapse.weight = 0.5;

        let initial = synapse.weight;
        synapse.decay();
        assert!(synapse.weight < initial);
    }

    #[test]
    fn state_transitions_on_weight() {
        let (local, remote) = test_nodes();
        let mut synapse = Synapse::new(local, remote, &test_config());
        synapse.state = SynapseState::Active;

        synapse.weight = 0.5;
        synapse.update_state();
        assert_eq!(synapse.state, SynapseState::Active);

        synapse.weight = 0.05;
        synapse.update_state();
        assert_eq!(synapse.state, SynapseState::Weakening);

        synapse.weight = 0.005;
        synapse.update_state();
        assert_eq!(synapse.state, SynapseState::Dormant);
    }

    #[test]
    fn type_affinity_tracking() {
        let (local, remote) = test_nodes();
        let mut synapse = Synapse::new(local, remote, &test_config());

        synapse.record_transmission(1000, "query");
        synapse.record_transmission(1000, "query");
        synapse.record_transmission(1000, "data");

        assert!((synapse.affinity_for("query") - 0.666).abs() < 0.01);
        assert!((synapse.affinity_for("data") - 0.333).abs() < 0.01);
        assert!((synapse.affinity_for("event") - 0.0).abs() < f32::EPSILON);
    }
}
