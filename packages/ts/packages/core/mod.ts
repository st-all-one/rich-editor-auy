import wasmInit, {
  a11y_tree as wasmA11yTree,
  canonicalize as wasmCanonicalize,
  canonicalize_envelope as wasmCanonicalizeEnvelope,
  delete_range as wasmDeleteRange,
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

export interface AuyError {
  readonly code: string;
  readonly message: string;
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
  readonly ordered?: boolean;
  readonly scope?: string;
  readonly colspan?: number;
  readonly rowspan?: number;
  readonly range: { readonly from: number; readonly to: number };
  readonly children: readonly A11yNode[];
}

export class DocumentError extends Error implements AuyError {
  readonly code: string;

  constructor(message: string, code = "auy_error") {
    super(message);
    this.name = "DocumentError";
    this.code = code;
  }
}

export class Document {
  #json: string;

  private constructor(json: string) {
    this.#json = json;
  }

  static fromJSON(json: string): Document {
    return new Document(wasmCanonicalize(json));
  }

  static fromEnvelope(_schema: unknown, bytes: string | Uint8Array): Document {
    const text = typeof bytes === "string"
      ? bytes
      : new TextDecoder().decode(bytes);
    return new Document(wasmEnvelopeDoc(text));
  }

  static fromEnvelopeBytes(
    _schema: unknown,
    bytes: string | Uint8Array,
  ): { document: Document; envelope: string } {
    const text = typeof bytes === "string"
      ? bytes
      : new TextDecoder().decode(bytes);
    return {
      document: new Document(wasmEnvelopeDoc(text)),
      envelope: wasmCanonicalizeEnvelope(text),
    };
  }

  toJSON(): string {
    return this.#json;
  }

  plainText(): string {
    return wasmPlainText(this.#json);
  }

  insertText(position: number, text: string): Document {
    return new Document(wasmInsertText(this.#json, position, text));
  }

  deleteRange(from: number, to: number): Document {
    return new Document(wasmDeleteRange(this.#json, from, to));
  }

  splitBlock(position: number): Document {
    return new Document(wasmSplitBlock(this.#json, position));
  }

  joinBackward(position: number): Document {
    return new Document(wasmJoinBackward(this.#json, position));
  }

  toggleMark(from: number, to: number, mark: string): Document {
    return new Document(wasmToggleMark(this.#json, from, to, mark));
  }

  a11yTree(): A11yNode {
    return JSON.parse(wasmA11yTree(this.#json)) as A11yNode;
  }

  validateA11y(): readonly string[] {
    const result = wasmValidateA11y(this.#json);
    return result === "ok" ? [] : result.split(",");
  }
}

export class EditorState {
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

let initialized = false;

export async function init(): Promise<void> {
  if (!initialized) {
    await wasmInit();
    initialized = true;
  }
}

export function fingerprint(): string {
  return wasmFingerprint();
}

export function version(): string {
  return wasmVersion();
}

export function canonicalize(json: string): string {
  return wasmCanonicalize(json);
}

export function plainText(json: string): string {
  return wasmPlainText(json);
}

export function validateA11y(json: string): readonly string[] {
  const result = wasmValidateA11y(json);
  return result === "ok" ? [] : result.split(",");
}
