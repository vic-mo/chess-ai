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

    // Real engine should return a valid move (not "0000" error indicator)
    assert_ne!(best.best, "0000", "Engine returned error");

    // Should be a legal move in UCI format (4 or 5 chars)
    assert!(
        best.best.len() == 4 || best.best.len() == 5,
        "Move {} is not valid UCI format",
        best.best
    );

    // SearchInfo streaming should work (Session 2)
    assert_eq!(infos.len(), 3, "Should receive SearchInfo for depths 1, 2, 3");

    // Verify SearchInfo fields are populated
    for (i, info) in infos.iter().enumerate() {
        assert_eq!(info.depth, (i + 1) as u32);
        assert!(info.nodes > 0, "Nodes should be > 0");
        assert!(info.nps > 0, "NPS should be > 0");
        assert!(!info.pv.is_empty(), "PV should not be empty");
    }
}
