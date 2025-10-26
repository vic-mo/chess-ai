// Quick test to verify mate detection in native engine
use engine::EngineImpl;
use engine::types::{EngineOptions, SearchLimit};

fn main() {
    let opts = EngineOptions {
        hash_size_mb: 16,
        threads: 1,
        contempt: None,
        skill_level: None,
        multi_pv: None,
        use_tablebases: None,
    };

    let mut eng = EngineImpl::new_with(opts);

    // Test 1: Mate in 1 - Qe8#
    println!("\n=== Test 1: Mate in 1 ===");
    let fen1 = "r5k1/5ppp/8/8/8/8/5PPP/4Q1K1 w - - 0 1";
    eng.position(fen1, &[]);

    let result1 = eng.analyze(SearchLimit::Depth { depth: 8 }, |info| {
        println!("depth {} score {:?} pv {}", info.depth, info.score, info.pv.join(" "));
    });

    println!("Best move: {}", result1.best);
    println!("Expected: e1e8");
    println!("Match: {}", result1.best == "e1e8");

    // Test 2: Starting position
    println!("\n=== Test 2: Starting position ===");
    eng.position("startpos", &[]);

    let result2 = eng.analyze(SearchLimit::Depth { depth: 5 }, |info| {
        println!("depth {} score {:?} pv {}", info.depth, info.score, info.pv.join(" "));
    });

    println!("Best move: {}", result2.best);

    // Test 3: Tactical position (Scholar's Mate setup)
    println!("\n=== Test 3: Tactical position ===");
    let fen3 = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1";
    eng.position(fen3, &[]);

    let result3 = eng.analyze(SearchLimit::Depth { depth: 5 }, |info| {
        println!("depth {} score {:?} pv {}", info.depth, info.score, info.pv.join(" "));
    });

    println!("Best move: {}", result3.best);
    println!("Expected: c4f7 (Bxf7+ - fork)");
}
