//! Transactions created by the stateful editor.

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::problem;
use crate::transaction::Transaction;

use super::{Editor, problem_js};

#[wasm_bindgen]
impl Editor {
    /// Starts a transaction against the current document version.
    pub fn transaction(&self) -> Transaction {
        Transaction::new(self.version())
    }

    /// Applies a transaction built with [`Editor::transaction`].
    ///
    /// # Errors
    ///
    /// Returns a problem object when the transaction was consumed, was built against another
    /// version, or contains an invalid operation.
    #[wasm_bindgen(js_name = applyTransaction)]
    pub fn apply_transaction(&mut self, transaction: &mut Transaction) -> Result<String, JsValue> {
        if transaction.consumed {
            return Err(problem_js(&problem::consumed_transaction()));
        }
        if transaction.base != self.version() {
            return Err(problem_js(&problem::stale_problem(
                transaction.base,
                self.version(),
            )));
        }
        let payload = transaction.payload();
        let result = self.apply(&payload)?;
        transaction.consumed = true;
        Ok(result)
    }
}
