/* @ts-self-types="./rich_editor_auy_wasm.d.ts" */

/**
 * Stateful editor exposed to JavaScript.
 *
 * Every mutation is atomic: the batch is applied to a clone and committed only when every
 * operation succeeds, so a rejected batch leaves the editor byte-for-byte unchanged.
 */
export class Editor {
    static __wrap(ptr) {
        const obj = Object.create(Editor.prototype);
        obj.__wbg_ptr = ptr;
        EditorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        EditorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_editor_free(ptr, 0);
    }
    /**
     * Applies a transaction built with [`Editor::transaction`].
     *
     * # Errors
     *
     * Returns a problem object when the transaction was consumed, was built against another
     * version, or contains an invalid operation.
     * @param {Transaction} transaction
     * @returns {string}
     */
    applyTransaction(transaction) {
        let deferred2_0;
        let deferred2_1;
        try {
            _assertClass(transaction, Transaction);
            const ret = wasm.editor_applyTransaction(this.__wbg_ptr, transaction.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Applies `{"ops": [...]}` atomically.
     *
     * # Errors
     *
     * Returns a problem object when any operation is rejected.
     * @param {string} payload
     * @returns {string}
     */
    apply(payload) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(payload, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.editor_apply(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Starts an IME composition over a range.
     *
     * # Errors
     *
     * Returns a problem object when the range is invalid or a composition is open.
     * @param {number} from
     * @param {number} to
     * @returns {string}
     */
    beginComposition(from, to) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_beginComposition(this.__wbg_ptr, from, to);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Returns `{index, from, to}` for the block containing a position.
     * @param {number} pos
     * @returns {string | undefined}
     */
    blockAt(pos) {
        const ret = wasm.editor_blockAt(this.__wbg_ptr, pos);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]);
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Returns `{from, to}` for one top-level block.
     * @param {number} index
     * @returns {string | undefined}
     */
    blockRange(index) {
        const ret = wasm.editor_blockRange(this.__wbg_ptr, index);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]);
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Returns the number of top-level blocks.
     * @returns {number}
     */
    blocks() {
        const ret = wasm.editor_blocks(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns whether redo is available.
     * @returns {boolean}
     */
    canRedo() {
        const ret = wasm.editor_canRedo(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Returns whether undo is available.
     * @returns {boolean}
     */
    canUndo() {
        const ret = wasm.editor_canUndo(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Cancels the composition and restores the original text.
     *
     * # Errors
     *
     * Returns a problem object when no composition is open.
     * @returns {string}
     */
    cancelComposition() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_cancelComposition(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Discards every undo and redo group.
     */
    clearHistory() {
        wasm.editor_clearHistory(this.__wbg_ptr);
    }
    /**
     * Commits the composition as one undo group.
     *
     * # Errors
     *
     * Returns a problem object when no composition is open.
     * @returns {string}
     */
    commitComposition() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_commitComposition(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Copies a range and returns the slice JSON.
     *
     * # Errors
     *
     * Returns a problem object when the range is invalid or cannot be copied.
     * @param {number} from
     * @param {number} to
     * @returns {string}
     */
    copyRange(from, to) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_copyRange(this.__wbg_ptr, from, to);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Copies a range and deletes it, returning `{"slice":..., "state":...}`.
     *
     * # Errors
     *
     * Returns a problem object when the range is invalid or cannot be copied.
     * @param {number} from
     * @param {number} to
     * @returns {string}
     */
    cutRange(from, to) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_cutRange(this.__wbg_ptr, from, to);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Deletes one scalar before the caret or the whole selection.
     *
     * # Errors
     *
     * Returns a problem object when the deletion is invalid.
     * @returns {string}
     */
    deleteBackward() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_deleteBackward(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Deletes one scalar after the caret or the whole selection.
     *
     * # Errors
     *
     * Returns a problem object when the deletion is invalid.
     * @returns {string}
     */
    deleteForward() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_deleteForward(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Deletes a range with the block semantics of a clipboard cut.
     *
     * # Errors
     *
     * Returns a problem object when the range is invalid.
     * @param {number} from
     * @param {number} to
     * @returns {string}
     */
    deleteRange(from, to) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_deleteRange(this.__wbg_ptr, from, to);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Deletes the selected range.
     *
     * # Errors
     *
     * Returns a problem object when the selection is empty or the deletion is invalid.
     * @returns {string}
     */
    deleteSelection() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_deleteSelection(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Returns the canonical document JSON.
     * @returns {string}
     */
    docJSON() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.editor_docJSON(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Validates an import payload without mutating the editor.
     *
     * Accepts the same envelope as `import`, including a bare `{"ops": [...]}` batch. A
     * malformed envelope or a payload-level rejection (schema mismatch, stale version, byte
     * limit) throws; content problems come back in the report with `accepted: false`.
     *
     * # Errors
     *
     * Returns a problem object when the envelope is not valid JSON or fails a precheck.
     * @param {string} payload
     * @returns {string}
     */
    dryRun(payload) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(payload, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.editor_dryRun(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Creates an editor from a durable envelope.
     *
     * # Errors
     *
     * Returns a problem object when the envelope format, fingerprint, or document is invalid.
     * @param {string} input
     * @param {string | null} [options]
     * @returns {Editor}
     */
    static fromEnvelope(input, options) {
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = isLikeNone(options) ? 0 : passStringToWasm0(options, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.editor_fromEnvelope(ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Editor.__wrap(ret[0]);
    }
    /**
     * Creates an editor from canonical document JSON.
     *
     * `options` is JSON with optional `historyLimits: [maxGroups, maxBytes]`.
     *
     * # Errors
     * Returns a problem object when the document or the options are rejected.
     * @param {string} input
     * @param {string | null} [options]
     * @returns {Editor}
     */
    static fromJSON(input, options) {
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = isLikeNone(options) ? 0 : passStringToWasm0(options, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.editor_fromJSON(ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Editor.__wrap(ret[0]);
    }
    /**
     * Returns the observable history state.
     * @returns {string}
     */
    historyJSON() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.editor_historyJSON(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Imports a document, fragment, or operation batch atomically.
     *
     * The returned report carries `accepted`, the resulting block count, the editor version,
     * every `ProblemDetails`, and the current state.
     * @param {string} payload
     * @returns {string}
     */
    import(payload) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ptr0 = passStringToWasm0(payload, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.editor_import(this.__wbg_ptr, ptr0, len0);
            deferred2_0 = ret[0];
            deferred2_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Inserts a hard break at the selection.
     *
     * # Errors
     *
     * Returns a problem object when the insertion is invalid.
     * @returns {string}
     */
    insertHardBreak() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_insertHardBreak(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Inserts text at the selection, replacing it first.
     *
     * # Errors
     *
     * Returns a problem object when the selection is missing or the insertion is invalid.
     * @param {string} text
     * @returns {string}
     */
    insertText(text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.editor_insertText(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Returns whether an IME composition is open.
     * @returns {boolean}
     */
    isComposing() {
        const ret = wasm.editor_isComposing(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Creates an empty editor using the complete schema.
     *
     * # Errors
     * Returns a problem object only if the full preset is inconsistent.
     */
    constructor() {
        const ret = wasm.editor_new();
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        EditorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Inserts a slice at a position and returns the applied state.
     *
     * # Errors
     *
     * Returns a problem object when the position or the slice is invalid.
     * @param {number} at
     * @param {string} slice
     * @returns {string}
     */
    pasteSlice(at, slice) {
        let deferred4_0;
        let deferred4_1;
        try {
            const val2 = +at;
            const ptr0 = passStringToWasm0(slice, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.editor_pasteSlice(this.__wbg_ptr, val2, ptr0, len0);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Converts a document position to UTF-16 code units inside its text block.
     * @param {number} pos
     * @returns {number | undefined}
     */
    posToUtf16(pos) {
        const ret = wasm.editor_posToUtf16(this.__wbg_ptr, pos);
        return ret === Number.MAX_SAFE_INTEGER ? undefined : ret;
    }
    /**
     * Redoes the latest undone group.
     *
     * # Errors
     *
     * Returns a problem object when the step no longer applies.
     * @returns {string}
     */
    redo() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_redo(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Replaces a range with a slice and returns the applied state.
     *
     * # Errors
     *
     * Returns a problem object when the range or the slice is invalid.
     * @param {number} from
     * @param {number} to
     * @param {string} slice
     * @returns {string}
     */
    replaceSlice(from, to, slice) {
        let deferred5_0;
        let deferred5_1;
        try {
            const val2 = +from;
            const val3 = +to;
            const ptr0 = passStringToWasm0(slice, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.editor_replaceSlice(this.__wbg_ptr, val2, val3, ptr0, len0);
            var ptr4 = ret[0];
            var len4 = ret[1];
            if (ret[3]) {
                ptr4 = 0; len4 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred5_0 = ptr4;
            deferred5_1 = len4;
            return getStringFromWasm0(ptr4, len4);
        } finally {
            wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
        }
    }
    /**
     * Returns the active schema fingerprint.
     * @returns {string}
     */
    schemaFingerprint() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.editor_schemaFingerprint(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Changes the type of the block containing the selection head.
     *
     * # Errors
     *
     * Returns a problem object when the node or an attribute is unknown.
     * @param {string} kind
     * @param {string | null} [attrs]
     * @returns {string}
     */
    setBlockType(kind, attrs) {
        let deferred4_0;
        let deferred4_1;
        try {
            let ptr0, len0, ptr1, len1;
            try {
                ptr0 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                len0 = WASM_VECTOR_LEN;
                ptr1 = isLikeNone(attrs) ? 0 : passStringToWasm0(attrs, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                len1 = WASM_VECTOR_LEN;
            } catch (e) {
                if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
                if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
                throw e;
            }
            const ret = wasm.editor_setBlockType(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Sets one document-root attribute.
     *
     * # Errors
     *
     * Returns a problem object when the attribute is unknown or the value is invalid.
     * @param {string} name
     * @param {string} value
     * @returns {string}
     */
    setDocAttr(name, value) {
        let deferred4_0;
        let deferred4_1;
        try {
            let ptr0, len0, ptr1, len1;
            try {
                ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                len0 = WASM_VECTOR_LEN;
                ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                len1 = WASM_VECTOR_LEN;
            } catch (e) {
                if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
                if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
                throw e;
            }
            const ret = wasm.editor_setDocAttr(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Sets one attribute on the inline atom covered by the selection.
     *
     * # Errors
     *
     * Returns a problem object when the selection is not one atom.
     * @param {string} name
     * @param {string} value
     * @returns {string}
     */
    setInlineAttr(name, value) {
        let deferred4_0;
        let deferred4_1;
        try {
            let ptr0, len0, ptr1, len1;
            try {
                ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                len0 = WASM_VECTOR_LEN;
                ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                len1 = WASM_VECTOR_LEN;
            } catch (e) {
                if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
                if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
                throw e;
            }
            const ret = wasm.editor_setInlineAttr(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Sets one attribute on the block containing the selection head.
     *
     * # Errors
     *
     * Returns a problem object when the attribute is unknown or the value is invalid.
     * @param {string} name
     * @param {string} value
     * @returns {string}
     */
    setNodeAttr(name, value) {
        let deferred4_0;
        let deferred4_1;
        try {
            let ptr0, len0, ptr1, len1;
            try {
                ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                len0 = WASM_VECTOR_LEN;
                ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                len1 = WASM_VECTOR_LEN;
            } catch (e) {
                if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
                if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
                throw e;
            }
            const ret = wasm.editor_setNodeAttr(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Sets or clears the stable identifier of the block at a position.
     *
     * # Errors
     *
     * Returns a problem object when the position is outside the document.
     * @param {number} at
     * @param {string | null} [id]
     * @returns {string}
     */
    setNodeId(at, id) {
        let deferred4_0;
        let deferred4_1;
        try {
            const val2 = +at;
            var ptr0 = isLikeNone(id) ? 0 : passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            const ret = wasm.editor_setNodeId(this.__wbg_ptr, val2, ptr0, len0);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Sets the selection without creating history.
     *
     * # Errors
     *
     * Returns a problem object when a position lies beyond the document.
     * @param {number} from
     * @param {number} to
     * @returns {string}
     */
    setSelection(from, to) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_setSelection(this.__wbg_ptr, from, to);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Snaps a position to the previous scalar boundary.
     * @param {number} pos
     * @returns {number}
     */
    snapToScalar(pos) {
        const ret = wasm.editor_snapToScalar(this.__wbg_ptr, pos);
        return ret >>> 0;
    }
    /**
     * Splits the block containing the selection head.
     *
     * # Errors
     *
     * Returns a problem object when the head is not inside a text block.
     * @returns {string}
     */
    splitBlock() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_splitBlock(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Returns `{doc, selection, version}`.
     * @returns {string}
     */
    stateJSON() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.editor_stateJSON(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Toggles a mark on the selection or in the stored marks.
     *
     * # Errors
     *
     * Returns a problem object when the mark or an attribute is unknown.
     * @param {string} name
     * @param {string | null} [attrs]
     * @returns {string}
     */
    toggleMark(name, attrs) {
        let deferred4_0;
        let deferred4_1;
        try {
            let ptr0, len0, ptr1, len1;
            try {
                ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                len0 = WASM_VECTOR_LEN;
                ptr1 = isLikeNone(attrs) ? 0 : passStringToWasm0(attrs, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                len1 = WASM_VECTOR_LEN;
            } catch (e) {
                if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
                if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
                throw e;
            }
            const ret = wasm.editor_toggleMark(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Starts a transaction against the current document version.
     * @returns {Transaction}
     */
    transaction() {
        const ret = wasm.editor_transaction(this.__wbg_ptr);
        return Transaction.__wrap(ret);
    }
    /**
     * Undoes the latest group.
     *
     * # Errors
     *
     * Returns a problem object when the inverse step no longer applies.
     * @returns {string}
     */
    undo() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_undo(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Updates the composed text.
     *
     * # Errors
     *
     * Returns a problem object when no composition is open or the text is too large.
     * @param {string} text
     * @returns {string}
     */
    updateComposition(text) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.editor_updateComposition(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Converts a UTF-16 offset inside a block to a document position.
     * @param {number} block
     * @param {number} utf16
     * @returns {number | undefined}
     */
    utf16ToPos(block, utf16) {
        const ret = wasm.editor_utf16ToPos(this.__wbg_ptr, block, utf16);
        return ret === Number.MAX_SAFE_INTEGER ? undefined : ret;
    }
    /**
     * Returns the accessibility report.
     * @returns {string}
     */
    validateA11y() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.editor_validateA11y(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Returns the structural validation report.
     * @returns {string}
     */
    validate() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.editor_validate(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Returns the current document version.
     * @returns {number}
     */
    version() {
        const ret = wasm.editor_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the render-oriented `DocView` projection.
     *
     * # Errors
     *
     * Returns a problem object only if the projection cannot be encoded.
     * @returns {string}
     */
    viewJSON() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.editor_viewJSON(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
}
if (Symbol.dispose) Editor.prototype[Symbol.dispose] = Editor.prototype.free;

/**
 * Ordered batch operations with a base version and history metadata.
 */
export class Transaction {
    static __wrap(ptr) {
        const obj = Object.create(Transaction.prototype);
        obj.__wbg_ptr = ptr;
        TransactionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TransactionFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_transaction_free(ptr, 0);
    }
    /**
     * Adds a mark to a range.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     * @param {number} from
     * @param {number} to
     * @param {string} mark
     */
    addMark(from, to, mark) {
        const val2 = +from;
        const val3 = +to;
        const ptr0 = passStringToWasm0(mark, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.transaction_addMark(this.__wbg_ptr, val2, val3, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Returns the document version the transaction was built against.
     * @returns {number}
     */
    baseVersion() {
        const ret = wasm.transaction_baseVersion(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns whether the editor already consumed the transaction.
     * @returns {boolean}
     */
    consumed() {
        const ret = wasm.transaction_consumed(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Deletes a byte range.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     * @param {number} from
     * @param {number} to
     */
    delete(from, to) {
        const ret = wasm.transaction_delete(this.__wbg_ptr, from, to);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Inserts plain text at a position.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     * @param {number} at
     * @param {string} text
     */
    insertText(at, text) {
        const val2 = +at;
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.transaction_insertText(this.__wbg_ptr, val2, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Joins the block containing a position with its previous sibling.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     * @param {number} at
     */
    joinBackward(at) {
        const ret = wasm.transaction_joinBackward(this.__wbg_ptr, at);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Removes a mark from a range.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     * @param {number} from
     * @param {number} to
     * @param {string} mark
     */
    removeMark(from, to, mark) {
        const val2 = +from;
        const val3 = +to;
        const ptr0 = passStringToWasm0(mark, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.transaction_removeMark(this.__wbg_ptr, val2, val3, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Replaces a range with a slice.
     *
     * # Errors
     *
     * Returns a problem object when the slice is not valid JSON.
     * @param {number} from
     * @param {number} to
     * @param {string} slice
     */
    replace(from, to, slice) {
        const val2 = +from;
        const val3 = +to;
        const ptr0 = passStringToWasm0(slice, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.transaction_replace(this.__wbg_ptr, val2, val3, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Sets whether the transaction enters history.
     * @param {boolean} add
     */
    setAddToHistory(add) {
        wasm.transaction_setAddToHistory(this.__wbg_ptr, add);
    }
    /**
     * Changes the type of a top-level block.
     *
     * # Errors
     *
     * Returns a problem object when the attributes are not valid JSON.
     * @param {number} at
     * @param {string} kind
     * @param {string | null} [attrs]
     */
    setBlockType(at, kind, attrs) {
        const val3 = +at;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = isLikeNone(attrs) ? 0 : passStringToWasm0(attrs, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.transaction_setBlockType(this.__wbg_ptr, val3, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Sets one document-root attribute.
     *
     * # Errors
     *
     * Returns a problem object when the value is not valid JSON.
     * @param {string} name
     * @param {string} value
     */
    setDocAttr(name, value) {
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.transaction_setDocAttr(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Sets the history group identifier.
     * @param {number} group
     */
    setGroup(group) {
        wasm.transaction_setGroup(this.__wbg_ptr, group);
    }
    /**
     * Sets one attribute on an inline atom.
     *
     * # Errors
     *
     * Returns a problem object when the value is not valid JSON.
     * @param {number} from
     * @param {number} to
     * @param {string} name
     * @param {string} value
     */
    setInlineAttr(from, to, name, value) {
        const val3 = +from;
        const val4 = +to;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.transaction_setInlineAttr(this.__wbg_ptr, val3, val4, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Sets one attribute on runs that already carry a mark.
     *
     * # Errors
     *
     * Returns a problem object when the value is not valid JSON.
     * @param {number} from
     * @param {number} to
     * @param {string} mark
     * @param {string} name
     * @param {string} value
     */
    setMarkAttr(from, to, mark, name, value) {
        const val4 = +from;
        const val5 = +to;
        let ptr0, len0, ptr1, len1, ptr2, len2;
        try {
            ptr0 = passStringToWasm0(mark, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
            ptr2 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len2 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr2) { wasm.__wbindgen_free(ptr2, len2 * 1, 1); }
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.transaction_setMarkAttr(this.__wbg_ptr, val4, val5, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Sets one attribute on a top-level block.
     *
     * # Errors
     *
     * Returns a problem object when the value is not valid JSON.
     * @param {number} at
     * @param {string} name
     * @param {string} value
     */
    setNodeAttr(at, name, value) {
        const val3 = +at;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.transaction_setNodeAttr(this.__wbg_ptr, val3, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Sets or clears the stable identifier of a block.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     * @param {number} at
     * @param {string | null} [id]
     */
    setNodeId(at, id) {
        const val2 = +at;
        var ptr0 = isLikeNone(id) ? 0 : passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.transaction_setNodeId(this.__wbg_ptr, val2, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Sets the origin used by the history group.
     *
     * # Errors
     *
     * Returns a problem object when the origin name is unknown.
     * @param {string} origin
     */
    setOrigin(origin) {
        const ptr0 = passStringToWasm0(origin, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.transaction_setOrigin(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Sets the selection without creating history.
     *
     * # Errors
     *
     * Returns a problem object when the selection is not valid JSON.
     * @param {string} selection
     */
    setSelection(selection) {
        const ptr0 = passStringToWasm0(selection, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.transaction_setSelection(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Splits the block containing a position.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     * @param {number} at
     */
    splitBlock(at) {
        const ret = wasm.transaction_splitBlock(this.__wbg_ptr, at);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}
if (Symbol.dispose) Transaction.prototype[Symbol.dispose] = Transaction.prototype.free;

/**
 * Returns the semantic accessibility tree as canonical JSON.
 *
 * # Errors
 *
 * Returns a problem object when the input is not a document accepted by the full schema.
 * @param {string} input
 * @returns {string}
 */
export function a11y_tree(input) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.a11y_tree(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Canonicalizes a document JSON string.
 *
 * # Errors
 *
 * Returns a problem object when the input is not a document accepted by the full schema.
 * @param {string} input
 * @returns {string}
 */
export function canonicalize(input) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.canonicalize(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Validates and canonicalizes a durable envelope, returning canonical envelope JSON.
 *
 * # Errors
 *
 * Returns a problem object when the envelope format, fingerprint, or document is invalid.
 * @param {string} input
 * @returns {string}
 */
export function canonicalize_envelope(input) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.canonicalize_envelope(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Deletes a byte range and returns canonical JSON.
 *
 * # Errors
 *
 * Returns a problem object when the range or input is invalid.
 * @param {string} input
 * @param {number} from
 * @param {number} to
 * @returns {string}
 */
export function delete_range(input, from, to) {
    let deferred5_0;
    let deferred5_1;
    try {
        const val2 = +from;
        const val3 = +to;
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.delete_range(ptr0, len0, val2, val3);
        var ptr4 = ret[0];
        var len4 = ret[1];
        if (ret[3]) {
            ptr4 = 0; len4 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred5_0 = ptr4;
        deferred5_1 = len4;
        return getStringFromWasm0(ptr4, len4);
    } finally {
        wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
    }
}

/**
 * Validates and canonicalizes a durable envelope, returning its document JSON.
 *
 * # Errors
 *
 * Returns a problem object when the envelope format, fingerprint, or document is invalid.
 * @param {string} input
 * @returns {string}
 */
export function envelope_doc(input) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.envelope_doc(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Inserts text at a byte position and returns canonical JSON.
 *
 * # Errors
 *
 * Returns a problem object when the position or input is invalid.
 * @param {string} input
 * @param {number} position
 * @param {string} text
 * @returns {string}
 */
export function insert_text(input, position, text) {
    let deferred5_0;
    let deferred5_1;
    try {
        const val3 = +position;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.insert_text(ptr0, len0, val3, ptr1, len1);
        var ptr4 = ret[0];
        var len4 = ret[1];
        if (ret[3]) {
            ptr4 = 0; len4 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred5_0 = ptr4;
        deferred5_1 = len4;
        return getStringFromWasm0(ptr4, len4);
    } finally {
        wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
    }
}

/**
 * Joins the block at a byte boundary with its previous sibling.
 *
 * # Errors
 *
 * Returns a problem object when the position or input is invalid.
 * @param {string} input
 * @param {number} position
 * @returns {string}
 */
export function join_backward(input, position) {
    let deferred4_0;
    let deferred4_1;
    try {
        const val2 = +position;
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.join_backward(ptr0, len0, val2);
        var ptr3 = ret[0];
        var len3 = ret[1];
        if (ret[3]) {
            ptr3 = 0; len3 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred4_0 = ptr3;
        deferred4_1 = len3;
        return getStringFromWasm0(ptr3, len3);
    } finally {
        wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
    }
}

/**
 * Returns the plain text of a document.
 *
 * # Errors
 *
 * Returns a problem object when the input is not a document accepted by the full schema.
 * @param {string} input
 * @returns {string}
 */
export function plain_text(input) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.plain_text(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Returns the canonical BLAKE3 fingerprint of the full schema.
 *
 * # Errors
 *
 * Returns a problem object only if the full preset is inconsistent.
 * @returns {string}
 */
export function schema_fingerprint() {
    let deferred2_0;
    let deferred2_1;
    try {
        const ret = wasm.schema_fingerprint();
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * Splits the block at a byte position and returns canonical JSON.
 *
 * # Errors
 *
 * Returns a problem object when the position or input is invalid.
 * @param {string} input
 * @param {number} position
 * @returns {string}
 */
export function split_block(input, position) {
    let deferred4_0;
    let deferred4_1;
    try {
        const val2 = +position;
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.split_block(ptr0, len0, val2);
        var ptr3 = ret[0];
        var len3 = ret[1];
        if (ret[3]) {
            ptr3 = 0; len3 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred4_0 = ptr3;
        deferred4_1 = len3;
        return getStringFromWasm0(ptr3, len3);
    } finally {
        wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
    }
}

/**
 * Toggles a named mark over a byte range and returns canonical JSON.
 *
 * # Errors
 *
 * Returns a problem object when the mark, range, or input is invalid.
 * @param {string} input
 * @param {number} from
 * @param {number} to
 * @param {string} mark
 * @returns {string}
 */
export function toggle_mark(input, from, to, mark) {
    let deferred6_0;
    let deferred6_1;
    try {
        const val3 = +from;
        const val4 = +to;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(mark, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.toggle_mark(ptr0, len0, val3, val4, ptr1, len1);
        var ptr5 = ret[0];
        var len5 = ret[1];
        if (ret[3]) {
            ptr5 = 0; len5 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred6_0 = ptr5;
        deferred6_1 = len5;
        return getStringFromWasm0(ptr5, len5);
    } finally {
        wasm.__wbindgen_free(deferred6_0, deferred6_1, 1);
    }
}

/**
 * Returns stable accessibility error codes, or `ok`.
 *
 * # Errors
 *
 * Returns a problem object when the input is not a document accepted by the full schema.
 * @param {string} input
 * @returns {string}
 */
export function validate_a11y(input) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.validate_a11y(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Returns the crate version exposed to JavaScript consumers.
 * @returns {string}
 */
export function version() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.version();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_4c88d8bf264fd984: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            table.set(0, undefined);
            table.set(1024, undefined);
            table.set(1025, null);
            table.set(1026, true);
            table.set(1027, false);
        },
    };
    return {
        __proto__: null,
        "./rich_editor_auy_wasm_bg.js": import0,
    };
}

const EditorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_editor_free(ptr, 1));
const TransactionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_transaction_free(ptr, 1));

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(bytes) {
    numBytesDecoded += bytes.length;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
        cachedTextDecoder.decode();
        numBytesDecoded = bytes.length;
    }
    return cachedTextDecoder.decode(bytes);
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

let __wbg_init_promise;
async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;
    if (__wbg_init_promise === undefined) {
        __wbg_init_promise = __wbg_init_once(module_or_path);
        __wbg_init_promise.catch(() => { __wbg_init_promise = undefined; });
    }
    return __wbg_init_promise;
}

async function __wbg_init_once(module_or_path) {

    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('rich_editor_auy_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
