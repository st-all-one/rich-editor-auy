//! Attribute, identifier, selection, and origin setters for transactions.

use serde_json::json;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::problem;
use crate::transaction::Transaction;

use crate::transaction::json_value;
#[wasm_bindgen]
impl Transaction {
    /// Sets one attribute on a top-level block.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the value is not valid JSON.
    #[wasm_bindgen(js_name = setNodeAttr)]
    pub fn set_node_attr(&mut self, at: u32, name: &str, value: &str) -> Result<(), JsValue> {
        let value = json_value(value)?;
        self.push(json!({"op": "set_block_attr", "at": at, "name": name, "value": value}))
    }

    /// Sets one attribute on an inline atom.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the value is not valid JSON.
    #[wasm_bindgen(js_name = setInlineAttr)]
    pub fn set_inline_attr(
        &mut self,
        from: u32,
        to: u32,
        name: &str,
        value: &str,
    ) -> Result<(), JsValue> {
        let value = json_value(value)?;
        self.push(
            json!({"op": "set_inline_attr", "from": from, "to": to, "name": name, "value": value}),
        )
    }

    /// Sets one attribute on runs that already carry a mark.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the value is not valid JSON.
    #[allow(
        clippy::too_many_arguments,
        reason = "one marker, one attribute, and two bounds mirror the step"
    )]
    #[wasm_bindgen(js_name = setMarkAttr)]
    pub fn set_mark_attr(
        &mut self,
        from: u32,
        to: u32,
        mark: &str,
        name: &str,
        value: &str,
    ) -> Result<(), JsValue> {
        let value = json_value(value)?;
        self.push(json!({
            "op": "set_mark_attr",
            "from": from,
            "to": to,
            "mark": mark,
            "name": name,
            "value": value,
        }))
    }

    /// Sets one document-root attribute.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the value is not valid JSON.
    #[wasm_bindgen(js_name = setDocAttr)]
    pub fn set_doc_attr(&mut self, name: &str, value: &str) -> Result<(), JsValue> {
        let value = json_value(value)?;
        self.push(json!({"op": "set_doc_attr", "name": name, "value": value}))
    }

    /// Sets or clears the stable identifier of a block.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the transaction was already consumed.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "wasm-bindgen only converts owned optional strings"
    )]
    #[wasm_bindgen(js_name = setNodeId)]
    pub fn set_node_id(&mut self, at: u32, id: Option<String>) -> Result<(), JsValue> {
        self.push(json!({"op": "set_node_id", "at": at, "id": id}))
    }

    /// Sets the selection without creating history.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the selection is not valid JSON.
    #[wasm_bindgen(js_name = setSelection)]
    pub fn set_selection(&mut self, selection: &str) -> Result<(), JsValue> {
        let selection = json_value(selection)?;
        self.push(json!({"op": "set_selection", "selection": selection}))
    }

    /// Sets the origin used by the history group.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the origin name is unknown.
    #[wasm_bindgen(js_name = setOrigin)]
    pub fn set_origin(&mut self, origin: &str) -> Result<(), JsValue> {
        match origin {
            "user" | "paste" | "remote" | "api" | "import" => {
                self.origin = Some(String::from(origin));
                Ok(())
            }
            _ => Err(crate::problem_js(&problem::unknown_origin(origin))),
        }
    }

    /// Sets the history group identifier.
    #[wasm_bindgen(js_name = setGroup)]
    pub fn set_group(&mut self, group: u32) {
        self.group = Some(group);
    }

    /// Sets whether the transaction enters history.
    #[allow(
        clippy::fn_params_excessive_bools,
        reason = "the boundary mirrors the JavaScript boolean"
    )]
    #[wasm_bindgen(js_name = setAddToHistory)]
    pub fn set_add_to_history(&mut self, add: bool) {
        self.add_to_history = Some(add);
    }
}
