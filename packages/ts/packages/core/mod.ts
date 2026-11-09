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
  type Transaction as WasmTransaction,
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

/** Slice payload produced by copying a range. */
export interface SliceJSON {
  readonly text?: string;
  readonly runs?: readonly unknown[];
  readonly blocks?: readonly unknown[];
  /** One when the first copied block is only partial. */
  readonly openStart?: 0 | 1;
  /** One when the last copied block is only partial. */
  readonly openEnd?: 0 | 1;
}

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
  readonly insertedBlocks: readonly number[];
  readonly removedBlocks: readonly number[];
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
export interface AppliedJSON {
  readonly selection: SelectionJSON;
  readonly version: number;
  readonly changes: ChangeSetJSON;
  readonly history: HistoryJSON;
}

export interface AppliedResultJSON extends AppliedJSON {
  readonly applied: boolean;
}

export interface EditorOptions {
  /** Replacement `[maxGroups, maxBytes]` history budgets. */
  readonly historyLimits?: readonly [number, number];
}

export interface ValidationIssueJSON {
  readonly invariant: string;
  readonly path: string;
  readonly position?: number;
}

export interface ValidationReport {
  readonly ok: boolean;
  readonly issues: readonly ValidationIssueJSON[];
}

export interface A11yIssueJSON {
  readonly code: string;
  readonly severity: "error" | "warning";
  readonly path: string;
  readonly range: RangeJSON;
}

export interface A11yReportJSON {
  readonly ok: boolean;
  readonly errors: readonly A11yIssueJSON[];
  readonly warnings: readonly A11yIssueJSON[];
}

export interface BlockRefJSON {
  readonly index: number;
  readonly from: number;
  readonly to: number;
}

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

export interface SetBlockTypeOp {
  readonly op: "set_block_type";
  readonly at: number;
  readonly kind: string;
  readonly attrs?: AttrMap;
}

export interface ReplaceSliceOp {
  readonly op: "replace_slice";
  readonly from: number;
  readonly to: number;
  readonly slice: SliceJSON;
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
  | SetSelectionOp
  | SetBlockTypeOp
  | ReplaceSliceOp;

export interface BatchPayload {
  readonly ops: readonly BatchOp[];
  /** Document version the batch was computed against. */
  readonly baseVersion?: number;
  /** History origin name. */
  readonly origin?: string;
  /** Explicit history group identifier. */
  readonly group?: number;
  /** Whether the batch enters history; defaults to `true`. */
  readonly addToHistory?: boolean;
}

/** Anchor policy applied to an imported payload. */
export type IdPolicy = "preserve" | "regenerate" | "reject";

/** Resource limits applied to one import payload. */
export interface BatchLimits {
  readonly maxBytes?: number;
  readonly maxBlocks?: number;
  readonly maxDepth?: number;
  readonly maxNodes?: number;
  readonly maxRunsPerBlock?: number;
  /** Maximum operations in one `ops` import. */
  readonly maxOps?: number;
  /** Maximum table cells in the resulting document. */
  readonly maxTableCells?: number;
}

/** Ingestion policy for anchors and resource limits. */
export interface ImportOptions {
  readonly idPolicy?: IdPolicy;
  /** Required by `idPolicy: "regenerate"`. */
  readonly idSeed?: string;
  /** Whether the import enters history; defaults to `true`. */
  readonly addToHistory?: boolean;
  readonly limits?: BatchLimits;
}

/** Fields shared by every import payload. */
export interface ImportEnvelope {
  readonly schema?: string;
  readonly options?: ImportOptions;
  /** Echoed back in the report for correlation. */
  readonly requestId?: string;
  /** Free-form origin label echoed back in the report. */
  readonly source?: string;
  /** Free-form destination label echoed back in the report. */
  readonly destination?: string;
}

export interface ImportDocumentPayload extends ImportEnvelope {
  readonly mode: "document";
  readonly doc: DocumentJSON;
}

export interface ImportFragmentPayload extends ImportEnvelope {
  readonly mode: "fragment" | "replace" | "insert";
  readonly fragment: FragmentJSON;
  readonly from?: number;
  readonly to?: number;
  readonly at?: number;
}

export interface ImportOpsPayload extends ImportEnvelope {
  readonly mode: "ops";
  readonly base_version?: number;
  readonly ops: readonly BatchOp[];
}

export type ImportPayload =
  | ImportDocumentPayload
  | ImportFragmentPayload
  | ImportOpsPayload;

/** One anchor rewrite produced by `idPolicy: "regenerate"`. */
export interface IdPair {
  readonly from: string;
  readonly to: string;
}

export interface ImportReport {
  readonly accepted: boolean;
  /** Top-level blocks after the attempt. */
  readonly blocks: number;
  /** Top-level blocks accepted from the payload. */
  readonly acceptedBlocks: number;
  readonly version: number;
  /** Schema fingerprint the payload was applied against. */
  readonly schema: string;
  readonly requestId: string | null;
  readonly source: string | null;
  readonly destination: string | null;
  readonly idMap: readonly IdPair[];
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

/**
 * Editor surface the DOM adapter drives.
 *
 * Every method is a core operation: the adapter translates DOM input into these calls and never
 * edits the rendered tree itself. `Editor` satisfies this interface.
 */
export interface EditorLike extends StateProvider {
  apply(payload: BatchPayload | string): AppliedJSON;
  pasteSlice(at: number, slice: SliceJSON | string): AppliedJSON;
  replaceSlice(
    from: number,
    to: number,
    slice: SliceJSON | string,
  ): AppliedJSON;
  deleteRange(from: number, to: number): AppliedJSON;
  setSelection(from: number, to: number): AppliedJSON;
  undo(): AppliedResultJSON;
  redo(): AppliedResultJSON;
  beginComposition(from: number, to: number): AppliedJSON;
  updateComposition(text: string): AppliedJSON;
  commitComposition(): AppliedJSON;
  cancelComposition(): AppliedJSON;
  isComposing(): boolean;
  version(): number;
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
 * Composable transaction against one document version.
 *
 * The editor consumes a transaction on {@link Editor.applyTransaction}; reuse is rejected and a
 * transaction built against another version fails with a stale-batch problem.
 */
export class Transaction {
  #inner: WasmTransaction;

