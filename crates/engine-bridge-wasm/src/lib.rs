use engine::{
    types::{BestMove, EngineOptions, SearchInfo, SearchLimit},
    EngineImpl,
};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

// Set panic hook for better error messages in WASM
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct WasmEngine {
    inner: RefCell<EngineImpl>,
}

#[wasm_bindgen]
impl WasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(opts_js: JsValue) -> Result<WasmEngine, JsValue> {
        let opts: EngineOptions = serde_wasm_bindgen::from_value(opts_js)?;
        Ok(WasmEngine {
            inner: RefCell::new(EngineImpl::new_with(opts)),
        })
    }

    #[wasm_bindgen(js_name = "position")]
    pub fn position(&self, fen: String, moves_js: JsValue) -> Result<(), JsValue> {
        let moves: Vec<String> = serde_wasm_bindgen::from_value(moves_js)?;
        self.inner.borrow_mut().position(&fen, &moves);
        Ok(())
    }

    #[wasm_bindgen(js_name = "analyze")]
    pub fn analyze(&self, limit_js: JsValue) -> Result<JsValue, JsValue> {
        let limit: SearchLimit = serde_wasm_bindgen::from_value(limit_js)?;

        // Use RefCell to avoid aliasing issues - borrow happens inside this scope
        let best: BestMove = self.inner.borrow_mut().analyze(limit, |_info: SearchInfo| {
            // No-op callback - SearchInfo streaming not supported in WASM
        });

        Ok(serde_wasm_bindgen::to_value(&best)?)
    }

    #[wasm_bindgen(js_name = "stop")]
    pub fn stop(&self) {
        self.inner.borrow_mut().stop();
    }

    // ========== Game-specific methods ==========

    /// Validate if a UCI move is legal in the given position
    #[wasm_bindgen(js_name = "isMoveLegal")]
    pub fn is_move_legal(&self, fen: &str, uci_move: &str) -> bool {
        self.inner.borrow().is_move_legal(fen, uci_move)
    }

    /// Apply a UCI move and return the new FEN
    #[wasm_bindgen(js_name = "makeMove")]
    pub fn make_move(&self, fen: &str, uci_move: &str) -> Result<String, JsValue> {
        self.inner
            .borrow_mut()
            .make_move(fen, uci_move)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get all legal moves for a position as UCI strings
    #[wasm_bindgen(js_name = "legalMoves")]
    pub fn legal_moves(&self, fen: &str) -> Vec<JsValue> {
        self.inner
            .borrow()
            .legal_moves(fen)
            .into_iter()
            .map(|s| JsValue::from_str(&s))
            .collect()
    }

    /// Check if position is game over (returns [is_over, status])
    /// Status can be "checkmate", "stalemate", or null if not over
    #[wasm_bindgen(js_name = "isGameOver")]
    pub fn is_game_over(&self, fen: &str) -> JsValue {
        let (is_over, status) = self.inner.borrow().is_game_over(fen);
        let result = serde_json::json!({
            "isOver": is_over,
            "status": status,
        });
        serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
    }
}
