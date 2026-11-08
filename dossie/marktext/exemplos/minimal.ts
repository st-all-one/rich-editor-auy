/*
 * Exemplo mínimo de uso da engine Muya (@muyajs/core) — baseado no "Quick start" do
 * README oficial (ref/marktext/packages/muya/README.md:31-95) e na lista canônica de
 * plugins de ref/marktext/packages/muya/examples/src/main.ts:60-81.
 *
 * Licença do projeto de origem: MIT (ref/marktext/LICENSE).
 * Este arquivo é de nossa autoria, escrito como exemplo; não é código copiado.
 *
 * NÃO EXECUTADO neste ambiente — ver exemplos/README.md.
 */

import type { IMuyaOptions, TState } from '@muyajs/core';
import {
    CodeBlockLanguageSelector,
    en,
    EmojiSelector,
    FootnoteTool,
    ImageEditTool,
    ImageResizeBar,
    ImageToolBar,
    InlineFormatToolbar,
    LinkTools,
    Muya,
    ParagraphFrontButton,
    ParagraphFrontMenu,
    ParagraphQuickInsertMenu,
    PreviewToolBar,
    TableColumnToolbar,
    TableDragBar,
    TableRowColumMenu,
} from '@muyajs/core';

// Necessário quando se consome o pacote publicado (o build extrai o CSS para lib/style.css).
// Consumindo o workspace via `exports: ./src/*`, o próprio src/muya.ts:43-48 importa as folhas.
import '@muyajs/core/lib/style.css';

// ---------------------------------------------------------------------------
// 1. Plugins de UI: registro global na classe, uma vez, antes de qualquer instância.
//    `Muya.use` deduplica por `static pluginName` (src/muya.ts:142-152).
// ---------------------------------------------------------------------------
Muya.use(EmojiSelector);
Muya.use(FootnoteTool);
Muya.use(InlineFormatToolbar);
Muya.use(ImageToolBar);
Muya.use(ImageResizeBar);
Muya.use(CodeBlockLanguageSelector);
Muya.use(ParagraphFrontButton);
Muya.use(ParagraphFrontMenu);
Muya.use(ParagraphQuickInsertMenu);
Muya.use(TableColumnToolbar);
Muya.use(TableDragBar);
Muya.use(TableRowColumMenu);
Muya.use(PreviewToolBar);

// ImageEditTool é o único plugin que exige callbacks do host: onde buscar e onde
// persistir a imagem. Sem eles, inserir imagem fica sem destino.
Muya.use(ImageEditTool, {
    imagePathPicker: async () => 'assets/foto.png',
    imageAction: async (state: { src: string }) => state.src,
});

// LinkTools só navega se o host decidir como. A engine não abre nada sozinha.
Muya.use(LinkTools, {
    jumpClick: (linkInfo: { href?: string | null } | null) => {
        const href = linkInfo?.href;
        if (href && /^https?:\/\//.test(href))
            window.open(href, '_blank', 'noopener,noreferrer');
    },
});

// ---------------------------------------------------------------------------
// 2. Opções. `IMuyaOptions` está em src/types.ts:9-126; os defaults em
//    MUYA_DEFAULT_OPTIONS (src/config/index.ts). Aqui só o subconjunto que importa.
// ---------------------------------------------------------------------------
const options: Partial<IMuyaOptions> = {
    markdown: [
        '# Exemplo mínimo (Muya)',
        '',
        'Texto com **negrito**, *itálico*, `código` e um [link](https://example.com).',
        '',
        '- item um',
        '- item dois',
        '',
        '> citação',
        '',
        '```js',
        'console.log("olá")',
        '```',
        '',
    ].join('\n'),
    // Estruturais: mudá-las depois exige reparse (PARSE_AFFECTING_OPTIONS, src/muya.ts:126-135).
    footnote: false,
    frontMatter: true,
    githubAlerts: true,
    // Aparência: viram variáveis CSS --mu-* no nó raiz (src/muya.ts:1602-1616).
    fontSize: 16,
    lineHeight: 1.6,
    codeBlockLineNumbers: false,
    // Comportamento de edição.
    autoPairBracket: true,
    autoPairMarkdownSyntax: true,
    autoPairQuote: true,
    preferLooseListItem: true,
    tabSize: 4,
    listIndentation: 1,
};

// ---------------------------------------------------------------------------
// 3. Criação. `new Muya(el, opts)` SUBSTITUI o elemento hospedeiro por um div
//    contenteditable, copiando os atributos (src/muya.ts:1618-1646). Nada é
//    renderizado até init().
// ---------------------------------------------------------------------------
const host = document.querySelector<HTMLElement>('#editor')!;
const muya = new Muya(host, options);
muya.locale(en); // troca opcional de idioma (src/muya.ts:260-263)
muya.init({ focus: true });

// `host` agora está fora do DOM: use `muya.domNode` para qualquer manipulação.
const out = document.querySelector<HTMLPreElement>('#out')!;

// ---------------------------------------------------------------------------
// 4. Leitura. `getMarkdown()` serializa o estado atual; `getState()` devolve um
//    CLONE PROFUNDO do TState[] (src/state/index.ts:129-147).
//    Sempre chame flush() antes de persistir: edições pendentes vivem numa
//    transação aberta até o próximo frame (src/editor/transactions/index.ts:167-190).
// ---------------------------------------------------------------------------
document.querySelector('#save')!.addEventListener('click', () => {
    muya.flush();
    out.textContent = muya.getMarkdown();
});

document.querySelector('#state')!.addEventListener('click', () => {
    muya.flush();
    const state: TState[] = muya.getState();
    out.textContent = JSON.stringify(state.slice(0, 3), null, 2);
});

document.querySelector('#undo')!.addEventListener('click', () => muya.undo());
document.querySelector('#redo')!.addEventListener('click', () => muya.redo());

// `json-change` traz { op, source, prevDoc, doc }; o app desktop usa esse evento
// para derivar Markdown e marcar o documento como sujo (editor.vue:1907-1926).
muya.on('json-change', () => {
    console.log('[muya] mudou', muya.getSelectionState());
});

// ---------------------------------------------------------------------------
// 5. Escrita programática.
//    - setContent()  → substitui o documento e LIMPA o histórico (src/muya.ts:365-378).
//    - replaceContent() → substitui como UM passo desfazível (src/muya.ts:380-395).
//    - transact(fn)  → agrupa comandos síncronos em um passo de undo (src/muya.ts:295-297).
//    `transact` é síncrono: trabalho assíncrono precisa terminar ANTES do escopo.
// ---------------------------------------------------------------------------
function replaceDocument(markdown: string) {
    muya.flush(); // fecha a transação pendente antes de trocar o documento
    muya.replaceContent(markdown);
}

replaceDocument(`${muya.getMarkdown()}\n> escrito por código, um único undo\n`);

// ---------------------------------------------------------------------------
// 6. Seleção: snapshot serializável, restaurável (src/selection/types.ts:56-86).
// ---------------------------------------------------------------------------
const snapshot = muya.getSelectionState();
console.log('seleção serializável:', snapshot);
muya.setSelection(snapshot); // devolve boolean; false = snapshot rejeitado

// Exportação estática, sem o editor vivo:
//   import { renderToStaticHTML } from '@muyajs/core';
//   renderToStaticHTML('# oi', { sanitize: true });

// Desmontagem: remove listeners e nós. Obrigatório em SPA.
window.addEventListener('beforeunload', () => muya.destroy());
