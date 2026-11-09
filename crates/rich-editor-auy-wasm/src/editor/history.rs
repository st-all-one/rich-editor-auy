//! Undo, redo, selection, and history methods.

use rich_editor_auy::pos::Pos;
use rich_editor_auy::state::Selection;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::convert::{empty_applied_json, undo_json};
use crate::problem;

use super::{Editor, problem_js};

#[wasm_bindgen]
impl Editor {
    /// Undoes the latest group.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the inverse step no longer applies.
    pub fn undo(&mut self) -> Result<String, JsValue> {
        let applied = self
            .inner
            .undo()
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(undo_json(
            applied.as_ref(),
            self.inner.selection(),
            self.version(),
        ))
    }

    /// Redoes the latest undone group.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the step no longer applies.
    pub fn redo(&mut self) -> Result<String, JsValue> {
        let applied = self
            .inner
            .redo()
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(undo_json(
            applied.as_ref(),
            self.inner.selection(),
            self.version(),
        ))
    }

    /// Sets the selection without creating history.
    ///
    /// # Errors
    ///
    /// Returns a problem object when a position lies beyond the document.
    #[wasm_bindgen(js_name = setSelection)]
    pub fn set_selection(&mut self, from: u32, to: u32) -> Result<String, JsValue> {
        let selection = if from == to {
            Selection::caret(Pos(from))
        } else {
            Selection::text(Pos(from), Pos(to))
        };
        self.inner
            .set_selection(selection)
            .map_err(|error| problem_js(&problem::from_selection_error(&error)))?;
        Ok(empty_applied_json(
            self.inner.selection(),
            self.version(),
            self.inner.history_info(),
        ))
    }

    /// Discards every undo and redo group.
    #[wasm_bindgen(js_name = clearHistory)]
    pub fn clear_history(&mut self) {
        self.inner.clear_history();
    }
}
