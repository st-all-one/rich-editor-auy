/*
 * Trecho de análise — MarkText / Muya (@muyajs/core).
 * Origem: ref/marktext/packages/muya/src/block/base/content.ts, linhas 356–434.
 * Commit de referência: a520bef (2026-11-06). Pacote: @muyajs/core 0.2.0.
 * Licença: MIT (ref/marktext/LICENSE).
 *
 * Por que este trecho: é a definição da folha editável do documento — o único lugar
 * onde texto do bloco vira DOM contenteditable e onde a escrita vira operação
 * `ot-text-unicode` ancorada no caminho do bloco dentro da árvore de estado.
 *
 * Importações e helpers do arquivo original (TreeNode, Muya, diff, diffToTextOp,
 * Nullable, TBlockPath, TSelectionState) foram omitidos; o corpo abaixo é literal.
 */

/* ---- ref/marktext/packages/muya/src/block/base/content.ts:356 ---- */

class Content extends TreeNode {
    private _text: string;
    protected isComposed: boolean;
    private _compositionAnchor: Nullable<Text> = null;
    private _compositionLineEnd: Nullable<Element> = null;

    static override blockName = 'content';

    protected get hasSelection() {
        return !!this.getCursor();
    }

    protected get selection() {
        return this.muya.editor.selection;
    }

    protected get inlineEngine() {
        return this.muya.editor.inlineEngine;
    }

    protected get autoPairType() {
        return this.blockName;
    }

    get path(): TBlockPath {
        if (this.parent == null)
            return ['text'];

        const { path: pPath } = this.parent;

        return [...pPath, 'text'];
    }

    get text() {
        return this._text;
    }

    set text(text) {
        if (!this.muya.canMutate)
            return;
        const oldText = this._text;
        if (oldText !== text)
            this.transactions.captureSelection();
        this._text = text;
        const { path } = this;
        if (this.blockName === 'language-input') {
            path.pop();
            path.push('meta', 'lang');
        }

        if (oldText !== text) {
            const diffs = diff(oldText, text);

            this.transactions.editOperation(path, diffToTextOp(diffs));
        }
    }

    protected get isCollapsed() {
        const { isCollapsed } = this.getCursor() ?? {};

        return isCollapsed;
    }

    get isContainerBlock() {
        return false;
    }

    constructor(muya: Muya, text: string) {
        super(muya);

        this.tagName = 'span';
        this.classList = ['mu-content'];
        this.attributes = {
            contenteditable: !muya.isReadOnly,
        };
        this._text = text;
        this.isComposed = false;
    }

    /* ---- fim do trecho ---- */
