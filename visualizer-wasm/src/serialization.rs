use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Converts a Rust view model into a JavaScript value for the WASM boundary.
pub(super) fn serialize<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|error| JsValue::from_str(&error.to_string()))
}
