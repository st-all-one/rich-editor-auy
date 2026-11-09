//! `DocView` method kept separate to respect the file-length gate.

use rich_editor_auy::wire::view_json;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use super::{Editor, problem, problem_js};

#[wasm_bindgen]
impl Editor {
    /// Returns the render-oriented `DocView` projection.
    ///
    /// # Errors
    ///
    /// Returns a problem object only if the projection cannot be encoded.
    #[wasm_bindgen(js_name = viewJSON)]
    pub fn view_json(&self) -> Result<String, JsValue> {
        view_json(self.inner.doc(), self.inner.version(), &self.schema)
            .map_err(|_| problem_js(&problem::view_problem()))
    }
}
