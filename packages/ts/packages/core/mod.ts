import wasmInit, {
  a11y_tree as wasmA11yTree,
  canonicalize as wasmCanonicalize,
  canonicalize_envelope as wasmCanonicalizeEnvelope,
  delete_range as wasmDeleteRange,
  Editor as WasmEditor,
  envelope_doc as wasmEnvelopeDoc,
  insert_text as wasmInsertText,
  join_backward as wasmJoinBackward,
  plain_text as wasmPlainText,
  schema_fingerprint as wasmFingerprint,
  split_block as wasmSplitBlock,
  toggle_mark as wasmToggleMark,
  validate_a11y as wasmValidateA11y,
  version as wasmVersion,
} from "./wasm/rich_editor_auy_wasm.js";

/** RFC 9457 problem details returned by the ingestion boundary. */
export interface ProblemDetails {
  readonly type: string;
  readonly title: string;
  readonly status: number;
  readonly detail: string;
  readonly instance?: string;
  readonly code: string;
  readonly suggestion?: string;
}

export interface AuyError {
  readonly code: string;
  readonly message: string;
}

/** Attribute map keyed by schema names, never numeric ids. */
export type AttrMap = Readonly<Record<string, unknown>>;

/** Mark as exposed to renderers. */
export interface MarkView {
  readonly type: string;
  readonly attrs?: AttrMap;
}

/** Inline text run with absolute byte range. */
export interface RunView {
  readonly text: string;
  readonly range: readonly [number, number];
  readonly marks: readonly MarkView[];
}

/** Inline atom with schema name and attributes. */
export interface AtomView {
  readonly kind: string;
  readonly attrs: AttrMap;
  readonly range: readonly [number, number];
}

/** Render-oriented block projection; independent from `A11yNode`. */
export interface BlockView {
  readonly index: number;
  readonly kind: string;
  readonly id?: string;
  readonly attrs: AttrMap;
  readonly range: readonly [number, number];
  readonly text?: string;
  readonly runs?: readonly RunView[];
  readonly atoms?: readonly AtomView[];
  readonly children?: readonly BlockView[];
}

/** Complete render projection returned by {@link Editor.view}. */
export interface DocView {
  readonly version: number;
  readonly attrs: AttrMap;
  readonly blocks: readonly BlockView[];
}

/** Canonical document JSON. */
export type DocumentJSON = Record<string, unknown>;

/** Serialized table cell reference. */
export interface CellRefJSON {
  readonly table: number;
  readonly row: number;
  readonly column: number;
}

/** Serialized selection. */
export type SelectionJSON =
  | { readonly type: "none" }
  | { readonly type: "caret"; readonly pos: number; readonly assoc?: -1 | 1 }
  | {
    readonly type: "text";
    readonly anchor: number;
    readonly head: number;
    readonly assoc?: -1 | 1;
  }
  | { readonly type: "node"; readonly pos: number }
  | {
    readonly type: "table";
    readonly table: number;
    readonly anchor: CellRefJSON;
    readonly head: CellRefJSON;
  };

export interface RangeJSON {
  readonly from: number;
  readonly to: number;
}

export interface ChangeSetJSON {
  readonly blocks: RangeJSON;
  readonly ranges: readonly {
    readonly old: RangeJSON;
    readonly new: RangeJSON;
  }[];
  readonly selection: boolean;
  readonly full: boolean;
}

export interface HistoryJSON {
  readonly can_undo: boolean;
  readonly can_redo: boolean;
  readonly undo_depth: number;
  readonly redo_depth: number;
  readonly bytes: number;
  readonly last_origin?: string;
}

/** Observable editor state returned by mutations. */
export interface EditorStateJSON {
  readonly doc: DocumentJSON;
  readonly selection: SelectionJSON;
  readonly version: number;
  readonly changes?: ChangeSetJSON;
  readonly history?: HistoryJSON;
}

export interface InsertTextOp {
  readonly op: "insert_text";
  readonly at: number;
  readonly text: string;
}

export interface DeleteRangeOp {
  readonly op: "delete_range";
  readonly from: number;
  readonly to: number;
}

