import {
  type AppliedJSON,
  type BlockView,
  DocumentError,
  type DocumentJSON,
  Editor,
  type FragmentJSON,
  type ImportPayload,
  type ImportReport,
  init,
  type SliceJSON,
} from "../../packages/ts/packages/core/mod.ts";
import {
  describeChange,
  mount,
  type MountedEditor,
} from "../../packages/ts/packages/dom/mod.ts";

function element<T extends Element>(selector: string): T {
  const found = document.querySelector<T>(selector);
  if (found === null) throw new Error(`missing ${selector}`);
  return found;
}

const host = element<HTMLElement>("#editor");
const status = element<HTMLElement>("#status");
const changePanel = element<HTMLElement>("#change");
const reportPanel = element<HTMLElement>("#report");
const selectionPanel = element<HTMLElement>("#selection");
const jsonPanel = element<HTMLElement>("#json");
const viewPanel = element<HTMLElement>("#view");
const a11yPanel = element<HTMLElement>("#a11y");
const importArea = element<HTMLTextAreaElement>("#import-json");
const importReport = element<HTMLElement>("#import-report");

let editor!: Editor;
let mounted!: MountedEditor;
let lastApplied: AppliedJSON | null = null;

interface ServerReport {
  readonly ok: boolean;
  readonly exit: number;
  readonly stdout: string;
  readonly stderr: string;
  readonly issues: readonly {
    readonly code: string;
    readonly offset: number;
    readonly path: string;
  }[];
}

function setStatus(
  message: string,
  kind: "ok" | "error" | "info" = "info",
): void {
  status.textContent = message;
  status.className = kind;
}

function showError(error: unknown): void {
  const message = error instanceof DocumentError
    ? `${error.code}: ${error.message}`
    : error instanceof Error
    ? error.message
    : String(error);
  reportPanel.textContent = error instanceof DocumentError && error.detail
    ? `${message}\n\n${error.detail}`
    : message;
  setStatus(message, "error");
}

function decode(text: string): Editor {
  return text.trimStart().startsWith('{"format"')
    ? Editor.fromEnvelope(text)
    : Editor.fromJSON(text);
}

function documentFromText(text: string): DocumentJSON {
  const value = JSON.parse(text) as { readonly doc?: DocumentJSON };
  if (value.doc === undefined) return value as DocumentJSON;
  return value.doc;
}

function envelope(): string {
  const state = editor.toJSON();
  return JSON.stringify({
    format: "rich-editor-auy",
    version: 2,
    schema_fingerprint: editor.schemaFingerprint(),
    doc: state.doc,
  });
}

function selectionStart(): number {
  const selection = editor.selection();
  switch (selection.type) {
    case "caret":
      return selection.pos;
    case "text":
      return Math.min(selection.anchor, selection.head);
    case "node":
      return selection.pos;
    case "none":
    case "table":
      return 0;
  }
}

function selectionRange():
  | { readonly from: number; readonly to: number }
  | undefined {
  const selection = editor.selection();
  switch (selection.type) {
    case "caret":
      return { from: selection.pos, to: selection.pos };
    case "text":
      return {
        from: Math.min(selection.anchor, selection.head),
        to: Math.max(selection.anchor, selection.head),
      };
    case "node":
      return { from: selection.pos, to: selection.pos };
    case "none":
    case "table":
      return undefined;
  }
}

function blockBoundaryAt(pos: number): number {
  const block = editor.blockAt(pos);
  if (block === undefined) {
    return editor.blockRange(Math.max(0, editor.blocks() - 1))?.to ?? 0;
  }
  return editor.blockRange(block.index)?.to ?? pos;
}

function isApplied(value: unknown): value is AppliedJSON {
  return typeof value === "object" && value !== null &&
    "changes" in value && "selection" in value;
}

function problemText(report: ImportReport): string {
  return report.problems
    .map((problem) => `${problem.code}: ${problem.detail}`)
    .join("\n");
}

function sync(): void {
  mounted.dispatch(editor);
  syncPanels();
}

function syncPanels(): void {
  const state = editor.toJSON();
  changePanel.textContent = lastApplied === null
    ? "—"
    : describeChange(lastApplied.changes) ?? "sem alteração de texto";
  selectionPanel.textContent = JSON.stringify(state.selection, null, 2);
  jsonPanel.textContent = JSON.stringify(state, null, 2);
  viewPanel.textContent = JSON.stringify(editor.view(), null, 2);
  a11yPanel.textContent = JSON.stringify(editor.a11yTree(), null, 2);
}

