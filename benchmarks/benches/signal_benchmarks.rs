use criterion::{criterion_group, criterion_main, Criterion};
use ntl_core::signal::{Signal, NodeId};

fn signal_creation(c: &mut Criterion) {
    let origin = NodeId(vec![0u8; 32]);

    c.bench_function("create_signal", |b| {
        b.iter(|| {
            Signal::data("benchmark")
                .with_payload(serde_json::json!({"key": "value"}))
                .with_weight(0.5)
                .build_unsigned(origin.clone())
        })
    });
}

fn signal_validation(c: &mut Criterion) {
    let origin = NodeId(vec![0u8; 32]);
    let mut signal = Signal::data("benchmark")
        .with_weight(0.5)
        .build_unsigned(origin);
    signal.signature = vec![0u8; 64];

    c.bench_function("validate_signal", |b| {
        b.iter(|| signal.validate())
    });
}

criterion_group!(benches, signal_creation, signal_validation);
criterion_main!(benches);
