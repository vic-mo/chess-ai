use engine::types::*;

#[test]
fn score_cp_roundtrip() {
    let original = Score::Cp { value: 150 };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: Score = serde_json::from_str(&json).unwrap();

    assert_eq!(json, r#"{"kind":"cp","value":150}"#);
    match parsed {
        Score::Cp { value } => assert_eq!(value, 150),
        _ => panic!("Expected Cp score"),
    }
}

#[test]
fn score_mate_roundtrip() {
    let original = Score::Mate { plies: 5 };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: Score = serde_json::from_str(&json).unwrap();

    assert_eq!(json, r#"{"kind":"mate","plies":5}"#);
    match parsed {
        Score::Mate { plies } => assert_eq!(plies, 5),
        _ => panic!("Expected Mate score"),
    }
}

#[test]
fn search_limit_depth_roundtrip() {
    let original = SearchLimit::Depth { depth: 10 };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: SearchLimit = serde_json::from_str(&json).unwrap();

    assert_eq!(json, r#"{"kind":"depth","depth":10}"#);
    match parsed {
        SearchLimit::Depth { depth } => assert_eq!(depth, 10),
        _ => panic!("Expected Depth limit"),
    }
}

#[test]
fn search_limit_nodes_roundtrip() {
    let original = SearchLimit::Nodes { nodes: 1000000 };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: SearchLimit = serde_json::from_str(&json).unwrap();

    assert_eq!(json, r#"{"kind":"nodes","nodes":1000000}"#);
    match parsed {
        SearchLimit::Nodes { nodes } => assert_eq!(nodes, 1000000),
        _ => panic!("Expected Nodes limit"),
    }
}

#[test]
fn search_limit_time_roundtrip() {
    let original = SearchLimit::Time { move_time_ms: 5000 };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: SearchLimit = serde_json::from_str(&json).unwrap();

    // Verify camelCase on wire
    assert_eq!(json, r#"{"kind":"time","moveTimeMs":5000}"#);
    match parsed {
        SearchLimit::Time { move_time_ms } => assert_eq!(move_time_ms, 5000),
        _ => panic!("Expected Time limit"),
    }
}

#[test]
fn search_limit_infinite_roundtrip() {
    let original = SearchLimit::Infinite;
    let json = serde_json::to_string(&original).unwrap();
    let parsed: SearchLimit = serde_json::from_str(&json).unwrap();

    assert_eq!(json, r#"{"kind":"infinite"}"#);
    assert!(matches!(parsed, SearchLimit::Infinite));
}

#[test]
fn engine_options_roundtrip() {
    let original = EngineOptions {
        hash_size_mb: 128,
        threads: 4,
        contempt: Some(10),
        skill_level: Some(15),
        multi_pv: Some(3),
        use_tablebases: Some(true),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: EngineOptions = serde_json::from_str(&json).unwrap();

    // Verify camelCase on wire
    assert!(json.contains("hashSizeMB"));
    assert!(json.contains("skillLevel"));
    assert!(json.contains("multiPV"));
    assert!(json.contains("useTablebases"));

    assert_eq!(parsed.hash_size_mb, 128);
    assert_eq!(parsed.threads, 4);
    assert_eq!(parsed.contempt, Some(10));
    assert_eq!(parsed.skill_level, Some(15));
    assert_eq!(parsed.multi_pv, Some(3));
    assert_eq!(parsed.use_tablebases, Some(true));
}

#[test]
fn engine_options_minimal_roundtrip() {
    let original = EngineOptions {
        hash_size_mb: 64,
        threads: 1,
        contempt: None,
        skill_level: None,
        multi_pv: None,
        use_tablebases: None,
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: EngineOptions = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.hash_size_mb, 64);
    assert_eq!(parsed.threads, 1);
    assert_eq!(parsed.contempt, None);
}

#[test]
fn analyze_request_roundtrip() {
    let original = AnalyzeRequest {
        id: "test-123".to_string(),
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        moves: Some(vec!["e2e4".to_string(), "e7e5".to_string()]),
        limit: SearchLimit::Depth { depth: 10 },
        options: Some(EngineOptions {
            hash_size_mb: 64,
            threads: 1,
            contempt: None,
            skill_level: None,
            multi_pv: Some(1),
            use_tablebases: None,
        }),
        context: Some(AnalyzeRequestContext {
            allow_ponder: Some(true),
        }),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: AnalyzeRequest = serde_json::from_str(&json).unwrap();

    // Verify camelCase for context fields
    assert!(json.contains("allowPonder"));

    assert_eq!(parsed.id, "test-123");
    assert_eq!(
        parsed.fen,
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
    assert_eq!(
        parsed.moves,
        Some(vec!["e2e4".to_string(), "e7e5".to_string()])
    );
}

#[test]
fn analyze_request_minimal_roundtrip() {
    let original = AnalyzeRequest {
        id: "minimal".to_string(),
        fen: "startpos".to_string(),
        moves: None,
        limit: SearchLimit::Infinite,
        options: None,
        context: None,
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: AnalyzeRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.id, "minimal");
    assert_eq!(parsed.fen, "startpos");
    assert_eq!(parsed.moves, None);
}

#[test]
fn search_info_roundtrip() {
    let original = SearchInfo {
        id: "info-1".to_string(),
        depth: 10,
        seldepth: Some(12),
        nodes: 1000000,
        nps: 500000,
        time_ms: 2000,
        score: Score::Cp { value: 50 },
        pv: vec!["e2e4".to_string(), "e7e5".to_string()],
        hashfull: Some(500),
        tb_hits: Some(100),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: SearchInfo = serde_json::from_str(&json).unwrap();

    // Verify camelCase on wire
    assert!(json.contains("timeMs"));
    assert!(json.contains("tbHits"));

    assert_eq!(parsed.id, "info-1");
    assert_eq!(parsed.depth, 10);
    assert_eq!(parsed.time_ms, 2000);
    assert_eq!(parsed.tb_hits, Some(100));
}

#[test]
fn best_move_roundtrip() {
    let original = BestMove {
        id: "move-1".to_string(),
        best: "e2e4".to_string(),
        ponder: Some("e7e5".to_string()),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: BestMove = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.id, "move-1");
    assert_eq!(parsed.best, "e2e4");
    assert_eq!(parsed.ponder, Some("e7e5".to_string()));
}

#[test]
fn engine_event_search_info_roundtrip() {
    let original = EngineEvent::SearchInfo {
        payload: SearchInfo {
            id: "evt-1".to_string(),
            depth: 5,
            seldepth: None,
            nodes: 10000,
            nps: 50000,
            time_ms: 200,
            score: Score::Cp { value: 25 },
            pv: vec!["e2e4".to_string()],
            hashfull: None,
            tb_hits: None,
        },
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: EngineEvent = serde_json::from_str(&json).unwrap();

    // Verify event structure
    assert!(json.contains(r#""type":"searchInfo""#));
    assert!(json.contains(r#""payload":"#));

    match parsed {
        EngineEvent::SearchInfo { payload } => {
            assert_eq!(payload.id, "evt-1");
            assert_eq!(payload.depth, 5);
        }
        _ => panic!("Expected SearchInfo event"),
    }
}

#[test]
fn engine_event_best_move_roundtrip() {
    let original = EngineEvent::BestMove {
        payload: BestMove {
            id: "evt-2".to_string(),
            best: "d2d4".to_string(),
            ponder: Some("d7d5".to_string()),
        },
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: EngineEvent = serde_json::from_str(&json).unwrap();

    assert!(json.contains(r#""type":"bestMove""#));

    match parsed {
        EngineEvent::BestMove { payload } => {
            assert_eq!(payload.id, "evt-2");
            assert_eq!(payload.best, "d2d4");
        }
        _ => panic!("Expected BestMove event"),
    }
}

#[test]
fn engine_event_error_roundtrip() {
    let original = EngineEvent::Error {
        payload: ErrorPayload {
            id: "evt-3".to_string(),
            message: "Invalid FEN".to_string(),
        },
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: EngineEvent = serde_json::from_str(&json).unwrap();

    assert!(json.contains(r#""type":"error""#));

    match parsed {
        EngineEvent::Error { payload } => {
            assert_eq!(payload.id, "evt-3");
            assert_eq!(payload.message, "Invalid FEN");
        }
        _ => panic!("Expected Error event"),
    }
}

#[test]
fn typescript_json_compatibility_search_info() {
    // JSON from TypeScript with camelCase
    let ts_json = r#"{
        "id": "ts-1",
        "depth": 8,
        "seldepth": 10,
        "nodes": 250000,
        "nps": 125000,
        "timeMs": 2000,
        "score": {"kind": "cp", "value": 42},
        "pv": ["e2e4", "c7c5"],
        "hashfull": 300,
        "tbHits": 5
    }"#;

    let parsed: SearchInfo = serde_json::from_str(ts_json).unwrap();

    assert_eq!(parsed.id, "ts-1");
    assert_eq!(parsed.depth, 8);
    assert_eq!(parsed.time_ms, 2000);
    assert_eq!(parsed.tb_hits, Some(5));
}

#[test]
fn typescript_json_compatibility_engine_event() {
    // JSON from TypeScript
    let ts_json = r#"{
        "type": "searchInfo",
        "payload": {
            "id": "ts-evt",
            "depth": 3,
            "nodes": 1000,
            "nps": 10000,
            "timeMs": 100,
            "score": {"kind": "mate", "plies": 3},
            "pv": ["f7f8q"]
        }
    }"#;

    let parsed: EngineEvent = serde_json::from_str(ts_json).unwrap();

    match parsed {
        EngineEvent::SearchInfo { payload } => {
            assert_eq!(payload.id, "ts-evt");
            assert_eq!(payload.depth, 3);
            match payload.score {
                Score::Mate { plies } => assert_eq!(plies, 3),
                _ => panic!("Expected mate score"),
            }
        }
        _ => panic!("Expected SearchInfo event"),
    }
}

#[test]
fn typescript_json_compatibility_analyze_request() {
    // JSON from TypeScript
    let ts_json = r#"{
        "id": "req-ts",
        "fen": "startpos",
        "moves": ["e2e4"],
        "limit": {"kind": "depth", "depth": 15},
        "options": {
            "hashSizeMB": 256,
            "threads": 8,
            "skillLevel": 20,
            "multiPV": 2
        },
        "context": {
            "allowPonder": true
        }
    }"#;

    let parsed: AnalyzeRequest = serde_json::from_str(ts_json).unwrap();

    assert_eq!(parsed.id, "req-ts");
    assert_eq!(parsed.fen, "startpos");
    assert_eq!(parsed.moves, Some(vec!["e2e4".to_string()]));

    let options = parsed.options.unwrap();
    assert_eq!(options.hash_size_mb, 256);
    assert_eq!(options.threads, 8);
    assert_eq!(options.skill_level, Some(20));
    assert_eq!(options.multi_pv, Some(2));

    let context = parsed.context.unwrap();
    assert_eq!(context.allow_ponder, Some(true));
}