function findTextBlock(blocks: readonly BlockView[]): BlockView | undefined {
  for (const block of blocks) {
    if (
      block.kind === "paragraph" || block.kind === "heading" ||
      block.kind === "code_block" || block.kind === "raw_code"
    ) {
      return block;
    }
    if (block.children !== undefined) {
      const nested = findTextBlock(block.children);
      if (nested !== undefined) return nested;
    }
  }
  return undefined;
}

function ensureTextSelection(): void {
  let pos = selectionStart();
  let block = editor.blockAt(pos);
  if (block === undefined) {
    const candidate = findTextBlock(editor.view().blocks);
    if (candidate === undefined) return;
    pos = candidate.range[0];
    block = editor.blockAt(pos);
  }
  if (block === undefined) return;
  const range = editor.blockRange(block.index);
  if (range === undefined) return;
  const caret = Math.min(Math.max(pos, range.from), range.to);
  editor.setSelection(caret, caret);
}

function setBlock(kind: string, attrs?: Record<string, unknown>): void {
  mutate(`Bloco ${kind}`, () => {
    ensureTextSelection();
    return editor.setBlockType(kind, attrs);
  });
}

function mutate(label: string, run: () => unknown): void {
  try {
    const result = run();
    if (isApplied(result)) lastApplied = result;
    sync();
    setStatus(label, "ok");
  } catch (error) {
    showError(error);
  }
}

function toggleMark(name: string, attrs?: Record<string, unknown>): void {
  mutate(`Marca ${name}`, () => editor.toggleMark(name, attrs));
}

function insertBlocks(blocks: readonly unknown[]): ImportReport {
  const report = editor.import({
    mode: "insert",
    at: blockBoundaryAt(selectionStart()),
    fragment: { blocks },
  });
  if (!report.accepted) throw new Error(problemText(report));
  lastApplied = null;
  return report;
}

function tableBlock(rows = 3, cols = 3): Record<string, unknown> {
  const cells = Array.from(
    { length: rows },
    (_, row) =>
      Array.from({ length: cols }, (_, column) => ({
        type: row === 0 ? "table_header" : "table_cell",
        attrs: row === 0 ? { scope: "col" } : {},
        content: [{
          type: "paragraph",
          content: [{ type: "text", text: `c${row + 1}·${column + 1}` }],
        }],
      })),
  );
  return {
    type: "table",
    attrs: { summary: "Tabela inserida pela demo", col_widths: [50, 50, 50] },
    content: [
      {
        type: "table_caption",
        content: [{ type: "text", text: "Nova tabela" }],
      },
      ...cells.map((rowCells) => ({ type: "table_row", content: rowCells })),
    ],
  };
}

function listBlock(ordered: boolean): Record<string, unknown> {
  return {
    type: ordered ? "ordered_list" : "bullet_list",
    attrs: ordered ? { start: 1, type: "1", tight: true } : { tight: true },
    content: [1, 2].map((number) => ({
      type: "list_item",
      content: [{
        type: "paragraph",
        content: [{ type: "text", text: `Item ${number}` }],
      }],
    })),
  };
}

function quoteBlock(): Record<string, unknown> {
  return {
    type: "blockquote",
    content: [{
      type: "paragraph",
      content: [{ type: "text", text: "Citação editável inserida pela demo" }],
    }],
  };
}

function figureBlock(): Record<string, unknown> {
  return {
    type: "figure",
    content: [
      {
        type: "image_block",
        attrs: {
          src: "/assets/demo.svg",
          alt: "Diagrama inserido pela demo",
          width: 240,
          height: 120,
          caption: "Figura inserida",
          credit: "demo",
        },
      },
      {
        type: "caption",
        content: [{ type: "text", text: "Figura com legenda editável" }],
      },
    ],
  };
}

function reportJson(report: ImportReport): string {
  return JSON.stringify(
    {
      accepted: report.accepted,
      blocks: report.blocks,
      acceptedBlocks: report.acceptedBlocks,
      version: report.version,
      problems: report.problems,
      idMap: report.idMap,
    },
    null,
    2,
  );
}

