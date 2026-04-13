//! Network topology management and discovery.

use crate::signal::NodeId;

/// Health metrics for the local topology.
#[derive(Debug, Clone)]
pub struct TopologyHealth {
    /// Number of active synapses.
    pub active_synapses: u32,
    /// Average synapse weight.
    pub avg_synapse_weight: f32,
    /// Connectivity score (0.0 - 1.0).
    pub connectivity_score: f32,
    /// Redundancy score — alternative paths available.
    pub redundancy_score: f32,
    /// Timestamp of last bootstrap node contact.
    pub last_bootstrap_contact: u64,
}

/// Node capability announcement for discovery signals.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeCapabilities {
    /// Signal types this node handles.
    pub signal_types: Vec<String>,
    /// Adapters available on this node.
    pub adapters: Vec<String>,
    /// Node capacity tier.
    pub capacity: NodeCapacity,
    /// Geographic region hint.
    pub region: Option<String>,
}

/// Node capacity tier.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum NodeCapacity {
    /// Edge node — lightweight, limited resources.
    Edge,
    /// Standard node.
    Standard,
    /// High-capacity node.
    High,
    /// Bootstrap/infrastructure node.
    Infrastructure,
}

/// Placeholder for topology manager.
///
/// Will manage synapse topology, discovery, and health monitoring.
pub struct TopologyManager {
    local_node: NodeId,
    bootstrap_nodes: Vec<String>,
}

impl TopologyManager {
    /// Create a new topology manager.
    #[must_use]
    pub fn new(local_node: NodeId, bootstrap_nodes: Vec<String>) -> Self {
        Self {
            local_node,
            bootstrap_nodes,
        }
    }
}
