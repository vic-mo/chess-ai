use engine::EngineImpl;
use engine::types::{EngineOptions, SearchLimit, Score};

#[test]
fn test_mate_in_1() {
    let opts = EngineOptions {
        hash_size_mb: 16,
        threads: 1,
        contempt: None,
        skill_level: None,
        multi_pv: None,
        use_tablebases: None,
    };

    let mut eng = EngineImpl::new_with(opts);

    // Mate in 1: Ra8# is checkmate (back rank mate)
    let fen = "6k1/5ppp/8/8/8/8/8/R6K w - - 0 1";
    eng.position(fen, &[]);

    let mut search_infos = vec![];
    let result = eng.analyze(SearchLimit::Depth { depth: 8 }, |info| {
        println!("depth {} score {:?} pv {}", info.depth, info.score, info.pv.join(" "));
        search_infos.push(info);
    });

    println!("Best move: {}", result.best);
    println!("Expected: a1a8");

    // Check if we found mate
    let last_info = search_infos.last().unwrap();
    println!("Final score: {:?}", last_info.score);

    // The best move should be a1a8 (Ra8#)
    assert_eq!(result.best, "a1a8", "Engine should find Ra8# mate in 1");

    // Score should be mate
    match last_info.score {
        Score::Mate { plies } => {
            println!("Found mate in {} plies", plies.abs());
            assert!(plies.abs() <= 2, "Should find mate within 1 move (2 plies)");
        }
        Score::Cp { value } => {
            panic!("Expected mate score, got centipawn: {}", value);
        }
    }
}

#[test]
fn test_tactical_position() {
    let opts = EngineOptions {
        hash_size_mb: 16,
        threads: 1,
        contempt: None,
        skill_level: None,
        multi_pv: None,
        use_tablebases: None,
    };

    let mut eng = EngineImpl::new_with(opts);

    // Tactical position: Bxf7+ is a strong move
    let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1";
    eng.position(fen, &[]);

    let result = eng.analyze(SearchLimit::Depth { depth: 5 }, |info| {
        println!("depth {} score {:?} pv {}", info.depth, info.score, info.pv.join(" "));
    });

    println!("Best move: {}", result.best);

    // Should find Bxf7+ or similar tactical move
    assert!(
        result.best == "c4f7" || result.best == "f3e5" || result.best == "d2d4",
        "Should find a reasonable tactical move, got: {}",
        result.best
    );
}