async function refresh(): Promise<void> {
  const response = await fetch("/api/document", { cache: "no-store" });
  if (!response.ok) throw new Error(`GET /api/document: ${response.status}`);
  const document = documentFromText(await response.text());
  const report = editor.import({ mode: "document", doc: document });
  if (!report.accepted) throw new Error(problemText(report));
  lastApplied = null;
  sync();
  setStatus("Documento recarregado do PHP", "ok");
}

async function save(): Promise<void> {
  setStatus("Enviando envelope canônico ao PHP…");
  const response = await fetch("/api/document", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: envelope(),
  });
  const text = await response.text();
  reportPanel.textContent = text;
  if (!response.ok) {
    setStatus(`PHP rejeitou (${response.status})`, "error");
    return;
  }
  setStatus("Salvo no PHP; validação Rust verde", "ok");
}

async function validateOnServer(): Promise<void> {
  setStatus("Validando no CLI Rust via PHP…");
  const response = await fetch("/api/validate", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: envelope(),
  });
  const report = await response.json() as ServerReport;
  reportPanel.textContent = JSON.stringify(
    {
      server: report,
      local: {
        structural: editor.validate(),
        a11y: editor.validateA11yReport(),
      },
    },
    null,
    2,
  );
  setStatus(
    report.ok ? "Rust: documento válido" : "Rust: documento inválido",
    report.ok ? "ok" : "error",
  );
}

async function canonicalizeOnServer(): Promise<void> {
  setStatus("Canonicalizando no CLI Rust…");
  const response = await fetch("/api/canonical", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: envelope(),
  });
  const result = await response.json() as {
    readonly ok: boolean;
    readonly bytes?: string;
    readonly stderr?: string;
  };
  reportPanel.textContent = JSON.stringify(result, null, 2);
  setStatus(
    result.ok
      ? "Rust: bytes canônicos gerados"
      : "Rust: falha ao canonicalizar",
    result.ok ? "ok" : "error",
  );
}

async function copySelection(cut: boolean): Promise<void> {
  const range = selectionRange();
  if (range === undefined) {
    throw new Error("selecione texto ou um bloco primeiro");
  }
  if (cut) {
    const result = editor.cutRange(range.from, range.to);
    lastApplied = result.state;
    await navigator.clipboard.writeText(JSON.stringify(result.slice));
  } else {
    const slice: SliceJSON = editor.slice(range.from, range.to);
    await navigator.clipboard.writeText(JSON.stringify(slice));
  }
  sync();
  setStatus(
    cut ? "Recortado para o clipboard" : "Copiado para o clipboard",
    "ok",
  );
}

function isSlice(value: unknown): value is SliceJSON {
  if (typeof value !== "object" || value === null) return false;
  return "text" in value || "runs" in value || "blocks" in value;
}

async function paste(): Promise<void> {
  const text = await navigator.clipboard.readText();
  const range = selectionRange();
  let slice: unknown;
  try {
    slice = JSON.parse(text);
  } catch {
    slice = undefined;
  }
  if (isSlice(slice)) {
    lastApplied = range !== undefined && range.from !== range.to
      ? editor.replaceSlice(range.from, range.to, slice)
      : editor.insertSlice(range?.from ?? selectionStart(), slice);
  } else {
    lastApplied = editor.insertText(text);
  }
  sync();
  setStatus("Colado pelo core", "ok");
}

function clearMarks(): void {
  const range = selectionRange();
  if (range === undefined) throw new Error("selecione um intervalo primeiro");
  const transaction = editor.transaction();
  for (
    const mark of [
      "strong",
      "em",
      "underline",
      "strike",
      "code",
      "highlight",
      "sup",
      "sub",
      "link",
      "ref",
    ]
  ) {
    transaction.removeMark(range.from, range.to, mark);
  }
  lastApplied = editor.applyTransaction(transaction);
  sync();
  setStatus("Marcas removidas", "ok");
}

function importPayload(dryRun: boolean): void {
  const text = importArea.value.trim();
  if (text === "") throw new Error("cole um payload JSON no campo abaixo");
  const payload = JSON.parse(text) as ImportPayload;
  const report = dryRun ? editor.dryRun(payload) : editor.import(payload);
  importReport.textContent = reportJson(report);
  if (!dryRun) sync();
  setStatus(
    dryRun
      ? "Dry-run concluído"
      : report.accepted
      ? "Importação aceita"
      : "Importação rejeitada",
    report.accepted ? "ok" : "error",
  );
}