  /** Builds a transaction around a WASM handle. */
  constructor(inner: WasmTransaction) {
    this.#inner = inner;
  }

  insertText(at: number, text: string): this {
    guarded(() => this.#inner.insertText(at, text));
    return this;
  }

  delete(from: number, to: number): this {
    guarded(() => this.#inner.delete(from, to));
    return this;
  }

  replace(from: number, to: number, slice: SliceJSON): this {
    guarded(() => this.#inner.replace(from, to, JSON.stringify(slice)));
    return this;
  }

  splitBlock(at: number): this {
    guarded(() => this.#inner.splitBlock(at));
    return this;
  }

  joinBackward(at: number): this {
    guarded(() => this.#inner.joinBackward(at));
    return this;
  }

  setBlockType(at: number, kind: string, attrs?: AttrMap): this {
    guarded(() =>
      this.#inner.setBlockType(
        at,
        kind,
        attrs ? JSON.stringify(attrs) : undefined,
      )
    );
    return this;
  }

  addMark(from: number, to: number, mark: string): this {
    guarded(() => this.#inner.addMark(from, to, mark));
    return this;
  }

  removeMark(from: number, to: number, mark: string): this {
    guarded(() => this.#inner.removeMark(from, to, mark));
    return this;
  }

  setNodeAttr(at: number, name: string, value: unknown): this {
    guarded(() => this.#inner.setNodeAttr(at, name, JSON.stringify(value)));
    return this;
  }

  setInlineAttr(
    from: number,
    to: number,
    name: string,
    value: unknown,
  ): this {
    guarded(() =>
      this.#inner.setInlineAttr(from, to, name, JSON.stringify(value))
    );
    return this;
  }

  setMarkAttr(
    from: number,
    to: number,
    mark: string,
    name: string,
    value: unknown,
  ): this {
    guarded(() =>
      this.#inner.setMarkAttr(from, to, mark, name, JSON.stringify(value))
    );
    return this;
  }

  setDocAttr(name: string, value: unknown): this {
    guarded(() => this.#inner.setDocAttr(name, JSON.stringify(value)));
    return this;
  }

  setNodeId(at: number, id: string | null): this {
    guarded(() => this.#inner.setNodeId(at, id ?? undefined));
    return this;
  }

  setSelection(selection: SelectionJSON): this {
    guarded(() => this.#inner.setSelection(JSON.stringify(selection)));
    return this;
  }

  setOrigin(origin: string): this {
    guarded(() => this.#inner.setOrigin(origin));
    return this;
  }

  setGroup(group: number): this {
    this.#inner.setGroup(group);
    return this;
  }

  setAddToHistory(add: boolean): this {
    this.#inner.setAddToHistory(add);
    return this;
  }

  baseVersion(): number {
    return this.#inner.baseVersion();
  }

  consumed(): boolean {
    return this.#inner.consumed();
  }

  /** Returns the WASM handle for {@link Editor.applyTransaction}. */
  handle(): WasmTransaction {
    return this.#inner;
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

  static fromJSON(json: string, options?: EditorOptions): Editor {
    const text = options ? JSON.stringify(options) : undefined;
    return new Editor(guarded(() => WasmEditor.fromJSON(json, text)));
  }

  static fromEnvelope(
    bytes: string | Uint8Array,
    options?: EditorOptions,
  ): Editor {
    const text = typeof bytes === "string"
      ? bytes
      : new TextDecoder().decode(bytes);
    const optionsText = options ? JSON.stringify(options) : undefined;
    return new Editor(
      guarded(() => WasmEditor.fromEnvelope(text, optionsText)),
    );
  }

  /** Applies an atomic operation batch. */
  apply(payload: BatchPayload | string): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.apply(payloadText(payload))),
    ) as AppliedJSON;
  }

  /** Validates a document, fragment, or batch without mutating the editor. */
  dryRun(payload: BatchPayload | ImportPayload | string): ImportReport {
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

  undo(): AppliedResultJSON {
    return JSON.parse(
      guarded(() => this.#inner.undo()),
    ) as AppliedResultJSON;
  }

  redo(): AppliedResultJSON {
    return JSON.parse(
      guarded(() => this.#inner.redo()),
    ) as AppliedResultJSON;
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

  /** Starts an IME composition over a range. */
  beginComposition(from: number, to: number): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.beginComposition(from, to)),
    ) as AppliedJSON;
  }

  /** Updates the composed text; intermediate states never enter history. */
  updateComposition(text: string): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.updateComposition(text)),
    ) as AppliedJSON;
  }

  /** Commits the composition as one undo group. */
  commitComposition(): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.commitComposition()),
    ) as AppliedJSON;
  }

  /** Cancels the composition and restores the original text. */
  cancelComposition(): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.cancelComposition()),
    ) as AppliedJSON;
  }

  /** Returns whether an IME composition is open. */
  isComposing(): boolean {
    return this.#inner.isComposing();
  }

  /** Copies a range and returns the slice payload. */
  copyRange(from: number, to: number): SliceJSON {
    return JSON.parse(
      guarded(() => this.#inner.copyRange(from, to)),
    ) as SliceJSON;
  }

  /** Inserts a slice, replacing the range when one is given. */
  pasteSlice(at: number, slice: SliceJSON | string): AppliedJSON {
    const payload = typeof slice === "string" ? slice : JSON.stringify(slice);
    return JSON.parse(
      guarded(() => this.#inner.pasteSlice(at, payload)),
    ) as AppliedJSON;
  }

  /** Replaces a range with a slice. */
  replaceSlice(
    from: number,
    to: number,
    slice: SliceJSON | string,
  ): AppliedJSON {
    const payload = typeof slice === "string" ? slice : JSON.stringify(slice);
    return JSON.parse(
      guarded(() => this.#inner.replaceSlice(from, to, payload)),
    ) as AppliedJSON;
  }

  /** Deletes a range with block semantics and returns the applied state. */
  deleteRange(from: number, to: number): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.deleteRange(from, to)),
    ) as AppliedJSON;
  }

  /** Copies a range and deletes it. */
  cutRange(
    from: number,
    to: number,
  ): { readonly slice: SliceJSON; readonly state: AppliedJSON } {
    return JSON.parse(
      guarded(() => this.#inner.cutRange(from, to)),
    ) as { readonly slice: SliceJSON; readonly state: AppliedJSON };
  }

  setSelection(selection: SelectionJSON): AppliedJSON;
  setSelection(from: number, to: number): AppliedJSON;
  setSelection(selection: SelectionJSON | number, to?: number): AppliedJSON {
    if (typeof selection === "number") {
      return JSON.parse(
        guarded(() => this.#inner.setSelection(selection, to ?? selection)),
      ) as AppliedJSON;
    }
    return this.apply({ ops: [{ op: "set_selection", selection }] });
  }

  clearHistory(): void {
    this.#inner.clearHistory();
  }

  /** Starts a transaction against the current version. */
  transaction(): Transaction {
    return new Transaction(this.#inner.transaction());
  }

  /**
   * Applies and consumes a transaction.
   *
   * @throws {DocumentError} with a `stale_batch` or `consumed_transaction` problem when the
   * version moved or the transaction was already applied.
   */
  applyTransaction(transaction: Transaction): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.applyTransaction(transaction.handle())),
    ) as AppliedJSON;
  }

  /** Inserts text at the selection, replacing it first. */
  insertText(text: string): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.insertText(text)),
    ) as AppliedJSON;
  }

