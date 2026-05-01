use criterion::{criterion_group, criterion_main, Criterion};
use pdj_engine_bridge::{Engine, EngineConfig};

fn bench_control_latency(c: &mut Criterion) {
    let config = EngineConfig {
        sample_rate: 44100,
        buffer_size: 128,
        channel_count: 2,
    };
    
    // It's possible the engine fails to start in CI (no audio device),
    // but the engine stub works even if the device fails.
    let engine = Engine::new(config).unwrap_or_else(|_| {
        // Mock fallback if Engine::new absolutely required a device,
        // but `make_engine` in C++ allows null backend if needed.
        // Actually, Engine::new will fail if no device.
        // Let's just do a naive bench if it fails to load, skip it.
        panic!("Requires audio device to benchmark latency");
    });

    c.bench_function("engine_set_fader", |b| {
        let mut val = 0.0f32;
        b.iter(|| {
            engine.set_fader(0, val);
            val = if val > 0.9 { 0.0 } else { val + 0.1 };
        })
    });
}

criterion_group!(benches, bench_control_latency);
criterion_main!(benches);
