/* tslint:disable */
/* eslint-disable */

/**
 * Stateful editor exposed to JavaScript.
 *
 * Every mutation is atomic: the batch is applied to a clone and committed only when every
 * operation succeeds, so a rejected batch leaves the editor byte-for-byte unchanged.
 */
export class Editor {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Applies a transaction built with [`Editor::transaction`].
     *
     * # Errors
     *
     * Returns a problem object when the transaction was consumed, was built against another
     * version, or contains an invalid operation.
     */
    applyTransaction(transaction: Transaction): string;
    /**
     * Applies `{"ops": [...]}` atomically.
     *
     * # Errors
     *
     * Returns a problem object when any operation is rejected.
     */
    apply(payload: string): string;
    /**
     * Starts an IME composition over a range.
     *
     * # Errors
     *
     * Returns a problem object when the range is invalid or a composition is open.
     */
    beginComposition(from: number, to: number): string;
    /**
     * Returns `{index, from, to}` for the block containing a position.
     */
    blockAt(pos: number): string | undefined;
    /**
     * Returns `{from, to}` for one top-level block.
     */
    blockRange(index: number): string | undefined;
    /**
     * Returns the number of top-level blocks.
     */
    blocks(): number;
    /**
     * Returns whether redo is available.
     */
    canRedo(): boolean;
    /**
     * Returns whether undo is available.
     */
    canUndo(): boolean;
    /**
     * Cancels the composition and restores the original text.
     *
     * # Errors
     *
     * Returns a problem object when no composition is open.
     */
    cancelComposition(): string;
    /**
     * Discards every undo and redo group.
     */
    clearHistory(): void;
    /**
     * Commits the composition as one undo group.
     *
     * # Errors
     *
     * Returns a problem object when no composition is open.
     */
    commitComposition(): string;
    /**
     * Copies a range and returns the slice JSON.
     *
     * # Errors
     *
     * Returns a problem object when the range is invalid or cannot be copied.
     */
    copyRange(from: number, to: number): string;
    /**
     * Copies a range and deletes it, returning `{"slice":..., "state":...}`.
     *
     * # Errors
     *
     * Returns a problem object when the range is invalid or cannot be copied.
     */
    cutRange(from: number, to: number): string;
    /**
     * Deletes one scalar before the caret or the whole selection.
     *
     * # Errors
     *
     * Returns a problem object when the deletion is invalid.
     */
    deleteBackward(): string;
    /**
     * Deletes one scalar after the caret or the whole selection.
     *
     * # Errors
     *
     * Returns a problem object when the deletion is invalid.
     */
    deleteForward(): string;
    /**
     * Deletes a range with the block semantics of a clipboard cut.
     *
     * # Errors
     *
     * Returns a problem object when the range is invalid.
     */
    deleteRange(from: number, to: number): string;
    /**
     * Deletes the selected range.
     *
     * # Errors
     *
     * Returns a problem object when the selection is empty or the deletion is invalid.
     */
    deleteSelection(): string;
    /**
     * Returns the canonical document JSON.
     */
    docJSON(): string;
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
     */
    dryRun(payload: string): string;
    /**
     * Creates an editor from a durable envelope.
     *
     * # Errors
     *
     * Returns a problem object when the envelope format, fingerprint, or document is invalid.
     */
    static fromEnvelope(input: string, options?: string | null): Editor;
    /**
     * Creates an editor from canonical document JSON.
     *
     * `options` is JSON with optional `historyLimits: [maxGroups, maxBytes]`.
     *
     * # Errors
     * Returns a problem object when the document or the options are rejected.
     */
    static fromJSON(input: string, options?: string | null): Editor;
    /**
     * Returns the observable history state.
     */
    historyJSON(): string;
    /**
     * Imports a document, fragment, or operation batch atomically.
     *
     * The returned report carries `accepted`, the resulting block count, the editor version,
     * every `ProblemDetails`, and the current state.
     */
    import(payload: string): string;
    /**
     * Inserts a hard break at the selection.
     *
     * # Errors
     *
     * Returns a problem object when the insertion is invalid.
     */
    insertHardBreak(): string;
    /**
     * Inserts text at the selection, replacing it first.
     *
     * # Errors
     *
     * Returns a problem object when the selection is missing or the insertion is invalid.
     */
    insertText(text: string): string;
    /**
     * Returns whether an IME composition is open.
     */
    isComposing(): boolean;
    /**
     * Creates an empty editor using the complete schema.
     *
     * # Errors
     * Returns a problem object only if the full preset is inconsistent.
     */
    constructor();
    /**
     * Inserts a slice at a position and returns the applied state.
     *
     * # Errors
     *
     * Returns a problem object when the position or the slice is invalid.
     */
    pasteSlice(at: number, slice: string): string;
    /**
     * Converts a document position to UTF-16 code units inside its text block.
     */
    posToUtf16(pos: number): number | undefined;
    /**
     * Redoes the latest undone group.
     *
     * # Errors
     *
     * Returns a problem object when the step no longer applies.
     */
    redo(): string;
    /**
     * Replaces a range with a slice and returns the applied state.
     *
     * # Errors
     *
     * Returns a problem object when the range or the slice is invalid.
     */
    replaceSlice(from: number, to: number, slice: string): string;
    /**
     * Returns the active schema fingerprint.
     */
    schemaFingerprint(): string;
    /**
     * Changes the type of the block containing the selection head.
     *
     * # Errors
     *
     * Returns a problem object when the node or an attribute is unknown.
     */
    setBlockType(kind: string, attrs?: string | null): string;
    /**
     * Sets one document-root attribute.
     *
     * # Errors
     *
     * Returns a problem object when the attribute is unknown or the value is invalid.
     */
    setDocAttr(name: string, value: string): string;
    /**
     * Sets one attribute on the inline atom covered by the selection.
     *
     * # Errors
     *
     * Returns a problem object when the selection is not one atom.
     */
    setInlineAttr(name: string, value: string): string;
    /**
     * Sets one attribute on the block containing the selection head.
     *
     * # Errors
     *
     * Returns a problem object when the attribute is unknown or the value is invalid.
     */
    setNodeAttr(name: string, value: string): string;
    /**
     * Sets or clears the stable identifier of the block at a position.
     *
     * # Errors
     *
     * Returns a problem object when the position is outside the document.
     */
    setNodeId(at: number, id?: string | null): string;
    /**
     * Sets the selection without creating history.
     *
     * # Errors
     *
     * Returns a problem object when a position lies beyond the document.
     */
    setSelection(from: number, to: number): string;
    /**
     * Snaps a position to the previous scalar boundary.
     */
    snapToScalar(pos: number): number;
    /**
     * Splits the block containing the selection head.
     *
     * # Errors
     *
     * Returns a problem object when the head is not inside a text block.
     */
    splitBlock(): string;
    /**
     * Returns `{doc, selection, version}`.
     */
    stateJSON(): string;
    /**
     * Toggles a mark on the selection or in the stored marks.
     *
     * # Errors
     *
     * Returns a problem object when the mark or an attribute is unknown.
     */
    toggleMark(name: string, attrs?: string | null): string;
    /**
     * Starts a transaction against the current document version.
     */
    transaction(): Transaction;
    /**
     * Undoes the latest group.
     *
     * # Errors
     *
     * Returns a problem object when the inverse step no longer applies.
     */
    undo(): string;
    /**
     * Updates the composed text.
     *
     * # Errors
     *
     * Returns a problem object when no composition is open or the text is too large.
     */
    updateComposition(text: string): string;
    /**
     * Converts a UTF-16 offset inside a block to a document position.
     */
    utf16ToPos(block: number, utf16: number): number | undefined;
    /**
     * Returns the accessibility report.
     */
    validateA11y(): string;
    /**
     * Returns the structural validation report.
     */
    validate(): string;
    /**
     * Returns the current document version.
     */
    version(): number;
    /**
     * Returns the render-oriented `DocView` projection.
     *
     * # Errors
     *
     * Returns a problem object only if the projection cannot be encoded.
     */
    viewJSON(): string;
}

