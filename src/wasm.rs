use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn sort_json_string(original: &str) -> Result<String, String> {
    crate::sort_json_string(original).map_err(|error| error.to_string())
}

#[wasm_bindgen]
pub fn json_sort_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