const actions: Readonly<Record<string, () => void>> = {
  undo: () => mutate("Undo", () => editor.undo()),
  redo: () => mutate("Redo", () => editor.redo()),
  "clear-history": () => {
    editor.clearHistory();
    sync();
    setStatus("Histórico limpo", "ok");
  },
  strong: () => toggleMark("strong"),
  em: () => toggleMark("em"),
  underline: () => toggleMark("underline"),
  strike: () => toggleMark("strike"),
  code: () => toggleMark("code"),
  highlight: () => toggleMark("highlight"),
  sup: () => toggleMark("sup"),
  sub: () => toggleMark("sub"),
  link: () => {
    const href = window.prompt("URL do link", "https://example.com");
    if (href === null || href.trim() === "") return;
    toggleMark("link", { href: href.trim(), target: "_blank" });
  },
  "clear-marks": () => {
    try {
      clearMarks();
    } catch (error) {
      showError(error);
    }
  },
  paragraph: () => setBlock("paragraph"),
  h1: () => setBlock("heading", { level: 1 }),
  h2: () => setBlock("heading", { level: 2 }),
  h3: () => setBlock("heading", { level: 3 }),
  bullet: () =>
    mutate("Lista com marcadores", () => insertBlocks([listBlock(false)])),
  ordered: () =>
    mutate("Lista ordenada", () => insertBlocks([listBlock(true)])),
  quote: () => mutate("Citação", () => insertBlocks([quoteBlock()])),
  "code-block": () =>
    setBlock("code_block", { language: "ts", line_numbers: true }),
  "raw-code": () =>
    setBlock("raw_code", {
      language: "html",
      format: "html",
      filename: "inline.html",
      line_numbers: true,
    }),
  "hard-break": () => mutate("Quebra de linha", () => editor.insertHardBreak()),
  table: () => mutate("Tabela inserida", () => insertBlocks([tableBlock()])),
  figure: () => mutate("Figura inserida", () => insertBlocks([figureBlock()])),
  separator: () =>
    mutate("Separador inserido", () => insertBlocks([{ type: "separator" }])),
  copy: () => {
    void copySelection(false).catch(showError);
  },
  cut: () => {
    void copySelection(true).catch(showError);
  },
  paste: () => {
    void paste().catch(showError);
  },
  save: () => {
    void save().catch(showError);
  },
  load: () => {
    void refresh().catch(showError);
  },
  reset: () => {
    void fetch("/api/reset", { method: "POST" })
      .then(() => refresh())
      .catch(showError);
  },
  validate: () => {
    void validateOnServer().catch(showError);
  },
  canonical: () => {
    void canonicalizeOnServer().catch(showError);
  },
  view: () => {
    viewPanel.parentElement?.setAttribute("open", "");
    setStatus("DocView atualizado", "ok");
  },
  a11y: () => {
    a11yPanel.parentElement?.setAttribute("open", "");
    setStatus("A11yTree atualizada", "ok");
  },
  export: () => {
    jsonPanel.textContent = JSON.stringify(
      {
        envelope: JSON.parse(envelope()),
        state: editor.toJSON(),
        view: editor.view(),
        a11y: editor.a11yTree(),
      },
      null,
      2,
    );
    setStatus("JSON exportado", "ok");
  },
  import: () => {
    try {
      importPayload(false);
    } catch (error) {
      showError(error);
    }
  },
  "dry-run": () => {
    try {
      importPayload(true);
    } catch (error) {
      showError(error);
    }
  },
};

function bindToolbar(): void {
  for (
    const button of document.querySelectorAll<HTMLButtonElement>(
      "[data-action]",
    )
  ) {
    button.addEventListener("mousedown", (event) => event.preventDefault());
    button.addEventListener("click", () => {
      const action = actions[button.dataset.action ?? ""];
      if (action !== undefined) action();
    });
  }
}

async function boot(): Promise<void> {
  await init();
  const response = await fetch("/api/document", { cache: "no-store" });
  if (!response.ok) throw new Error(`GET /api/document: ${response.status}`);
  editor = decode(await response.text());
  mounted = mount(host, {
    state: editor,
    editor,
    label: "Documento AUY",
    onInput: () => syncPanels(),
  });
  bindToolbar();
  syncPanels();
  setStatus(
    `Pronto · ${
      editor.schemaFingerprint().slice(0, 12)
    }… · ${editor.blocks()} blocos`,
    "ok",
  );
}

boot().catch(showError);