  /** Deletes one scalar before the caret. */
  deleteBackward(): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.deleteBackward()),
    ) as AppliedJSON;
  }

  /** Deletes one scalar after the caret. */
  deleteForward(): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.deleteForward()),
    ) as AppliedJSON;
  }

  /** Deletes the selected range. */
  deleteSelection(): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.deleteSelection()),
    ) as AppliedJSON;
  }

  /** Splits the block at the caret. */
  splitBlock(): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.splitBlock()),
    ) as AppliedJSON;
  }

  /** Inserts a hard break at the selection. */
  insertHardBreak(): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.insertHardBreak()),
    ) as AppliedJSON;
  }

  /** Changes the type of the block at the caret. */
  setBlockType(kind: string, attrs?: AttrMap): AppliedJSON {
    return JSON.parse(
      guarded(() =>
        this.#inner.setBlockType(
          kind,
          attrs ? JSON.stringify(attrs) : undefined,
        )
      ),
    ) as AppliedJSON;
  }

  /** Toggles a mark on the selection. */
  toggleMark(name: string, attrs?: AttrMap): AppliedJSON {
    return JSON.parse(
      guarded(() =>
        this.#inner.toggleMark(name, attrs ? JSON.stringify(attrs) : undefined)
      ),
    ) as AppliedJSON;
  }

  /** Sets one attribute on the block at the caret. */
  setNodeAttr(name: string, value: unknown): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.setNodeAttr(name, JSON.stringify(value))),
    ) as AppliedJSON;
  }

  /** Sets one attribute on the atom covered by the selection. */
  setInlineAttr(name: string, value: unknown): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.setInlineAttr(name, JSON.stringify(value))),
    ) as AppliedJSON;
  }

  /** Sets one document-root attribute. */
  setDocAttr(name: string, value: unknown): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.setDocAttr(name, JSON.stringify(value))),
    ) as AppliedJSON;
  }

  /** Sets or clears the stable identifier of a block. */
  setNodeId(at: number, id: string | null): AppliedJSON {
    return JSON.parse(
      guarded(() => this.#inner.setNodeId(at, id ?? undefined)),
    ) as AppliedJSON;
  }

  /** Returns whether undo is available. */
  canUndo(): boolean {
    return this.#inner.canUndo();
  }

  /** Returns whether redo is available. */
  canRedo(): boolean {
    return this.#inner.canRedo();
  }

  /** Returns the structural validation report. */
  validate(): ValidationReport {
    return JSON.parse(this.#inner.validate()) as ValidationReport;
  }

  /** Returns the accessibility report. */
  validateA11yReport(): A11yReportJSON {
    return JSON.parse(this.#inner.validateA11y()) as A11yReportJSON;
  }

  /** Converts a document position to UTF-16 code units. */
  posToUtf16(pos: number): number | undefined {
    return this.#inner.posToUtf16(pos);
  }

  /** Converts a UTF-16 offset inside a block to a document position. */
  utf16ToPos(block: number, utf16: number): number | undefined {
    return this.#inner.utf16ToPos(block, utf16);
  }

  /** Snaps a position to the previous scalar boundary. */
  snapToScalar(pos: number): number {
    return this.#inner.snapToScalar(pos);
  }

  /** Returns the block containing a position. */
  blockAt(pos: number): BlockRefJSON | undefined {
    const value = this.#inner.blockAt(pos);
    return value === undefined ? undefined : JSON.parse(value) as BlockRefJSON;
  }

  /** Returns the range of one top-level block. */
  blockRange(index: number): RangeJSON | undefined {
    const value = this.#inner.blockRange(index);
    return value === undefined ? undefined : JSON.parse(value) as RangeJSON;
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