/**
 * Ordered batch operations with a base version and history metadata.
 */
export class Transaction {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Adds a mark to a range.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     */
    addMark(from: number, to: number, mark: string): void;
    /**
     * Returns the document version the transaction was built against.
     */
    baseVersion(): number;
    /**
     * Returns whether the editor already consumed the transaction.
     */
    consumed(): boolean;
    /**
     * Deletes a byte range.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     */
    delete(from: number, to: number): void;
    /**
     * Inserts plain text at a position.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     */
    insertText(at: number, text: string): void;
    /**
     * Joins the block containing a position with its previous sibling.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     */
    joinBackward(at: number): void;
    /**
     * Removes a mark from a range.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     */
    removeMark(from: number, to: number, mark: string): void;
    /**
     * Replaces a range with a slice.
     *
     * # Errors
     *
     * Returns a problem object when the slice is not valid JSON.
     */
    replace(from: number, to: number, slice: string): void;
    /**
     * Sets whether the transaction enters history.
     */
    setAddToHistory(add: boolean): void;
    /**
     * Changes the type of a top-level block.
     *
     * # Errors
     *
     * Returns a problem object when the attributes are not valid JSON.
     */
    setBlockType(at: number, kind: string, attrs?: string | null): void;
    /**
     * Sets one document-root attribute.
     *
     * # Errors
     *
     * Returns a problem object when the value is not valid JSON.
     */
    setDocAttr(name: string, value: string): void;
    /**
     * Sets the history group identifier.
     */
    setGroup(group: number): void;
    /**
     * Sets one attribute on an inline atom.
     *
     * # Errors
     *
     * Returns a problem object when the value is not valid JSON.
     */
    setInlineAttr(from: number, to: number, name: string, value: string): void;
    /**
     * Sets one attribute on runs that already carry a mark.
     *
     * # Errors
     *
     * Returns a problem object when the value is not valid JSON.
     */
    setMarkAttr(from: number, to: number, mark: string, name: string, value: string): void;
    /**
     * Sets one attribute on a top-level block.
     *
     * # Errors
     *
     * Returns a problem object when the value is not valid JSON.
     */
    setNodeAttr(at: number, name: string, value: string): void;
    /**
     * Sets or clears the stable identifier of a block.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     */
    setNodeId(at: number, id?: string | null): void;
    /**
     * Sets the origin used by the history group.
     *
     * # Errors
     *
     * Returns a problem object when the origin name is unknown.
     */
    setOrigin(origin: string): void;
    /**
     * Sets the selection without creating history.
     *
     * # Errors
     *
     * Returns a problem object when the selection is not valid JSON.
     */
    setSelection(selection: string): void;
    /**
     * Splits the block containing a position.
     *
     * # Errors
     *
     * Returns a problem object when the transaction was already consumed.
     */
    splitBlock(at: number): void;
}

