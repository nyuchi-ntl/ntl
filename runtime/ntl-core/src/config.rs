//! Configuration loading and management for NTL nodes.

use serde::{Deserialize, Serialize};

use crate::activation::ActivationConfig;
use crate::propagation::PropagationConfig;
use crate::synapse::SynapseConfig;

/// Complete node configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Network configuration.
    pub network: NetworkConfig,
    /// Synapse configuration.
    pub synapse: SynapseConfig,
    /// Propagation configuration.
    pub propagation: PropagationConfig,
    /// Activation configuration.
    pub activation: ActivationConfig,
    /// Crypto module to use.
    pub crypto_module: String,
}

/// Network-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bootstrap node addresses.
    pub bootstrap_nodes: Vec<String>,
    /// Address to bind to.
    pub bind_address: String,
    /// Port to listen on.
    pub port: u16,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                bootstrap_nodes: vec![
                    "ntl://bootstrap-1.nyuchi.com:4433".to_string(),
                    "ntl://bootstrap-2.nyuchi.com:4433".to_string(),
                ],
                bind_address: "0.0.0.0".to_string(),
                port: 4433,
            },
            synapse: SynapseConfig::default(),
            propagation: PropagationConfig::default(),
            activation: ActivationConfig::default(),
            crypto_module: "pq-v1".to_string(),
        }
    }
}

impl NodeConfig {
    /// Load configuration from a TOML file.
    pub fn from_file(path: &str) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::Error::Config(format!("Failed to read {path}: {e}")))?;

        toml::from_str(&content)
            .map_err(|e| crate::Error::Config(format!("Failed to parse config: {e}")))
    }

    /// Write configuration to a TOML file.
    pub fn to_file(&self, path: &str) -> crate::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::Error::Config(format!("Failed to serialize config: {e}")))?;

        std::fs::write(path, content)
            .map_err(|e| crate::Error::Config(format!("Failed to write {path}: {e}")))?;

        Ok(())
    }
}
