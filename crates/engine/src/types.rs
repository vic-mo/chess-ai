use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineOptions {
    pub hash_size_mb: u32,
    pub threads: u32,
    pub contempt: Option<i32>,
    pub skill_level: Option<u32>,
    pub multi_pv: Option<u32>,
    pub use_tablebases: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SearchLimit {
    #[serde(rename = "depth")]
    Depth { depth: u32 },
    #[serde(rename = "nodes")]
    Nodes { nodes: u64 },
    #[serde(rename = "time")]
    Time { move_time_ms: u64 },
    #[serde(rename = "infinite")]
    Infinite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Score {
    #[serde(rename = "cp")]
    Cp { value: i32 },
    #[serde(rename = "mate")]
    Mate { plies: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInfo {
    pub id: String,
    pub depth: u32,
    pub seldepth: Option<u32>,
    pub nodes: u64,
    pub nps: u64,
    pub time_ms: u64,
    pub score: Score,
    pub pv: Vec<String>,
    pub hashfull: Option<u32>,
    pub tb_hits: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestMove {
    pub id: String,
    pub best: String,
    pub ponder: Option<String>,
}
