use engine::{
    types::{BestMove, EngineOptions, SearchInfo, SearchLimit},
    EngineImpl,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmEngine {
    inner: EngineImpl,
}

#[wasm_bindgen]
impl WasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(opts_js: JsValue) -> Result<WasmEngine, JsValue> {
        let opts: EngineOptions = serde_wasm_bindgen::from_value(opts_js)?;
        Ok(WasmEngine {
            inner: EngineImpl::new_with(opts),
        })
    }

    #[wasm_bindgen(js_name = "position")]
    pub fn position(&mut self, fen: String, moves_js: JsValue) -> Result<(), JsValue> {
        let moves: Vec<String> = serde_wasm_bindgen::from_value(moves_js)?;
        self.inner.position(&fen, &moves);
        Ok(())
    }

    #[wasm_bindgen(js_name = "analyze")]
    pub fn analyze(&mut self, limit_js: JsValue, cb: js_sys::Function) -> Result<JsValue, JsValue> {
        let limit: SearchLimit = serde_wasm_bindgen::from_value(limit_js)?;
        let mut closure_cb = |info: SearchInfo| {
            let _ = cb.call1(
                &JsValue::NULL,
                &serde_wasm_bindgen::to_value(&info).unwrap(),
            );
        };
        let best: BestMove = self.inner.analyze(limit, &mut closure_cb);
        Ok(serde_wasm_bindgen::to_value(&best)?)
    }

    #[wasm_bindgen(js_name = "stop")]
    pub fn stop(&mut self) {
        self.inner.stop();
    }
}
