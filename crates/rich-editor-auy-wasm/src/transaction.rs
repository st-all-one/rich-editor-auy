//! Composable editor transaction exposed to JavaScript.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "the wasm-bindgen class shim expands to Rc and panic!; the core path stays panic-free"
)]
//!
//! A transaction collects batch operations against a document version. The editor consumes it
//! only when every operation succeeds and the version still matches; reuse is rejected.

use serde_json::{Value, json};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::problem;

/// Ordered batch operations with a base version and history metadata.
#[wasm_bindgen]
pub struct Transaction {
    pub(crate) ops: Vec<Value>,
    pub(crate) base: u32,
    pub(crate) consumed: bool,
    pub(crate) origin: Option<String>,
    pub(crate) group: Option<u32>,
    pub(crate) add_to_history: Option<bool>,
}

impl Transaction {
    /// Creates an empty transaction against a document version.
    pub(crate) fn new(base: u32) -> Self {
        Self {
            ops: Vec::new(),
            base,
            consumed: false,
            origin: None,
            group: None,
            add_to_history: None,
        }
    }

    /// Appends one operation, rejecting a consumed transaction.
    pub(crate) fn push(&mut self, op: Value) -> Result<(), JsValue> {
        if self.consumed {
            return Err(crate::problem_js(&problem::consumed_transaction()));
        }
        self.ops.push(op);
        Ok(())
    }

    /// Builds the batch payload sent to the editor.
    pub(crate) fn payload(&self) -> String {
        let mut payload = json!({ "ops": Value::Array(self.ops.clone()) });
        let Some(object) = payload.as_object_mut() else {
            return String::from("{\"ops\":[]}");
        };
        if let Some(origin) = &self.origin {
            object.insert(String::from("origin"), Value::String(origin.clone()));
        }
        if let Some(group) = self.group {
            object.insert(String::from("group"), json!(group));
        }
        if let Some(add) = self.add_to_history {
            object.insert(String::from("addToHistory"), json!(add));
        }
        payload.to_string()
    }
}

#[wasm_bindgen]
impl Transaction {
    /// Returns the document version the transaction was built against.
    #[wasm_bindgen(js_name = baseVersion)]
    pub fn base_version(&self) -> u32 {
        self.base
    }

    /// Returns whether the editor already consumed the transaction.
    #[wasm_bindgen(js_name = consumed)]
    pub fn consumed(&self) -> bool {
        self.consumed
    }

    /// Inserts plain text at a position.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the transaction was already consumed.
    #[wasm_bindgen(js_name = insertText)]
    pub fn insert_text(&mut self, at: u32, text: &str) -> Result<(), JsValue> {
        self.push(json!({"op": "insert_text", "at": at, "text": text}))
    }

    /// Deletes a byte range.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the transaction was already consumed.
    #[wasm_bindgen(js_name = delete)]
    pub fn delete(&mut self, from: u32, to: u32) -> Result<(), JsValue> {
        self.push(json!({"op": "delete_range", "from": from, "to": to}))
    }

    /// Replaces a range with a slice.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the slice is not valid JSON.
    #[wasm_bindgen(js_name = replace)]
    pub fn replace(&mut self, from: u32, to: u32, slice: &str) -> Result<(), JsValue> {
        let slice = json_value(slice)?;
        self.push(json!({"op": "replace_slice", "from": from, "to": to, "slice": slice}))
    }

    /// Splits the block containing a position.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the transaction was already consumed.
    #[wasm_bindgen(js_name = splitBlock)]
    pub fn split_block(&mut self, at: u32) -> Result<(), JsValue> {
        self.push(json!({"op": "split_block", "at": at}))
    }

    /// Joins the block containing a position with its previous sibling.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the transaction was already consumed.
    #[wasm_bindgen(js_name = joinBackward)]
    pub fn join_backward(&mut self, at: u32) -> Result<(), JsValue> {
        self.push(json!({"op": "join_backward", "at": at}))
    }

    /// Changes the type of a top-level block.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the attributes are not valid JSON.
    #[wasm_bindgen(js_name = setBlockType)]
    pub fn set_block_type(
        &mut self,
        at: u32,
        kind: &str,
        attrs: Option<String>,
    ) -> Result<(), JsValue> {
        let attrs = attrs_value(attrs)?;
        self.push(json!({"op": "set_block_type", "at": at, "kind": kind, "attrs": attrs}))
    }

    /// Adds a mark to a range.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the transaction was already consumed.
    #[wasm_bindgen(js_name = addMark)]
    pub fn add_mark(&mut self, from: u32, to: u32, mark: &str) -> Result<(), JsValue> {
        self.push(
            json!({"op": "toggle_mark", "from": from, "to": to, "mark": mark, "active": true}),
        )
    }

    /// Removes a mark from a range.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the transaction was already consumed.
    #[wasm_bindgen(js_name = removeMark)]
    pub fn remove_mark(&mut self, from: u32, to: u32, mark: &str) -> Result<(), JsValue> {
        self.push(
            json!({"op": "toggle_mark", "from": from, "to": to, "mark": mark, "active": false}),
        )
    }
}

/// Parses optional attribute JSON.
pub(crate) fn attrs_value(attrs: Option<String>) -> Result<Value, JsValue> {
    match attrs {
        Some(attrs) => json_value(&attrs),
        None => Ok(Value::Null),
    }
}

/// Parses one JSON value.
pub(crate) fn json_value(value: &str) -> Result<Value, JsValue> {
    serde_json::from_str(value).map_err(|error| {
        crate::problem_js(&problem::malformed_problem(format!("{error}"), "value"))
    })
}
