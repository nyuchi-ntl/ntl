//! Propagation engine for NTL signal routing.
//!
//! Determines how signals move through the synapse topology based on
//! relevance, weight, and activation patterns.

use serde::{Deserialize, Serialize};

use crate::signal::NodeId;
use crate::synapse::Synapse;

/// Propagation scope — determines how a signal routes through the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropagationScope {
    /// Propagate to all active synapses. Use sparingly.
    Flood {
        /// Maximum hops for flood propagation.
        max_hops: u16,
    },
    /// Propagate to highest-scoring synapses (default).
    Weighted {
        /// Minimum synapse weight to consider.
        min_synapse_weight: f32,
    },
    /// Directed toward a specific destination node.
    Targeted {
        /// The target node.
        destination: NodeId,
    },
    /// Follow the gradient of type affinity.
    Gradient {
        /// Signal type to follow affinity for.
        signal_type: String,
    },
}

impl Default for PropagationScope {
    fn default() -> Self {
        Self::Weighted {
            min_synapse_weight: 0.0,
        }
    }
}

/// Configuration for the propagation engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationConfig {
    /// Default propagation strategy.
    pub default_strategy: PropagationScope,
    /// Default TTL for signals.
    pub default_ttl: u16,
    /// Minimum signal weight to propagate.
    pub min_propagation_weight: f32,
    /// Default attenuation factor.
    pub attenuation_factor: f32,
    /// Maximum synapses to propagate to per hop.
    pub max_fanout: usize,
    /// Deduplication cache duration in seconds.
    pub dedup_cache_seconds: u64,
    /// Scoring weights.
    pub scoring: ScoringWeights,
}

impl Default for PropagationConfig {
    fn default() -> Self {
        Self {
            default_strategy: PropagationScope::default(),
            default_ttl: 10,
            min_propagation_weight: 0.01,
            attenuation_factor: 0.9,
            max_fanout: 5,
            dedup_cache_seconds: 300,
            scoring: ScoringWeights::default(),
        }
    }
}

/// Weights for the synapse scoring function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    /// Weight factor in scoring (default: 0.4).
    pub weight_factor: f32,
    /// Latency factor in scoring (default: 0.2).
    pub latency_factor: f32,
    /// Type affinity factor in scoring (default: 0.3).
    pub affinity_factor: f32,
    /// Recency factor in scoring (default: 0.1).
    pub recency_factor: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            weight_factor: 0.4,
            latency_factor: 0.2,
            affinity_factor: 0.3,
            recency_factor: 0.1,
        }
    }
}

/// Score a synapse for signal propagation.
///
/// Higher scores indicate better candidates for carrying a signal.
#[must_use]
pub fn score_synapse(
    synapse: &Synapse,
    signal_type: &str,
    now_ns: u64,
    weights: &ScoringWeights,
) -> f32 {
    let weight_score = synapse.weight;

    let latency_score = if synapse.avg_latency_ns == 0 {
        1.0
    } else {
        1.0 / (1.0 + (synapse.avg_latency_ns as f32 / 1_000_000.0)) // Normalize to ms
    };

    let affinity_score = synapse.affinity_for(signal_type);

    let hours_since_active = if now_ns > synapse.last_active {
        (now_ns - synapse.last_active) as f32 / 3_600_000_000_000.0
    } else {
        0.0
    };
    let recency_score = 1.0 / (1.0 + hours_since_active);

    (weight_score * weights.weight_factor)
        + (latency_score * weights.latency_factor)
        + (affinity_score * weights.affinity_factor)
        + (recency_score * weights.recency_factor)
}

/// Select synapses for propagation based on scope and scoring.
///
/// Returns synapses sorted by score descending, limited by max_fanout.
pub fn select_synapses<'a>(
    synapses: &'a [Synapse],
    scope: &PropagationScope,
    signal_type: &str,
    arrival_synapse: Option<&str>,
    config: &PropagationConfig,
) -> Vec<&'a Synapse> {
    let now_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let mut candidates: Vec<(&Synapse, f32)> = synapses
        .iter()
        .filter(|s| {
            // Don't propagate back on arrival synapse
            if let Some(arrival_id) = arrival_synapse {
                if s.id.0 == arrival_id {
                    return false;
                }
            }
            // Only active synapses
            s.state == crate::synapse::SynapseState::Active
        })
        .filter(|s| match scope {
            PropagationScope::Flood { .. } => true,
            PropagationScope::Weighted { min_synapse_weight } => s.weight >= *min_synapse_weight,
            PropagationScope::Targeted { destination } => {
                // For targeted, prefer the direct synapse if available
                s.remote_node == *destination || true // All are candidates for routing
            }
            PropagationScope::Gradient { .. } => true,
        })
        .map(|s| {
            let score = score_synapse(s, signal_type, now_ns, &config.scoring);
            (s, score)
        })
        .collect();

    // Sort by score descending
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Apply fanout limit
    let limit = match scope {
        PropagationScope::Flood { .. } => candidates.len(), // No limit for flood
        PropagationScope::Targeted { destination } => {
            // For targeted, if we have a direct synapse, just use that
            if let Some(direct) = candidates.iter().find(|(s, _)| s.remote_node == *destination) {
                return vec![direct.0];
            }
            1 // Otherwise, best single path
        }
        _ => config.max_fanout,
    };

    candidates.into_iter().take(limit).map(|(s, _)| s).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::NodeId;
    use crate::synapse::{Synapse, SynapseConfig, SynapseState};

    fn make_synapse(id: &str, weight: f32) -> Synapse {
        let local = NodeId(vec![0u8; 32]);
        let remote = NodeId(vec![id.as_bytes()[0]; 32]);
        let mut s = Synapse::new(local, remote, &SynapseConfig::default());
        s.id = crate::synapse::SynapseId(id.to_string());
        s.weight = weight;
        s.state = SynapseState::Active;
        s
    }

    #[test]
    fn select_weighted_respects_fanout() {
        let synapses: Vec<Synapse> = (0..10)
            .map(|i| make_synapse(&format!("syn-{i:03}"), 0.5))
            .collect();

        let config = PropagationConfig {
            max_fanout: 3,
            ..Default::default()
        };

        let scope = PropagationScope::Weighted {
            min_synapse_weight: 0.0,
        };

        let selected = select_synapses(&synapses, &scope, "data", None, &config);
        assert_eq!(selected.len(), 3);
    }

    #[test]
    fn select_excludes_arrival_synapse() {
        let synapses = vec![
            make_synapse("arrival", 0.9),
            make_synapse("other-1", 0.5),
            make_synapse("other-2", 0.3),
        ];

        let config = PropagationConfig::default();
        let scope = PropagationScope::default();

        let selected = select_synapses(&synapses, &scope, "data", Some("arrival"), &config);
        assert!(selected.iter().all(|s| s.id.0 != "arrival"));
    }

    #[test]
    fn higher_weight_scores_higher() {
        let weights = ScoringWeights::default();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let strong = make_synapse("strong", 0.9);
        let weak = make_synapse("weak", 0.1);

        let score_strong = score_synapse(&strong, "data", now, &weights);
        let score_weak = score_synapse(&weak, "data", now, &weights);

        assert!(score_strong > score_weak);
    }
}