export interface SplitBlockOp {
  readonly op: "split_block";
  readonly at: number;
}

export interface JoinBackwardOp {
  readonly op: "join_backward";
  readonly at: number;
}

export interface ToggleMarkOp {
  readonly op: "toggle_mark";
  readonly from: number;
  readonly to: number;
  readonly mark: string;
}

export interface SetBlockAttrOp {
  readonly op: "set_block_attr";
  readonly at: number;
  readonly name: string;
  readonly value: unknown;
}

export interface SetDocAttrOp {
  readonly op: "set_doc_attr";
  readonly name: string;
  readonly value: unknown;
}

export interface SetInlineAttrOp {
  readonly op: "set_inline_attr";
  readonly from: number;
  readonly to: number;
  readonly name: string;
  readonly value: unknown;
}

export interface SetMarkAttrOp {
  readonly op: "set_mark_attr";
  readonly from: number;
  readonly to: number;
  readonly mark: string;
  readonly name: string;
  readonly value: unknown;
}

export interface ReplaceDocumentOp {
  readonly op: "replace_document";
  readonly doc: DocumentJSON;
}

export interface FragmentJSON {
  readonly blocks: readonly unknown[];
}

export interface ReplaceFragmentOp {
  readonly op: "replace_fragment";
  readonly from?: number;
  readonly to?: number;
  readonly at?: number;
  readonly fragment: FragmentJSON;
}

export interface SetSelectionOp {
  readonly op: "set_selection";
  readonly selection: SelectionJSON;
}

export type BatchOp =
  | InsertTextOp
  | DeleteRangeOp
  | SplitBlockOp
  | JoinBackwardOp
  | ToggleMarkOp
  | SetBlockAttrOp
  | SetDocAttrOp
  | SetInlineAttrOp
  | SetMarkAttrOp
  | ReplaceDocumentOp
  | ReplaceFragmentOp
  | SetSelectionOp;

export interface BatchPayload {
  readonly ops: readonly BatchOp[];
}

export interface ImportDocumentPayload {
  readonly mode: "document";
  readonly schema?: string;
  readonly doc: DocumentJSON;
}

export interface ImportFragmentPayload {
  readonly mode: "fragment" | "replace" | "insert";
  readonly schema?: string;
  readonly fragment: FragmentJSON;
  readonly from?: number;
  readonly to?: number;
  readonly at?: number;
}

export interface ImportOpsPayload {
  readonly mode: "ops";
  readonly schema?: string;
  readonly base_version?: number;
  readonly ops: readonly BatchOp[];
}

export type ImportPayload =
  | ImportDocumentPayload
  | ImportFragmentPayload
  | ImportOpsPayload;

export interface ImportReport {
  readonly accepted: boolean;
  readonly blocks: number;
  readonly version: number;
  readonly problems: readonly ProblemDetails[];
  readonly state: EditorStateJSON;
}

export interface A11yNode {
  readonly path: string;
  readonly role: string;
  readonly id?: string;
  readonly name?: string;
  readonly description?: string;
  readonly alt?: string;
  readonly lang?: string;
  readonly dir?: string;
  readonly level?: number;
  readonly text?: string;
  readonly marks?: readonly string[];
  readonly href?: string;
  readonly src?: string;
  readonly caption?: string;
  readonly transcript?: string;
  readonly code_language?: string;
  readonly ordered?: boolean;
  readonly scope?: string;
  readonly colspan?: number;
  readonly rowspan?: number;
  readonly range: { readonly from: number; readonly to: number };
  readonly children: readonly A11yNode[];
}

/** Anything that can project its document into the semantic tree. */
export interface A11yTreeProvider {
  a11yTree(): A11yNode;
}

/** Stateful or stateless source accepted by the DOM adapter. */
export interface StateProvider extends A11yTreeProvider {
  doc(): Document;
}

/** Parses a structured boundary failure when the thrown value is a problem object. */
function parseProblem(value: string): ProblemDetails | undefined {
  try {
    const parsed = JSON.parse(value) as Partial<ProblemDetails>;
    if (typeof parsed.code === "string") return parsed as ProblemDetails;
    return undefined;
  } catch {
    return undefined;
  }
}

