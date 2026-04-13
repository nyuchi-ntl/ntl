//! NTL Node — the primary runtime entry point.

use crate::activation::ActivationConfig;
use crate::config::NodeConfig;
use crate::propagation::PropagationConfig;
use crate::synapse::SynapseConfig;

/// An NTL node — a participant in the neural transfer network.
pub struct Node {
    config: NodeConfig,
    // TODO: identity, crypto module, synapse manager, propagation engine,
    //       activation state, signal handlers, adapters
}

/// Builder for constructing and configuring an NTL node.
pub struct NodeBuilder {
    config_path: Option<String>,
    crypto_module: Option<String>,
    bootstrap_nodes: Option<Vec<String>>,
    max_synapses: Option<u32>,
    synapse_config: SynapseConfig,
    propagation_config: PropagationConfig,
    activation_config: ActivationConfig,
}

impl Node {
    /// Create a new NodeBuilder.
    #[must_use]
    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }
}

impl NodeBuilder {
    fn new() -> Self {
        Self {
            config_path: None,
            crypto_module: None,
            bootstrap_nodes: None,
            max_synapses: None,
            synapse_config: SynapseConfig::default(),
            propagation_config: PropagationConfig::default(),
            activation_config: ActivationConfig::default(),
        }
    }

    /// Load configuration from a TOML file.
    #[must_use]
    pub fn with_config_file(mut self, path: &str) -> Self {
        self.config_path = Some(path.to_string());
        self
    }

    /// Set the cryptographic module to use.
    #[must_use]
    pub fn with_crypto_module(mut self, module: &str) -> Self {
        self.crypto_module = Some(module.to_string());
        self
    }

    /// Override bootstrap nodes.
    #[must_use]
    pub fn with_bootstrap(mut self, nodes: Vec<&str>) -> Self {
        self.bootstrap_nodes = Some(nodes.into_iter().map(String::from).collect());
        self
    }

    /// Set maximum synapse count.
    #[must_use]
    pub fn with_max_synapses(mut self, max: u32) -> Self {
        self.max_synapses = Some(max);
        self
    }

    /// Build the node (async — performs initialization).
    ///
    /// # Errors
    ///
    /// Returns an error if configuration is invalid or initialization fails.
    pub async fn build(self) -> crate::Result<Node> {
        let config = if let Some(path) = &self.config_path {
            NodeConfig::from_file(path)?
        } else {
            NodeConfig::default()
        };

        // TODO: Initialize crypto module, generate/load identity,
        //       set up synapse manager, propagation engine, activation state

        Ok(Node { config })
    }
}
