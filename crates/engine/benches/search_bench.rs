use criterion::{criterion_group, criterion_main, Criterion};
use engine::{
    types::{EngineOptions, SearchLimit},
    EngineImpl,
};

fn bench_iterative(c: &mut Criterion) {
    c.bench_function("engine_iterative_depth_4", |b| {
        b.iter(|| {
            let mut eng = EngineImpl::new_with(EngineOptions {
                hash_size_mb: 64,
                threads: 1,
                contempt: None,
                skill_level: None,
                multi_pv: Some(1),
                use_tablebases: None,
            });
            eng.position("startpos", &[]);
            let _ = eng.analyze(SearchLimit::Depth { depth: 4 }, |_| {});
        });
    });
}

criterion_group!(benches, bench_iterative);
criterion_main!(benches);
