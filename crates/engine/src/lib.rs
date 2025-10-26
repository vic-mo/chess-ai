pub mod attacks;
pub mod bitboard;
pub mod board;
pub mod io;
#[allow(clippy::module_inception)]
pub mod r#move;
pub mod movelist;
pub mod piece;
pub mod square;
pub mod types;

use types::*;

pub struct EngineImpl {
    pub opts: EngineOptions,
    pub current_fen: String,
    stopped: bool,
}

impl Default for EngineImpl {
    fn default() -> Self {
        Self {
            opts: EngineOptions {
                hash_size_mb: 64,
                threads: 1,
                contempt: None,
                skill_level: None,
                multi_pv: Some(1),
                use_tablebases: None,
            },
            current_fen: "startpos".to_string(),
            stopped: false,
        }
    }
}

impl EngineImpl {
    pub fn new_with(opts: EngineOptions) -> Self {
        Self {
            opts,
            ..Default::default()
        }
    }

    pub fn new_game(&mut self) {
        self.current_fen = "startpos".to_string();
        self.stopped = false;
    }

    pub fn position(&mut self, fen: &str, _moves: &[String]) {
        self.current_fen = fen.to_string();
    }

    pub fn set_option(&mut self, _key: &str, _value: &str) {
        // TODO: parse key/value into opts
    }

    pub fn analyze<F>(&mut self, limit: SearchLimit, mut info_sink: F) -> BestMove
    where
        F: FnMut(SearchInfo),
    {
        self.stopped = false;
        // Dummy iterative deepening loop for scaffold
        let mut nodes = 0u64;
        for depth in 1..=match limit {
            SearchLimit::Depth { depth } => depth,
            _ => 6,
        } {
            if self.stopped {
                break;
            }
            nodes += (depth as u64) * 1000;
            let info = SearchInfo {
                id: "scaffold".into(),
                depth,
                seldepth: Some(depth + 2),
                nodes,
                nps: 1200000,
                time_ms: depth as u64 * 50,
                score: Score::Cp {
                    value: depth as i32 * 10,
                },
                pv: vec!["e2e4".into(), "e7e5".into()],
                hashfull: Some(10 * depth),
                tb_hits: None,
            };
            info_sink(info);
        }
        BestMove {
            id: "scaffold".into(),
            best: "e2e4".into(),
            ponder: Some("e7e5".into()),
        }
    }

    pub fn stop(&mut self) {
        self.stopped = true;
    }
}