/**
 * Returns the semantic accessibility tree as canonical JSON.
 *
 * # Errors
 *
 * Returns a problem object when the input is not a document accepted by the full schema.
 */
export function a11y_tree(input: string): string;

/**
 * Canonicalizes a document JSON string.
 *
 * # Errors
 *
 * Returns a problem object when the input is not a document accepted by the full schema.
 */
export function canonicalize(input: string): string;

/**
 * Validates and canonicalizes a durable envelope, returning canonical envelope JSON.
 *
 * # Errors
 *
 * Returns a problem object when the envelope format, fingerprint, or document is invalid.
 */
export function canonicalize_envelope(input: string): string;

/**
 * Deletes a byte range and returns canonical JSON.
 *
 * # Errors
 *
 * Returns a problem object when the range or input is invalid.
 */
export function delete_range(input: string, from: number, to: number): string;

/**
 * Validates and canonicalizes a durable envelope, returning its document JSON.
 *
 * # Errors
 *
 * Returns a problem object when the envelope format, fingerprint, or document is invalid.
 */
export function envelope_doc(input: string): string;

/**
 * Inserts text at a byte position and returns canonical JSON.
 *
 * # Errors
 *
 * Returns a problem object when the position or input is invalid.
 */
export function insert_text(input: string, position: number, text: string): string;

/**
 * Joins the block at a byte boundary with its previous sibling.
 *
 * # Errors
 *
 * Returns a problem object when the position or input is invalid.
 */
export function join_backward(input: string, position: number): string;

/**
 * Returns the plain text of a document.
 *
 * # Errors
 *
 * Returns a problem object when the input is not a document accepted by the full schema.
 */
export function plain_text(input: string): string;

/**
 * Returns the canonical BLAKE3 fingerprint of the full schema.
 *
 * # Errors
 *
 * Returns a problem object only if the full preset is inconsistent.
 */
export function schema_fingerprint(): string;

/**
 * Splits the block at a byte position and returns canonical JSON.
 *
 * # Errors
 *
 * Returns a problem object when the position or input is invalid.
 */
export function split_block(input: string, position: number): string;

/**
 * Toggles a named mark over a byte range and returns canonical JSON.
 *
 * # Errors
 *
 * Returns a problem object when the mark, range, or input is invalid.
 */
export function toggle_mark(input: string, from: number, to: number, mark: string): string;

/**
 * Returns stable accessibility error codes, or `ok`.
 *
 * # Errors
 *
 * Returns a problem object when the input is not a document accepted by the full schema.
 */
export function validate_a11y(input: string): string;

/**
 * Returns the crate version exposed to JavaScript consumers.
 */
