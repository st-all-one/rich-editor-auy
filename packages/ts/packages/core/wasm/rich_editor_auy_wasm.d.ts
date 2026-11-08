/* tslint:disable */
/* eslint-disable */

/**
 * Returns the semantic accessibility tree as canonical JSON.
 */
export function a11y_tree(input: string): string;

/**
 * Canonicalizes a document JSON string.
 */
export function canonicalize(input: string): string;

/**
 * Validates and canonicalizes a durable envelope, returning canonical envelope JSON.
 */
export function canonicalize_envelope(input: string): string;

/**
 * Deletes a byte range and returns canonical JSON.
 */
export function delete_range(input: string, from: number, to: number): string;

/**
 * Validates and canonicalizes a durable envelope, returning its document JSON.
 */
export function envelope_doc(input: string): string;

/**
 * Inserts text at a byte position and returns canonical JSON.
 */
export function insert_text(input: string, position: number, text: string): string;

/**
 * Joins the block at a byte boundary with its previous sibling.
 */
export function join_backward(input: string, position: number): string;

/**
 * Returns the plain text of a document.
 */
export function plain_text(input: string): string;

/**
 * Returns the canonical BLAKE3 schema fingerprint.
 */
export function schema_fingerprint(): string;

/**
 * Splits the block at a byte position and returns canonical JSON.
 */
export function split_block(input: string, position: number): string;

/**
 * Toggles a named mark over a byte range and returns canonical JSON.
 */
export function toggle_mark(input: string, from: number, to: number, mark: string): string;

/**
 * Returns stable accessibility error codes, or `ok`.
 */
export function validate_a11y(input: string): string;

/**
 * Returns the crate version exposed to JavaScript consumers.
 */
export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly a11y_tree: (a: number, b: number) => [number, number, number, number];
    readonly canonicalize: (a: number, b: number) => [number, number, number, number];
    readonly canonicalize_envelope: (a: number, b: number) => [number, number, number, number];
    readonly delete_range: (a: number, b: number, c: number, d: number) => [number, number, number, number];
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
