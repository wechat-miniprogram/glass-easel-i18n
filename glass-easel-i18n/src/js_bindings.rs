use wasm_bindgen::prelude::*;

use crate::{compile, CompiledTemplate};

#[wasm_bindgen]
pub struct JsCompileResult(Result<CompiledTemplate, String>);

#[wasm_bindgen]
impl JsCompileResult {
    #[wasm_bindgen(js_name = "isSuccess")]
    pub fn success(&self) -> bool {
        match &self.0 {
            Ok(..) => true,
            Err(..) => false,
        }
    }

    #[wasm_bindgen(js_name = "getOutput")]
    pub fn output(&self) -> Option<String> {
        match &self.0 {
            Ok(CompiledTemplate { output, source_map: _ }) => Some(output.clone()),
            Err(_) => None,
        }
    }

    #[wasm_bindgen(js_name = "getSourceMap")]
    pub fn source_map(&self) -> Option<Vec<u8>> {
        match &self.0 {
            Ok(CompiledTemplate { output: _, source_map }) => Some(source_map.clone()),
            Err(_) => None,
        }
    }
}

#[wasm_bindgen(js_name = "compile")]
pub fn js_compile(path: &str, source: &str, trans_source: &str) -> JsCompileResult {
    let r = compile(path, source, trans_source);
    JsCompileResult(r)
}