function toDocumentError(error: unknown): DocumentError {
  if (error instanceof DocumentError) return error;
  if (typeof error === "string") {
    const problem = parseProblem(error);
    if (problem !== undefined) {
      return new DocumentError(
        problem.detail || problem.title || problem.code,
        problem.code,
        problem,
      );
    }
    return new DocumentError(error);
  }
  if (error instanceof Error) return new DocumentError(error.message);
  return new DocumentError(String(error));
}

function guarded<T>(run: () => T): T {
  try {
    return run();
  } catch (error) {
    throw toDocumentError(error);
  }
}

function payloadText(payload: BatchPayload | ImportPayload | string): string {
  return typeof payload === "string" ? payload : JSON.stringify(payload);
}

export class DocumentError extends Error implements AuyError {
  readonly code: string;
  readonly type?: string;
  readonly status?: number;
  readonly detail: string;
  readonly instance?: string;
  readonly suggestion?: string;

  constructor(message: string, code = "auy_error", problem?: ProblemDetails) {
    super(message);
    this.name = "DocumentError";
    this.code = code;
    this.type = problem?.type;
    this.status = problem?.status;
    this.detail = problem?.detail ?? message;
    this.instance = problem?.instance;
    this.suggestion = problem?.suggestion;
  }
}

export class Document implements A11yTreeProvider {
  #json: string;

  private constructor(json: string) {
    this.#json = json;
  }

  static fromJSON(json: string): Document {
    return new Document(guarded(() => wasmCanonicalize(json)));
  }

  static fromEnvelope(_schema: unknown, bytes: string | Uint8Array): Document {
    const text = typeof bytes === "string"
      ? bytes
      : new TextDecoder().decode(bytes);
    return new Document(guarded(() => wasmEnvelopeDoc(text)));
  }

  static fromEnvelopeBytes(
    _schema: unknown,
    bytes: string | Uint8Array,
  ): { document: Document; envelope: string } {
    const text = typeof bytes === "string"
      ? bytes
      : new TextDecoder().decode(bytes);
    return {
      document: new Document(guarded(() => wasmEnvelopeDoc(text))),
      envelope: guarded(() => wasmCanonicalizeEnvelope(text)),
    };
  }

  toJSON(): string {
    return this.#json;
  }

