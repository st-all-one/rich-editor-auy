/*
 * Cópia de análise — MarkText / Muya (@muyajs/core).
 * Origem 1: ref/marktext/packages/muya/src/inline/index.ts  (arquivo completo, 95 linhas)
 * Origem 2: ref/marktext/packages/muya/src/inline/referenceDefinitions.ts (arquivo completo, 72 linhas)
 * Commit de referência: a520bef (2026-11-06). Versão declarada do pacote: @muyajs/core 0.2.0.
 * Licença: MIT (ref/marktext/LICENSE, "Copyright (c) 2017-present Luo Ran / 2018-present MarkText Contributors").
 *
 * Por que estes dois arquivos: mostram (a) o ponto único que re-renderiza o conteúdo inline
 * de um bloco e (b) como definições de referência `[ref]: url` são indexadas por varredura do
 * estado do documento, sem serem um tipo de bloco próprio.
 *
 * Nada aqui é código executável sem o resto do pacote (snabbdom, lexer próprio, KaTeX etc.).
 */

/* ============================================================================
 * PARTE 1 — src/inline/index.ts
 * ========================================================================== */

import type Format from '../block/base/format';
import type { Muya } from '../muya';
import type { IRenderCursor } from '../selection/types';
import type { IParagraphState } from '../state/types';
import type { IHighlight, Labels } from './model/types';
import { prepareReadingLinks } from '../editor/readOnly';
import logger from '../utils/logger';
import { tokenizer } from './lexer';
import { profileForBlock } from './parser/profiles';
import { getReferenceDefinition, ReferenceDefinitionIndex } from './referenceDefinitions';
import Renderer from './renderer';
import { ImageMounts } from './resources/imageMounts';

const debug = logger('inlineEngine:');

class InlineEngine {
    private _labels: Labels = new Map();

    get labels(): Labels {
        return this._labels;
    }

    public renderer: Renderer;
    private _references = new ReferenceDefinitionIndex();

    constructor(public muya: Muya) {
        this.renderer = new Renderer(() => {
            const selected = muya.editor.selection?.image;
            return {
                root: muya.domNode,
                readOnly: muya.isReadOnly,
                softNewlineAsSpace: muya.options.softNewlineAsSpace,
                references: this.labels,
                selectedImage: selected ? { block: selected.block, range: { ...selected.imageInfo.token.range } } : null,
                translate: text => muya.i18n.t(text),
            };
        }, new ImageMounts(muya.domNode));
    }

    private _tokenizer(block: Format, highlights: IHighlight[]) {
        const { options } = this.muya;
        const { text } = block;
        const { labels } = this;

        return tokenizer(text, { profile: profileForBlock(block.blockName), labels, options, highlights });
    }

    /** Invalidates pending and completed loads before repainting all image occurrences. */
    invalidateImageCache() {
        this.renderer.invalidateImages();

        const { scrollPage } = this.muya.editor;
        if (!scrollPage)
            return;

        scrollPage.breadthFirstTraverse((node) => {
            if (node.isContent())
                node.update();
        });
    }

    setImagePreview(src: string, preview: string): void {
        this.renderer.images.setPreview(src, preview);
    }

    dispose(): void {
        this.renderer.dispose();
    }

    patch(block: Format, cursor?: IRenderCursor, highlights: IHighlight[] = []) {
        this.collectReferenceDefinitions();
        const { domNode } = block;
        if (block.isParent())
            debug.error('Patch can only handle content block');

        const tokens = this._tokenizer(block, highlights);
        const renderCursor = !this.muya.isReadOnly && cursor?.block === block ? cursor : {};
        const html = this.renderer.output(tokens, block, renderCursor);
        domNode!.innerHTML = html;

        if (this.muya.isReadOnly)
            prepareReadingLinks(domNode!);
    }

    collectReferenceDefinitions(): Labels {
        this._labels = this._references.collect(this.muya.editor.transactions.candidateState, state => this.getLabelInfo(state));
        return this.labels;
    }

    getLabelInfo(blockOrState: Pick<IParagraphState, 'text'>) {
        return getReferenceDefinition(blockOrState);
    }
}

export default InlineEngine;

/* ============================================================================
 * PARTE 2 — src/inline/referenceDefinitions.ts
 * ========================================================================== */

import type { IParagraphState, TState } from '../state/types';
import type { Labels, ReferenceDefinitionToken } from './model/types';
import { decodeMarkdownString } from '../utils/markdownEscape';
import { beginRules } from './parser/rules';
import { normalizeReferenceLabel } from './referenceLabel';

export function parseReferenceDefinition(text: string): Omit<ReferenceDefinitionToken, 'type' | 'parent' | 'range'> | null {
    const match = beginRules.reference_definition.exec(text);
    if (!match)
        return null;
    return {
        raw: match[0],
        leftBracket: match[1],
        label: match[2],
        backlash: match[3] || '',
        rightBracket: match[4],
        leftHrefMarker: match[5] || '',
        href: match[6],
        rightHrefMarker: match[7] || '',
        leftTitleSpace: match[8],
        titleMarker: match[9] || '',
        title: match[10] || '',
        rightTitleSpace: match[11] || '',
    };
}

export function getReferenceDefinition(blockOrState: Pick<IParagraphState, 'text'>) {
    const definition = parseReferenceDefinition(blockOrState.text);
    if (!definition)
        return { label: null, info: null };
    return {
        label: normalizeReferenceLabel(definition.label + definition.backlash),
        info: {
            href: decodeMarkdownString(definition.href),
            title: decodeMarkdownString(definition.title),
        },
    };
}

export class ReferenceDefinitionIndex {
    private _labelsState: readonly TState[] | null = null;
    private _labels: Labels = new Map();

    // Definitions live in paragraphs across the document; rescanning for every block caused #4887.
    collect(state: readonly TState[], read = getReferenceDefinition): Labels {
        if (this._labelsState === state)
            return this._labels;

        const labels = new Map();
        const stack: Iterator<TState>[] = [state[Symbol.iterator]()];
        while (stack.length) {
            const next = stack[stack.length - 1].next();
            if (next.done) {
                stack.pop();
                continue;
            }
            const state = next.value;
            if (state.name === 'paragraph') {
                const { label, info } = read(state);
                if (label && info && !labels.has(label))
                    labels.set(label, info);
            }
            else if ('children' in state && Array.isArray(state.children)) {
                stack.push(state.children[Symbol.iterator]());
            }
        }

        this._labels = labels;
        this._labelsState = state;
        return labels;
    }
}
