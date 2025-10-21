use engine::{
    types::{EngineOptions, SearchLimit},
    EngineImpl,
};

#[test]
fn smoke_analyze() {
    let mut eng = EngineImpl::new_with(EngineOptions {
        hash_size_mb: 64,
        threads: 1,
        contempt: None,
        skill_level: None,
        multi_pv: Some(1),
        use_tablebases: None,
    });
    eng.position("startpos", &[]);
    let mut infos = vec![];
    let best = eng.analyze(SearchLimit::Depth { depth: 3 }, |i| infos.push(i));
    assert_eq!(best.best, "e2e4");
    assert!(!infos.is_empty());
}