  plainText(): string {
    return guarded(() => wasmPlainText(this.#json));
  }

  insertText(position: number, text: string): Document {
    return new Document(
      guarded(() => wasmInsertText(this.#json, position, text)),
    );
  }

  deleteRange(from: number, to: number): Document {
    return new Document(guarded(() => wasmDeleteRange(this.#json, from, to)));
  }

  splitBlock(position: number): Document {
    return new Document(guarded(() => wasmSplitBlock(this.#json, position)));
  }

  joinBackward(position: number): Document {
    return new Document(guarded(() => wasmJoinBackward(this.#json, position)));
  }

  toggleMark(from: number, to: number, mark: string): Document {
    return new Document(
      guarded(() => wasmToggleMark(this.#json, from, to, mark)),
    );
  }

  a11yTree(): A11yNode {
    return JSON.parse(guarded(() => wasmA11yTree(this.#json))) as A11yNode;
  }

  validateA11y(): readonly string[] {
    const result = guarded(() => wasmValidateA11y(this.#json));
    return result === "ok" ? [] : result.split(",");
  }
}

/** Stateless convenience wrapper kept for one-shot document edits. */
export class EditorState implements StateProvider {
  #doc: Document;

  private constructor(doc: Document) {
    this.#doc = doc;
  }

  static fromEnvelope(
    schema: unknown,
    bytes: string | Uint8Array,
  ): EditorState {
    return new EditorState(Document.fromEnvelope(schema, bytes));
  }

  static fromJSON(json: string): EditorState {
    return new EditorState(Document.fromJSON(json));
  }

  doc(): Document {
    return this.#doc;
  }

  insertText(position: number, text: string): EditorState {
    return new EditorState(this.#doc.insertText(position, text));
  }

  deleteRange(from: number, to: number): EditorState {
    return new EditorState(this.#doc.deleteRange(from, to));
  }

  splitBlock(position: number): EditorState {
    return new EditorState(this.#doc.splitBlock(position));
  }

  joinBackward(position: number): EditorState {
    return new EditorState(this.#doc.joinBackward(position));
  }

  toggleMark(from: number, to: number, mark: string): EditorState {
    return new EditorState(this.#doc.toggleMark(from, to, mark));
  }

  a11yTree(): A11yNode {
    return this.#doc.a11yTree();
  }
}

/**
 * Stateful editor over the canonical document.
 *
 * Mutations are atomic transactions against the full schema; rejected batches leave the
 * editor unchanged and surface as {@link DocumentError} carrying RFC 9457 problem details.
 */
export class Editor implements StateProvider {
  #inner: WasmEditor;

  constructor(inner?: WasmEditor) {
    this.#inner = inner ?? guarded(() => new WasmEditor());
  }

  static fromJSON(json: string): Editor {
    return new Editor(guarded(() => WasmEditor.fromJSON(json)));
  }

  static fromEnvelope(bytes: string | Uint8Array): Editor {
    const text = typeof bytes === "string"
      ? bytes
      : new TextDecoder().decode(bytes);
    return new Editor(guarded(() => WasmEditor.fromEnvelope(text)));
  }

  /** Applies an atomic operation batch. */
  apply(payload: BatchPayload | string): EditorStateJSON {
    return JSON.parse(
      guarded(() => this.#inner.apply(payloadText(payload))),
    ) as EditorStateJSON;
  }

  /** Validates a batch without mutating the editor. */
  dryRun(payload: BatchPayload | string): ImportReport {
    return JSON.parse(
      guarded(() => this.#inner.dryRun(payloadText(payload))),
    ) as ImportReport;
  }

  /** Imports a document, fragment, or operation batch atomically. */
  import(payload: ImportPayload | string): ImportReport {
    return JSON.parse(
      guarded(() => this.#inner.import(payloadText(payload))),
    ) as ImportReport;
  }

  undo(): EditorStateJSON & { readonly applied: boolean } {
    return JSON.parse(
      guarded(() => this.#inner.undo()),
    ) as EditorStateJSON & { readonly applied: boolean };
  }

  redo(): EditorStateJSON & { readonly applied: boolean } {
    return JSON.parse(
      guarded(() => this.#inner.redo()),
    ) as EditorStateJSON & { readonly applied: boolean };
  }

  /** Returns the current document. */
  doc(): Document {
    return Document.fromJSON(this.#inner.docJSON());
  }

  /** Returns the full editor state, suitable for `JSON.stringify`. */
  toJSON(): EditorStateJSON {
    return JSON.parse(this.#inner.stateJSON()) as EditorStateJSON;
  }

  /** Returns the render projection with names, ranges, runs, atoms and children. */
  view(): DocView {
    return JSON.parse(this.#inner.viewJSON()) as DocView;
  }

  version(): number {
    return this.#inner.version();
  }

  blocks(): number {
    return this.#inner.blocks();
  }

  history(): HistoryJSON {
    return JSON.parse(this.#inner.historyJSON()) as HistoryJSON;
  }

  schemaFingerprint(): string {
    return this.#inner.schemaFingerprint();
  }

  setSelection(from: number, to: number): EditorStateJSON {
    return JSON.parse(
      guarded(() => this.#inner.setSelection(from, to)),
    ) as EditorStateJSON;
  }

  clearHistory(): void {
    this.#inner.clearHistory();
  }

  a11yTree(): A11yNode {
    return this.doc().a11yTree();
  }
}

let initialized = false;

export async function init(): Promise<void> {
  if (!initialized) {
    await wasmInit();
    initialized = true;
  }
}

export function fingerprint(): string {
  return guarded(() => wasmFingerprint());
}

export function version(): string {
  return wasmVersion();
}

export function canonicalize(json: string): string {
  return guarded(() => wasmCanonicalize(json));
}

export function plainText(json: string): string {
  return guarded(() => wasmPlainText(json));
}

export function validateA11y(json: string): readonly string[] {
  const result = guarded(() => wasmValidateA11y(json));
  return result === "ok" ? [] : result.split(",");
}
