//! Test utilities and mocks for NTL development.

use crate::signal::{NodeId, Signal, SignalType};

/// Create a test NodeId with a simple pattern.
#[must_use]
pub fn test_node_id(id: u8) -> NodeId {
    NodeId(vec![id; 32])
}

/// Create a simple unsigned test signal.
#[must_use]
pub fn test_signal(signal_type: SignalType, weight: f32) -> Signal {
    let builder = match signal_type {
        SignalType::Data => Signal::data("test"),
        SignalType::Query => Signal::query("test"),
        SignalType::Event => Signal::event("test"),
        SignalType::Command => Signal::command("test"),
        SignalType::Discovery => Signal::discovery(),
        SignalType::Heartbeat => Signal::heartbeat(),
        _ => Signal::data("test"),
    };

    let mut signal = builder.with_weight(weight).build_unsigned(test_node_id(0));
    signal.signature = vec![0u8; 64]; // Fake signature for testing
    signal
}
