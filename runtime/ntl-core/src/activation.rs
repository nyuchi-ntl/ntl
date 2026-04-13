//! Activation model for NTL nodes.
//!
//! Replaces traditional rate limiting with biologically-inspired
//! threshold-based signal processing.

use serde::{Deserialize, Serialize};

/// Activation function type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivationFunction {
    /// Binary: fires when potential >= threshold.
    Step,
    /// Probabilistic: firing probability increases smoothly.
    Sigmoid,
    /// Always passes a small fraction (leak rate = 0.01).
    Leaky,
}

impl Default for ActivationFunction {
    fn default() -> Self {
        Self::Step
    }
}

/// Configuration for the activation model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationConfig {
    /// Base activation threshold.
    pub base_threshold: f32,
    /// Activation function to use.
    pub activation_function: ActivationFunction,
    /// Refractory period in milliseconds.
    pub refractory_period_ms: u64,
    /// Maximum accumulated potential (prevents overflow).
    pub max_potential: f32,
    /// Whether to dynamically adjust threshold based on load.
    pub dynamic_threshold: bool,
}

impl Default for ActivationConfig {
    fn default() -> Self {
        Self {
            base_threshold: 0.5,
            activation_function: ActivationFunction::Step,
            refractory_period_ms: 10,
            max_potential: 10.0,
            dynamic_threshold: true,
        }
    }
}

/// Activation state for a node.
#[derive(Debug)]
pub struct ActivationState {
    /// Current accumulated potential.
    potential: f32,
    /// Current effective threshold.
    threshold: f32,
    /// Base threshold (before dynamic adjustment).
    base_threshold: f32,
    /// Nanosecond timestamp when refractory period ends.
    refractory_until: u64,
    /// Refractory duration in nanoseconds.
    refractory_period_ns: u64,
    /// Activation function.
    function: ActivationFunction,
    /// Maximum potential.
    max_potential: f32,
    /// Whether dynamic threshold is enabled.
    dynamic: bool,
    /// Signals processed (for stats).
    signals_fired: u64,
    /// Signals accumulated but not yet fired.
    signals_accumulated: u64,
}

impl ActivationState {
    /// Create a new activation state from configuration.
    #[must_use]
    pub fn new(config: &ActivationConfig) -> Self {
        Self {
            potential: 0.0,
            threshold: config.base_threshold,
            base_threshold: config.base_threshold,
            refractory_until: 0,
            refractory_period_ns: config.refractory_period_ms * 1_000_000,
            function: config.activation_function,
            max_potential: config.max_potential,
            dynamic: config.dynamic_threshold,
            signals_fired: 0,
            signals_accumulated: 0,
        }
    }

    /// Add a signal's contribution to the activation potential.
    ///
    /// Returns `true` if the node fires (should process the signal).
    pub fn accumulate(&mut self, signal_weight: f32, synapse_weight: f32) -> bool {
        let contribution = signal_weight * synapse_weight;
        self.potential = (self.potential + contribution).min(self.max_potential);
        self.signals_accumulated += 1;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        if self.in_refractory(now) {
            return false;
        }

        let should_fire = self.evaluate();

        if should_fire {
            self.fire(now);
        }

        should_fire
    }

    /// Evaluate the activation function.
    fn evaluate(&self) -> bool {
        match self.function {
            ActivationFunction::Step => self.potential >= self.threshold,
            ActivationFunction::Sigmoid => {
                let probability = 1.0 / (1.0 + (-10.0 * (self.potential - self.threshold)).exp());
                rand::random::<f32>() < probability
            }
            ActivationFunction::Leaky => {
                if self.potential >= self.threshold {
                    true
                } else {
                    rand::random::<f32>() < 0.01
                }
            }
        }
    }

    /// Fire the activation gate.
    fn fire(&mut self, now_ns: u64) {
        self.potential = 0.0;
        self.refractory_until = now_ns + self.refractory_period_ns;
        self.signals_fired += 1;
    }

    /// Check if the node is in its refractory period.
    fn in_refractory(&self, now_ns: u64) -> bool {
        now_ns < self.refractory_until
    }

    /// Adjust the threshold based on current load.
    pub fn adjust_for_load(&mut self, load_factor: f32) {
        if self.dynamic {
            self.threshold = self.base_threshold * (1.0 + load_factor);
        }
    }

    /// Get the current potential.
    #[must_use]
    pub fn potential(&self) -> f32 {
        self.potential
    }

    /// Get the current effective threshold.
    #[must_use]
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Get the number of times this gate has fired.
    #[must_use]
    pub fn fire_count(&self) -> u64 {
        self.signals_fired
    }

    /// Reset the activation state.
    pub fn reset(&mut self) {
        self.potential = 0.0;
        self.threshold = self.base_threshold;
        self.refractory_until = 0;
        self.signals_fired = 0;
        self.signals_accumulated = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ActivationConfig {
        ActivationConfig {
            base_threshold: 0.5,
            activation_function: ActivationFunction::Step,
            refractory_period_ms: 0, // Disable for testing
            max_potential: 10.0,
            dynamic_threshold: false,
        }
    }

    #[test]
    fn step_fires_above_threshold() {
        let mut state = ActivationState::new(&test_config());

        // Below threshold
        let fired = state.accumulate(0.3, 1.0);
        assert!(!fired);
        assert!((state.potential - 0.3).abs() < f32::EPSILON);

        // Above threshold (0.3 + 0.3 = 0.6 >= 0.5)
        let fired = state.accumulate(0.3, 1.0);
        assert!(fired);
        assert!((state.potential - 0.0).abs() < f32::EPSILON); // Reset after firing
    }

    #[test]
    fn synapse_weight_modulates_contribution() {
        let mut state = ActivationState::new(&test_config());

        // Signal weight 1.0 but synapse weight 0.1 = contribution 0.1
        let fired = state.accumulate(1.0, 0.1);
        assert!(!fired);
        assert!((state.potential - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn potential_capped_at_max() {
        let mut state = ActivationState::new(&ActivationConfig {
            base_threshold: 100.0, // High threshold so it won't fire
            refractory_period_ms: 0,
            ..test_config()
        });

        for _ in 0..100 {
            state.accumulate(1.0, 1.0);
        }

        assert!(state.potential <= state.max_potential);
    }

    #[test]
    fn dynamic_threshold_increases_with_load() {
        let mut state = ActivationState::new(&ActivationConfig {
            dynamic_threshold: true,
            ..test_config()
        });

        state.adjust_for_load(1.0); // 100% load
        assert!((state.threshold - 1.0).abs() < f32::EPSILON); // 0.5 * (1 + 1.0) = 1.0
    }

    #[test]
    fn fire_count_tracks() {
        let mut state = ActivationState::new(&test_config());

        state.accumulate(1.0, 1.0); // Should fire
        state.accumulate(1.0, 1.0); // Should fire again
        state.accumulate(1.0, 1.0); // Should fire again

        assert_eq!(state.fire_count(), 3);
    }
}