export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_editor_free: (a: number, b: number) => void;
    readonly __wbg_transaction_free: (a: number, b: number) => void;
    readonly a11y_tree: (a: number, b: number) => [number, number, number, number];
    readonly canonicalize: (a: number, b: number) => [number, number, number, number];
    readonly canonicalize_envelope: (a: number, b: number) => [number, number, number, number];
    readonly delete_range: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly editor_apply: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_applyTransaction: (a: number, b: number) => [number, number, number, number];
    readonly editor_beginComposition: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_blockAt: (a: number, b: number) => [number, number];
    readonly editor_blockRange: (a: number, b: number) => [number, number];
    readonly editor_blocks: (a: number) => number;
    readonly editor_canRedo: (a: number) => number;
    readonly editor_canUndo: (a: number) => number;
    readonly editor_cancelComposition: (a: number) => [number, number, number, number];
    readonly editor_clearHistory: (a: number) => void;
    readonly editor_commitComposition: (a: number) => [number, number, number, number];
    readonly editor_copyRange: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_cutRange: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_deleteBackward: (a: number) => [number, number, number, number];
    readonly editor_deleteForward: (a: number) => [number, number, number, number];
    readonly editor_deleteRange: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_deleteSelection: (a: number) => [number, number, number, number];
    readonly editor_docJSON: (a: number) => [number, number];
    readonly editor_dryRun: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_fromEnvelope: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly editor_fromJSON: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly editor_historyJSON: (a: number) => [number, number];
    readonly editor_import: (a: number, b: number, c: number) => [number, number];
    readonly editor_insertHardBreak: (a: number) => [number, number, number, number];
    readonly editor_insertText: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_isComposing: (a: number) => number;
    readonly editor_new: () => [number, number, number];
    readonly editor_pasteSlice: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly editor_posToUtf16: (a: number, b: number) => number;
    readonly editor_redo: (a: number) => [number, number, number, number];
    readonly editor_replaceSlice: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly editor_schemaFingerprint: (a: number) => [number, number];
    readonly editor_setBlockType: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly editor_setDocAttr: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly editor_setInlineAttr: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly editor_setNodeAttr: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly editor_setNodeId: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly editor_setSelection: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_snapToScalar: (a: number, b: number) => number;
    readonly editor_splitBlock: (a: number) => [number, number, number, number];
    readonly editor_stateJSON: (a: number) => [number, number];
    readonly editor_toggleMark: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly editor_transaction: (a: number) => number;
    readonly editor_undo: (a: number) => [number, number, number, number];
    readonly editor_updateComposition: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_utf16ToPos: (a: number, b: number, c: number) => number;
    readonly editor_validate: (a: number) => [number, number];
    readonly editor_validateA11y: (a: number) => [number, number];
    readonly editor_version: (a: number) => number;
    readonly editor_viewJSON: (a: number) => [number, number, number, number];
    readonly envelope_doc: (a: number, b: number) => [number, number, number, number];
    readonly insert_text: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly join_backward: (a: number, b: number, c: number) => [number, number, number, number];
    readonly plain_text: (a: number, b: number) => [number, number, number, number];
    readonly schema_fingerprint: () => [number, number, number, number];
    readonly split_block: (a: number, b: number, c: number) => [number, number, number, number];
    readonly toggle_mark: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly transaction_addMark: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly transaction_baseVersion: (a: number) => number;
    readonly transaction_consumed: (a: number) => number;
    readonly transaction_delete: (a: number, b: number, c: number) => [number, number];
    readonly transaction_insertText: (a: number, b: number, c: number, d: number) => [number, number];
    readonly transaction_joinBackward: (a: number, b: number) => [number, number];
    readonly transaction_removeMark: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly transaction_replace: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly transaction_setAddToHistory: (a: number, b: number) => void;
    readonly transaction_setBlockType: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly transaction_setDocAttr: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly transaction_setGroup: (a: number, b: number) => void;
    readonly transaction_setInlineAttr: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly transaction_setMarkAttr: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number];
    readonly transaction_setNodeAttr: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly transaction_setNodeId: (a: number, b: number, c: number, d: number) => [number, number];
    readonly transaction_setOrigin: (a: number, b: number, c: number) => [number, number];
    readonly transaction_setSelection: (a: number, b: number, c: number) => [number, number];
    readonly transaction_splitBlock: (a: number, b: number) => [number, number];
    readonly validate_a11y: (a: number, b: number) => [number, number, number, number];
    readonly version: () => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
