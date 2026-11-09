//! Selection-oriented commands exposed to JavaScript.

use rich_editor_auy::error::StepError;
use rich_editor_auy::pos::Pos;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::attr;
use crate::convert::applied_json;
use crate::names::{json_value, mark_from_json, node_attrs};
use crate::problem;

use super::{Editor, problem_js};

#[wasm_bindgen]
impl Editor {
    /// Inserts text at the selection, replacing it first.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the selection is missing or the insertion is invalid.
    #[wasm_bindgen(js_name = insertText)]
    pub fn insert_text(&mut self, text: &str) -> Result<String, JsValue> {
        let info = self
            .inner
            .insert_text(text)
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Deletes one scalar before the caret or the whole selection.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the deletion is invalid.
    #[wasm_bindgen(js_name = deleteBackward)]
    pub fn delete_backward(&mut self) -> Result<String, JsValue> {
        let info = self
            .inner
            .delete_backward()
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Deletes one scalar after the caret or the whole selection.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the deletion is invalid.
    #[wasm_bindgen(js_name = deleteForward)]
    pub fn delete_forward(&mut self) -> Result<String, JsValue> {
        let info = self
            .inner
            .delete_forward()
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Deletes the selected range.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the selection is empty or the deletion is invalid.
    #[wasm_bindgen(js_name = deleteSelection)]
    pub fn delete_selection(&mut self) -> Result<String, JsValue> {
        let info = self
            .inner
            .delete_selection()
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Splits the block containing the selection head.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the head is not inside a text block.
    #[wasm_bindgen(js_name = splitBlock)]
    pub fn split_block(&mut self) -> Result<String, JsValue> {
        let info = self
            .inner
            .split_block()
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Inserts a hard break at the selection.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the insertion is invalid.
    #[wasm_bindgen(js_name = insertHardBreak)]
    pub fn insert_hard_break(&mut self) -> Result<String, JsValue> {
        let info = self
            .inner
            .insert_hard_break()
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Changes the type of the block containing the selection head.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the node or an attribute is unknown.
    #[wasm_bindgen(js_name = setBlockType)]
    pub fn set_block_type(&mut self, kind: &str, attrs: Option<String>) -> Result<String, JsValue> {
        let node = self
            .schema
            .node_kind(kind)
            .ok_or_else(|| problem_js(&problem::token_problem(kind, "node")))?;
        let decoded = node_attrs(&self.schema, node, attrs)?;
        let info = self
            .inner
            .set_block_type(node, decoded)
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Toggles a mark on the selection or in the stored marks.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the mark or an attribute is unknown.
    #[wasm_bindgen(js_name = toggleMark)]
    pub fn toggle_mark(&mut self, name: &str, attrs: Option<String>) -> Result<String, JsValue> {
        let mark = mark_from_json(&self.schema, name, attrs)?;
        let info = self
            .inner
            .toggle_mark(mark)
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Sets one attribute on the block containing the selection head.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the attribute is unknown or the value is invalid.
    #[wasm_bindgen(js_name = setNodeAttr)]
    pub fn set_node_attr(&mut self, name: &str, value: &str) -> Result<String, JsValue> {
        let head = self
            .inner
            .selection()
            .head()
            .ok_or_else(|| problem_js(&problem::missing_selection()))?;
        let kind = attr::block_kind_at(self.inner.doc(), head).ok_or_else(|| {
            problem_js(&problem::from_step_error(&StepError::OutOfBounds {
                position: head.0,
                size: self.inner.doc().size().0,
            }))
        })?;
        let spec = self
            .schema
            .node_attr(kind, name)
            .ok_or_else(|| problem_js(&problem::attr_problem(name, "node")))?;
        let decoded = attr::value_from_json(&spec.kind, &json_value(value)?, name)
            .map_err(|problem| problem_js(&problem))?;
        let mut transaction = self.inner.transaction();
        transaction
            .set_node_attr(head, spec.id, decoded)
            .map_err(|error| problem_js(&problem::from_step_error(&error)))?;
        let info = self
            .inner
            .apply(transaction)
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Sets one attribute on the inline atom covered by the selection.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the selection is not one atom.
    #[wasm_bindgen(js_name = setInlineAttr)]
    pub fn set_inline_attr(&mut self, name: &str, value: &str) -> Result<String, JsValue> {
        let range = self
            .inner
            .selection()
            .range()
            .ok_or_else(|| problem_js(&problem::missing_selection()))?;
        let kind = attr::atom_kind_at(self.inner.doc(), range.start, range.end)
            .ok_or_else(|| problem_js(&problem::missing_selection()))?;
        let spec = self
            .schema
            .node_attr(kind, name)
            .ok_or_else(|| problem_js(&problem::attr_problem(name, "inline")))?;
        let decoded = attr::value_from_json(&spec.kind, &json_value(value)?, name)
            .map_err(|problem| problem_js(&problem))?;
        let mut transaction = self.inner.transaction();
        transaction
            .set_inline_attr(range, spec.id, decoded)
            .map_err(|error| problem_js(&problem::from_step_error(&error)))?;
        let info = self
            .inner
            .apply(transaction)
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Sets one document-root attribute.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the attribute is unknown or the value is invalid.
    #[wasm_bindgen(js_name = setDocAttr)]
    pub fn set_doc_attr(&mut self, name: &str, value: &str) -> Result<String, JsValue> {
        let spec = self
            .schema
            .doc()
            .attrs
            .iter()
            .find(|spec| &*spec.name == name)
            .ok_or_else(|| problem_js(&problem::attr_problem(name, "doc")))?;
        let decoded = attr::value_from_json(&spec.kind, &json_value(value)?, name)
            .map_err(|problem| problem_js(&problem))?;
        let mut transaction = self.inner.transaction();
        transaction
            .set_doc_attr(spec.id, decoded)
            .map_err(|error| problem_js(&problem::from_step_error(&error)))?;
        let info = self
            .inner
            .apply(transaction)
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Sets or clears the stable identifier of the block at a position.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the position is outside the document.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "wasm-bindgen only converts owned optional strings"
    )]
    #[wasm_bindgen(js_name = setNodeId)]
    pub fn set_node_id(&mut self, at: u32, id: Option<String>) -> Result<String, JsValue> {
        let mut transaction = self.inner.transaction();
        transaction
            .set_node_id(Pos(at), id.as_deref())
            .map_err(|error| problem_js(&problem::from_step_error(&error)))?;
        let info = self
            .inner
            .apply(transaction)
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(applied_json(&info))
    }

    /// Returns whether undo is available.
    #[wasm_bindgen(js_name = canUndo)]
    pub fn can_undo(&self) -> bool {
        self.inner.history_info().can_undo
    }

    /// Returns whether redo is available.
    #[wasm_bindgen(js_name = canRedo)]
    pub fn can_redo(&self) -> bool {
        self.inner.history_info().can_redo
    }
}
