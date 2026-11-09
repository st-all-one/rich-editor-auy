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
     * Applies `{"ops": [...]}` atomically.
     *
     * # Errors
     *
     * Returns a problem object when any operation is rejected.
     */
    apply(payload: string): string;
    /**
     * Returns the number of top-level blocks.
     */
    blocks(): number;
    /**
     * Discards every undo and redo group.
     */
    clearHistory(): void;
    /**
     * Returns the canonical document JSON.
     */
    docJSON(): string;
    /**
     * Validates `{"ops": [...]}` without mutating the editor.
     *
     * # Errors
     *
     * Returns a problem object only when the payload is not valid JSON.
     */
    dryRun(payload: string): string;
    /**
     * Creates an editor from a durable envelope.
     *
     * # Errors
     *
     * Returns a problem object when the envelope format, fingerprint, or document is invalid.
     */
    static fromEnvelope(input: string): Editor;
    /**
     * Creates an editor from canonical document JSON.
     *
     * # Errors
     * Returns a problem object when the document is rejected by the full schema.
     */
    static fromJSON(input: string): Editor;
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
     * Creates an empty editor using the complete schema.
     *
     * # Errors
     * Returns a problem object only if the full preset is inconsistent.
     */
    constructor();
    /**
     * Redoes the latest undone group.
     *
     * # Errors
     *
     * Returns a problem object when the step no longer applies.
     */
    redo(): string;
    /**
     * Returns the active schema fingerprint.
     */
    schemaFingerprint(): string;
    /**
     * Sets the selection without creating history.
     *
     * # Errors
     *
     * Returns a problem object when a position lies beyond the document.
     */
    setSelection(from: number, to: number): string;
    /**
     * Returns `{doc, selection, version}`.
     */
    stateJSON(): string;
    /**
     * Undoes the latest group.
     *
     * # Errors
     *
     * Returns a problem object when the inverse step no longer applies.
     */
    undo(): string;
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
    readonly a11y_tree: (a: number, b: number) => [number, number, number, number];
    readonly canonicalize: (a: number, b: number) => [number, number, number, number];
    readonly canonicalize_envelope: (a: number, b: number) => [number, number, number, number];
    readonly delete_range: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly editor_apply: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_blocks: (a: number) => number;
    readonly editor_clearHistory: (a: number) => void;
    readonly editor_docJSON: (a: number) => [number, number];
    readonly editor_dryRun: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_fromEnvelope: (a: number, b: number) => [number, number, number];
    readonly editor_fromJSON: (a: number, b: number) => [number, number, number];
    readonly editor_historyJSON: (a: number) => [number, number];
    readonly editor_import: (a: number, b: number, c: number) => [number, number];
    readonly editor_new: () => [number, number, number];
    readonly editor_redo: (a: number) => [number, number, number, number];
    readonly editor_schemaFingerprint: (a: number) => [number, number];
    readonly editor_setSelection: (a: number, b: number, c: number) => [number, number, number, number];
    readonly editor_stateJSON: (a: number) => [number, number];
    readonly editor_undo: (a: number) => [number, number, number, number];
    readonly editor_version: (a: number) => number;
    readonly editor_viewJSON: (a: number) => [number, number, number, number];
    readonly envelope_doc: (a: number, b: number) => [number, number, number, number];
    readonly insert_text: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly join_backward: (a: number, b: number, c: number) => [number, number, number, number];
    readonly plain_text: (a: number, b: number) => [number, number, number, number];
    readonly schema_fingerprint: () => [number, number, number, number];
    readonly split_block: (a: number, b: number, c: number) => [number, number, number, number];
    readonly toggle_mark: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
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
