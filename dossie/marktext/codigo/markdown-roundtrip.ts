/*
 * Trechos de análise — MarkText / Muya (@muyajs/core).
 * Origem A: ref/marktext/packages/muya/src/state/markdownToState.ts, linhas 56–200 (contínuo,
 *           a partir de `export class MarkdownToState`).
 * Origem B: ref/marktext/packages/muya/src/state/stateToMarkdown.ts: declaração da classe (linha 67)
 *           + corpo de `generate`/`_convertStatesToMarkdown` (linhas 108–169). Campos e construtor
 *           (linhas 68–106) foram omitidos; o restante é literal.
 * Commit de referência: a520bef (2026-11-06). Pacote: @muyajs/core 0.2.0.
 * Licença: MIT (ref/marktext/LICENSE).
 *
 * Por que estes trechos: são o "round-trip" Markdown → árvore de blocos → Markdown.
 * A (A) mostra que o parser de blocos delega ao `marked` e converte tokens em TState
 * com uma pilha de pais; a (B) mostra que a serialização é feita por tipo de bloco,
 * com um resolvedor de separação entre blocos. Juntas explicam por que o Markdown
 * sobrevive à edição, mas também por que pequenas normalizações escapam.
 *
 * Importações omitidas; os corpos abaixo são literais.
 */

/* ============================================================================
 * PARTE A — src/state/markdownToState.ts:56
 * ========================================================================== */

export class MarkdownToState {
    constructor(private _options: Partial<IMarkdownToStateOptions> = DEFAULT_OPTIONS) {}

    generate(markdown: string): TState[] {
        return this._convertMarkdownToState(markdown);
    }

    private _convertMarkdownToState(markdown: string): TState[] {
        const {
            footnote = false,
            texMathDollars = true,
            texMathGfm = false,
            texMathSingleBackslash = false,
            texMathDoubleBackslash = false,
            trimUnnecessaryCodeBlockEmptyLines = false,
            frontMatter = true,
            multilineBlockquote = false,
            githubAlerts = false,
            fancyLists = false,
        } = this._options;

        // Synthetic block-end markers make this stream wider than the lexer output.
        const tokens: TBlockToken[] = lexBlock(markdown, {
            footnote,
            texMathDollars,
            frontMatter,
            texMathGfm,
            texMathSingleBackslash,
            texMathDoubleBackslash,
            multilineBlockquote,
            githubAlerts,
            fancyLists,
        });

        const states: TState[] = [];
        let token: TBlockToken | undefined;
        const parentList: TState[][] = [states];

        // eslint-disable-next-line no-cond-assign
        while ((token = tokens.shift())) {
            if (CONTAINER_TOKEN_TYPES.has(token.type))
                this._handleContainerToken(token, parentList, tokens);
            else
                this._handleLeafToken(token, parentList, tokens, trimUnnecessaryCodeBlockEmptyLines);
        }

        return states.length ? states : [{ name: 'paragraph', text: '' }];
    }

    private _handleContainerToken(
        token: TBlockToken,
        parentList: TState[][],
        tokens: TBlockToken[],
    ) {
        let state: TState;
        switch (token.type) {
            case 'block-end': {
                // Fix #1735 the blockquote maybe empty. like bellow:
                // >
                // bar
                if (
                    parentList[0].length === 0
                    && (
                        token.tokenType === 'blockquote'
                        || token.tokenType === 'multilineBlockquote'
                        || token.tokenType === 'list-item'
                    )
                ) {
                    state = {
                        name: 'paragraph' as const,
                        text: '',
                    };
                    parentList[0].push(state);
                }
                parentList.shift();
                break;
            }

            case 'githubAlert': {
                state = { name: 'github-alert', meta: token.meta, children: [] };
                parentList[0].push(state);
                parentList.unshift(state.children);
                tokens.unshift({ type: 'block-end', tokenType: 'githubAlert' });
                tokens.unshift(...token.tokens as TBlockToken[]);
                break;
            }

            case 'blockquote': {
                state = {
                    name: 'block-quote' as const,
                    children: [],
                };
                parentList[0].push(state);
                parentList.unshift(state.children);
                tokens.unshift({ type: 'block-end', tokenType: 'blockquote' });
                tokens.unshift(...(token.tokens as TBlockToken[]));
                break;
            }

            case 'multilineBlockquote': {
                state = {
                    name: 'block-quote' as const,
                    meta: { style: 'fenced' as const },
                    children: [],
                };
                parentList[0].push(state);
                parentList.unshift(state.children);
                tokens.unshift({ type: 'block-end', tokenType: 'multilineBlockquote' });
                tokens.unshift(...(token.tokens as TBlockToken[]));
                break;
            }

            case 'list': {
                state = this._buildListState(token);
                parentList[0].push(state);
                parentList.unshift(state.children);
                tokens.unshift({ type: 'block-end', tokenType: 'list' });
                tokens.unshift(...(token.items as TBlockToken[]));
                break;
            }

            case 'list_item': {
                const { listItemType, checked, orderMarker } = token;
                let itemState: IListItemState | ITaskListItemState;
                if (listItemType === 'task') {
                    itemState = {
                        name: 'task-list-item',
                        meta: {
                            checked: Boolean(checked),
                            ...(orderMarker ? { orderMarker } : {}),
                        },
                        children: [],
                    };
                }
                else {
                    itemState = {
                        name: 'list-item',
                        ...(orderMarker ? { meta: { orderMarker } } : {}),
                        children: [],
                    };
                }

                state = itemState;
                parentList[0].push(state);
                parentList.unshift(state.children);

/* ============================================================================
 * PARTE B — src/state/stateToMarkdown.ts:67 + 108–169
 * ========================================================================== */

export default class ExportMarkdown {
    generate(states: TState[]) {
        this._blockBoundary.reset();
        return this._convertStatesToMarkdown(states);
    }

    private _convertStatesToMarkdown(
        states: TState[],
        indent = '',
        listIndent = '',
        isTightItem = false,
    ): string {
        const result: string[] = [];
        // helper for CommonMark 264
        let previousList: IWrittenList | undefined;
        let previousState: TState | undefined;

        for (const originalState of states) {
            let state = originalState;
            if (
                state.name !== 'order-list'
                && state.name !== 'bullet-list'
                && state.name !== 'task-list'
            ) {
                previousList = undefined;
            }

            if (isAnyListState(state)) {
                const markerOverride = !this._isLooseParentList
                    && previousState?.name === 'paragraph'
                    && previousState.text.trim() !== ''
                    && this._startsWithEmptyDashBulletItem(state)
                    ? SETEXT_SAFE_BULLET_MARKER
                    : undefined;
                const followsTightList = previousState !== undefined
                    && isAnyListState(previousState)
                    && !previousState.meta.loose
                    && !state.meta.loose;
                previousList = this._serializeListBlock(
                    state,
                    result,
                    indent,
                    listIndent,
                    previousList,
                    markerOverride,
                    followsTightList,
                );
            }
            else if (state.name === 'list-item' || state.name === 'task-list-item') {
                this._serializeListItemBlock(state, result, indent, listIndent, previousState === undefined);
            }
            else {
                const prepared = this._prepareBlockBoundary(state, previousState, isTightItem);
                state = prepared.state;
                this._serializeSimpleBlock(state, result, indent, prepared.separation);
            }

            previousState = state;
        }

        return result.join('');
    }

    /* ---- fim dos trechos ---- */
