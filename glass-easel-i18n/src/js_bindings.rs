use wasm_bindgen::prelude::*;

use crate::{compile, search, CompiledTemplate, UntranslatedTerms};

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
            Ok(CompiledTemplate {
                output,
                source_map: _,
            }) => Some(output.clone()),
            Err(_) => None,
        }
    }

    #[wasm_bindgen(js_name = "getSourceMap")]
    pub fn source_map(&self) -> Option<Vec<u8>> {
        match &self.0 {
            Ok(CompiledTemplate {
                output: _,
                source_map,
            }) => Some(source_map.clone()),
            Err(_) => None,
        }
    }
}

#[wasm_bindgen(js_name = "compile")]
pub fn js_compile(
    path: &str,
    source: &str,
    trans_source: &str,
    attributes: Vec<String>,
) -> JsCompileResult {
    let r = compile(path, source, trans_source, &attributes);
    JsCompileResult(r)
}

#[wasm_bindgen]
pub struct JsUntranslatedTerms(Result<UntranslatedTerms, String>);

#[wasm_bindgen]
impl JsUntranslatedTerms {
    #[wasm_bindgen(js_name = "isSuccess")]
    pub fn success(&self) -> bool {
        match &self.0 {
            Ok(..) => true,
            Err(..) => false,
        }
    }

    #[wasm_bindgen(js_name = "getOutput")]
    pub fn output(&self) -> Option<Vec<String>> {
        match &self.0 {
            Ok(UntranslatedTerms { output }) => Some(output.clone()),
            Err(_) => None,
        }
    }
}

#[wasm_bindgen(js_name = "search")]
pub fn js_search(path: &str, source: &str, attributes: Vec<String>) -> JsUntranslatedTerms {
    let r = search(path, source, &attributes);
    JsUntranslatedTerms(r)
}
