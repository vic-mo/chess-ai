use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineOptions {
    #[serde(rename = "hashSizeMB")]
    pub hash_size_mb: u32,
    pub threads: u32,
    pub contempt: Option<i32>,
    pub skill_level: Option<u32>,
    #[serde(rename = "multiPV")]
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
    #[serde(rename = "time", rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct SearchInfo {
    pub id: String,
    pub depth: u32,
    pub seldepth: Option<u32>,
    pub nodes: u64,
    pub nps: u64,
    #[serde(rename = "timeMs")]
    pub time_ms: u64,
    pub score: Score,
    pub pv: Vec<String>,
    pub hashfull: Option<u32>,
    #[serde(rename = "tbHits")]
    pub tb_hits: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestMove {
    pub id: String,
    pub best: String,
    pub ponder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeRequestContext {
    pub allow_ponder: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    pub id: String,
    pub fen: String,
    pub moves: Option<Vec<String>>,
    pub limit: SearchLimit,
    pub options: Option<EngineOptions>,
    pub context: Option<AnalyzeRequestContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EngineEvent {
    #[serde(rename = "searchInfo")]
    SearchInfo { payload: SearchInfo },
    #[serde(rename = "bestMove")]
    BestMove { payload: BestMove },
    #[serde(rename = "error")]
    Error { payload: ErrorPayload },
}
