//! Projection and clipboard methods kept separate to respect the file-length gate.

use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::state::CompositionEvent;
use rich_editor_auy::wire::{ParseOptions, from_slice_json, view_json};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::convert;

use super::{Editor, problem, problem_js};

#[wasm_bindgen]
impl Editor {
    /// Copies a range and returns the slice JSON.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the range is invalid or cannot be copied.
    #[wasm_bindgen(js_name = copyRange)]
    pub fn copy_range(&self, from: u32, to: u32) -> Result<String, JsValue> {
        let slice = self
            .inner
            .slice(Range::new(Pos(from), Pos(to)))
            .map_err(|error| problem_js(&problem::from_step_error(&error)))?;
        Ok(slice.to_json_string(&self.schema))
    }

    /// Inserts a slice at a position and returns the applied state.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the position or the slice is invalid.
    #[wasm_bindgen(js_name = pasteSlice)]
    pub fn paste_slice(&mut self, at: u32, slice: &str) -> Result<String, JsValue> {
        let slice = from_slice_json(slice, &self.schema, &ParseOptions::default())
            .map_err(|error| problem_js(&problem::from_parse_error(&error)))?;
        let info = self
            .inner
            .insert_slice(Pos(at), &slice)
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(convert::applied_json(&info))
    }

    /// Replaces a range with a slice and returns the applied state.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the range or the slice is invalid.
    #[wasm_bindgen(js_name = replaceSlice)]
    pub fn replace_slice(&mut self, from: u32, to: u32, slice: &str) -> Result<String, JsValue> {
        let slice = from_slice_json(slice, &self.schema, &ParseOptions::default())
            .map_err(|error| problem_js(&problem::from_parse_error(&error)))?;
        let info = self
            .inner
            .replace_slice(Range::new(Pos(from), Pos(to)), &slice)
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(convert::applied_json(&info))
    }

    /// Deletes a range with the block semantics of a clipboard cut.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the range is invalid.
    #[wasm_bindgen(js_name = deleteRange)]
    pub fn delete_range(&mut self, from: u32, to: u32) -> Result<String, JsValue> {
        let info = self
            .inner
            .delete_range(Range::new(Pos(from), Pos(to)))
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(convert::applied_json(&info))
    }

    /// Copies a range and deletes it, returning `{"slice":..., "state":...}`.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the range is invalid or cannot be copied.
    #[wasm_bindgen(js_name = cutRange)]
    pub fn cut_range(&mut self, from: u32, to: u32) -> Result<String, JsValue> {
        let slice = self.copy_range(from, to)?;
        let state = self.delete_range(from, to)?;
        Ok(format!("{{\"slice\":{slice},\"state\":{state}}}"))
    }
}

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

#[wasm_bindgen]
impl Editor {
    /// Starts an IME composition over a range.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the range is invalid or a composition is open.
    #[wasm_bindgen(js_name = beginComposition)]
    pub fn begin_composition(&mut self, from: u32, to: u32) -> Result<String, JsValue> {
        self.composition(CompositionEvent::Start {
            at: Range::new(Pos(from), Pos(to)),
        })
    }

    /// Updates the composed text.
    ///
    /// # Errors
    ///
    /// Returns a problem object when no composition is open or the text is too large.
    #[wasm_bindgen(js_name = updateComposition)]
    pub fn update_composition(&mut self, text: &str) -> Result<String, JsValue> {
        self.composition(CompositionEvent::Update { text: text.into() })
    }

    /// Commits the composition as one undo group.
    ///
    /// # Errors
    ///
    /// Returns a problem object when no composition is open.
    #[wasm_bindgen(js_name = commitComposition)]
    pub fn commit_composition(&mut self) -> Result<String, JsValue> {
        self.composition(CompositionEvent::Commit)
    }

    /// Cancels the composition and restores the original text.
    ///
    /// # Errors
    ///
    /// Returns a problem object when no composition is open.
    #[wasm_bindgen(js_name = cancelComposition)]
    pub fn cancel_composition(&mut self) -> Result<String, JsValue> {
        self.composition(CompositionEvent::Cancel)
    }

    /// Returns whether an IME composition is open.
    #[wasm_bindgen(js_name = isComposing)]
    pub fn is_composing(&self) -> bool {
        self.inner.is_composing()
    }

    /// Applies one composition event and returns the applied state.
    fn composition(&mut self, event: CompositionEvent) -> Result<String, JsValue> {
        let info = self
            .inner
            .apply_composition(event)
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(convert::applied_json(&info))
    }
}
