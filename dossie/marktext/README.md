# MarkText / Muya — dossiê de referência

> Estudo de caso verificável para o design do `rich-editor-auy`.
> Fonte: `ref/marktext/`, commit `a520bef` (2026-11-06), `marktext-monorepo` 0.24.0-dev e
> `@muyajs/core` 0.2.0 (`ref/marktext/packages/muya/package.json:3`). Licença **MIT**
> (`ref/marktext/LICENSE`). Nada aqui foi executado: não há `node_modules/` no clone
> (`ls -d node_modules` → exit 2). Ver `exemplos/README.md` para o que ficou não verificado.

---

## 1. O que é, em uma frase

MarkText é um **aplicativo Electron de Markdown WYSIWYG** (`packages/desktop`) construído sobre
uma **engine de edição própria e reutilizável, Muya** (`packages/muya`, publicada como
`@muyajs/core`), que mantém o documento como uma árvore de blocos em JSON e serializa para
Markdown sob demanda.

A distinção é a coisa mais importante deste dossiê:

| Camada | Caminho | O que é | Reutilizável fora do app? |
|---|---|---|---|
| Engine de edição | `packages/muya/` | `@muyajs/core`: parser, modelo de blocos, renderer inline, seleção, histórico, UI flutuante | **Sim** — é biblioteca de navegador; tem demo Vite próprio (`packages/muya/examples`) e suíte Playwright (`packages/muya/e2e`) |
| App desktop | `packages/desktop/` | Electron 45 + Vue 3 + Pinia: janelas, IPC, sistema de arquivos, abas, preferências, temas, exportação | Não — é o produto |
| Site/documentação | `packages/website/` | Next.js 15; contém os docs de usuário e de desenvolvedor | Não |

Evidência da extração: o README da engine diz "a standalone Markdown editor for the web, extracted
from MarkText" (`ref/marktext/packages/muya/README.md:3`) e o FAQ afirma que "the Electron-specific
assumptions have been progressively peeled away"
(`ref/marktext/packages/muya/README.md:277`). O pacote tem `exports`, `main`, `module` e
`types` próprios (`ref/marktext/packages/muya/package.json:7-16`), mas **não é publicado a partir
deste repositório**: o desktop o consome por `workspace:*`
(`ref/marktext/packages/desktop/package.json` dependências; e
`ref/marktext/packages/muya/README.md:271`).

### Veredito

- **Engine reutilizável** = `packages/muya` (`@muyajs/core`): modelo `TState[]`, parser/serializer
  Markdown, transações `ot-json1`, renderer inline, seleção e plugins de UI. É biblioteca de
  navegador, agnóstica de Electron, e é o que interessa estudar.
- **App Electron** = `packages/desktop`: janelas, IPC, sistema de arquivos, abas, preferências,
  temas e exportação. Consome a engine; não é reutilizável.
- **A fonte de verdade em disco é Markdown, não o estado JSON.** A árvore `TState[]` vive só em
  memória; o desktop escuta `json-change`, chama `getMarkdown()` e grava o texto
  (`editor.vue:1907-1926`; `arquitetura.md` §8.1). Nenhum arquivo de estado JSON é persistido.
  Consequência: o formato do arquivo é o contrato com o usuário, mas qualquer mudança no
  serializador altera arquivos já existentes.

## 2. Fluxo real do usuário (app)

O app é o que o usuário instala; o guia de instalação lista AppImage/deb/rpm/snap/tar.gz para
Linux, DMG para macOS 13+ e instalador NSIS/zip para Windows
(`ref/marktext/packages/website/content/docs/end-user/INSTALLATION.md:3-40`).

Fluxo de edição, com evidência:

1. **Abrir** — `CmdOrCtrl+O` ou menu; o app também registra `.md`, `.markdown`, `.mmd`, `.mdown`,
   `.mdtxt` e `.mdtext` (`INSTALLATION.md:14`). O texto é lido no processo principal, com detecção
   de encoding e line ending (`ref/marktext/packages/desktop/src/main/filesystem/markdown.ts:164-200`).
2. **Editar em WYSIWYG** — o renderer monta a engine dentro de um componente Vue, registrando
   17 plugins de UI (`ref/marktext/packages/desktop/src/renderer/src/components/editorWithTabs/editor.vue:1693-1722`)
   e passando 52 opções (`editor.vue:1724-1779`). A engine substitui o elemento hospedeiro por
   um `div` `contenteditable` (`ref/marktext/packages/muya/src/muya.ts:1618-1646`) e nada é
   renderizado antes de `init()` (`ref/marktext/packages/muya/README.md:93-94`,
   `ref/marktext/packages/muya/src/muya.ts:246-253`).
3. **Digitar** — cada bloco editável é um `span.mu-content[contenteditable]`
   (`ref/marktext/packages/muya/src/block/base/content.ts:423-433`). Ao digitar, o bloco inteiro
   é re-tokenizado e seu `innerHTML` é substituído
   (`ref/marktext/packages/muya/src/inline/index.ts:74-84`).
4. **Salvar** — a cada `json-change`, o desktop chama `muya.getMarkdown()` e alimenta o store
   (`editor.vue:1907-1926`). A gravação em disco é feita **no processo principal**, de forma
   atômica (`ref/marktext/packages/desktop/src/main/filesystem/index.ts:126`), com conversão de
   line ending e verificação de perda por encoding
   (`ref/marktext/packages/desktop/src/main/filesystem/markdown.ts:134-163`).
5. **Modos** — WYSIWYG, código-fonte (CodeMirror 5) e leitura; o README de usuário descreve a
   troca e o outline (`ref/marktext/packages/website/content/docs/end-user/BASICS.md:32-42`).

O que o usuário **sente**: WYSIWYG com sintaxe escondida por padrão, menus `/`, barra flutuante
de formatação, tabelas arrastáveis, matemática, diagramas, find/replace com regex, 13 idiomas
(`ref/marktext/packages/muya/README.md:7-25`).

## 3. Fluxo do desenvolvedor que usa a engine

O `README` da engine documenta um init de 4 passos que é genuinamente curto
(`ref/marktext/packages/muya/README.md:31-95`; bloco de código em `:33-95`):

```ts
Muya.use(InlineFormatToolbar);                 // plugins são globais na classe
const muya = new Muya(container, { markdown: '# Hello, Muya' });
muya.init();                                   // nada renderiza antes disso
```

O exemplo canônico e completo — qual conjunto de plugins registrar — está em
`ref/marktext/packages/muya/examples/src/main.ts:60-81`. API resumida (todas verificadas em
`src/muya.ts`):

| Método | Linha | Comportamento |
|---|---|---|
| `init({ focus })` | `:246` | Registra blocos, cria a raiz `ScrollPage`, liga eventos |
| `getState()` | `:280` | **Clone profundo** do `TState[]` (`src/state/index.ts:135-137`) |
| `getMarkdown()` | `:284` | Serializa o estado atual para Markdown |
| `flush()` | `:290` | Fecha a transação pendente antes de persistir/trocar de documento |
| `transact(fn)` | `:295` | Agrupa comandos síncronos em **um** passo de undo |
| `setContent(md \| TState[])` | `:365` | Substitui o documento e **limpa o histórico** |
| `replaceContent(...)` | `:380` | Substitui como um passo desfazível |
| `getSelectionState()` / `setSelection()` | `:651` / `:655` | Snapshot serializável de seleção e restauração validada |

Ponto de atrito real: **não existe** API pública para inserir um nó arbitrário. A conversão de um
bloco em outro passa por `muya.updateParagraph(type)` (`src/muya.ts:1176-1178`), que traduz o rótulo
do menu por `PARAGRAPH_LABEL_MAP` (`src/muya.ts:62-92`) e delega a `replaceBlockByLabel` /
`insertBlockBelowByLabel` de `src/block/blockTransforms.ts` (`src/muya.ts:12,1380-1387`) — ou seja,
o conjunto de blocos é fechado e indexado por string, não por schema.

## 4. O que é reutilizável para nós

Reutilizável **como ideia**, não como código (somos Rust headless, sem DOM):

1. **Modelo de blocos discriminado por `name`** com texto Markdown cru dentro da folha
   (`ref/marktext/packages/muya/src/state/types.ts:4-206`; união em `:208-233`). É a mesma escolha
   que precisamos: preservar a fonte, guardar o mínimo de metadados.
2. **Estado canônico em árvore + operações `ot-json1` + `ot-text-unicode`**
   (`src/state/index.ts:21-36`, `src/editor/transactions/index.ts:128-146`). O `preview()` de um
   documento imutável é um padrão limpo de "candidato antes do commit" (`src/state/index.ts:50-52`).
3. **Transação com escopo e fronteira de histórico explícita** (`merge`, `isolated`, `ignore`,
   `separate`) e rollback por exceção (`src/editor/transactions/index.ts:75-107,167-204`).
4. **Regras de edição pós-operação** (`EDITING_RULES`, `src/editor/transactions/index.ts:31`) —
   o gancho certo para "lista ficou loose/tight" sem espalhar lógica pelos comandos.
5. **Definições de referência indexadas por varredura do estado, com cache por identidade de
   snapshot** (`src/inline/referenceDefinitions.ts:41-71`). Muito próximo do que precisamos para
   referências entre publicações.
6. **Seleção como snapshot serializável** com unidades documentadas (UTF-16 para texto, range
   half-open para imagem, coordenadas zero-based incluindo cabeçalho para tabela)
   (`src/selection/types.ts:56-86`).

Não reutilizável: `contenteditable`, `innerHTML`, `snabbdom`, `marked`/`turndown`, CSS embutido,
e todo o acoplamento Electron (IPC, `webUtils.getPathForFile`, clipboard nativo).

## 5. Prós e contras

**Prós**

- Modelo de dados explícito e serializável, com texto-fonte preservado dentro de cada bloco.
- Round-trip Markdown testado de verdade: suíte de conformidade com números publicados —
  91,0% do CommonMark 0.31 (593/652) e 89,3% do GFM (600/672)
  (`ref/marktext/packages/muya/test/spec/conformance.md:15-20`).
- Engenharia de edição séria: 361 specs unitários em `src/**/__tests__`, 160 specs Playwright
  (`find` observado), IME com casos dedicados (`e2e/tests/typing/ime.spec.ts`), testes de
  seleção, clipboard, undo e sanitização XSS (`e2e/tests/security/sanitize.spec.ts`).
- UI separada do core por plugins com nome (`Muya.use(Plugin)`, `src/muya.ts:142-152`).

**Contras**

- **Acoplado ao DOM/contenteditable e a uma pilha JS pesada** (28 dependências de runtime (52 com devDependencies) no
  `package.json` da engine: KaTeX, Mermaid, Vega, PlantUML, Prism, DOMPurify, snabbdom, marked,
  turndown). Nada disso cabe num núcleo Rust headless.
- **Não é neutro em CSS**: o entry importa **6 folhas** de estilo (`src/muya.ts:43-48`) que somam
  **2.555 linhas**; o diretório tem **7 arquivos** (2.934 linhas) porque `exportStyle.css`
  (379 linhas) não é importado pelo entry — entra inline no HTML de export via
  `src/state/markdownToHtml.ts:6` — enquanto `prismjs/light.theme.css` (108) é importado.
  Inclui **16 `@font-face`** em `index.css:68-203`: 8 faces de Open Sans (4 pesos × normal/itálico),
  6 de DejaVu Sans Mono, Noto Music e Noto Sans Math. Tema é contrato de variáveis CSS: a engine
  define kebab-case (`--mu-*`, `--editor-color-*`, `index.css:1-66`), enquanto os **32 temas** do
  desktop ainda usam camelCase (`--editorColor`, `--themeColor` — `--editorColor` aparece 488 vezes
  nos temas).
- **Sem virtualização**: `setContent` de 10 mil parágrafos reconstrói a árvore DOM inteira
  (`src/block/scrollPage/index.ts:87-96`), com orçamento de teste de 60 s
  (`e2e/tests/stability/perf.spec.ts:35`).
- **Extensão é fork, não plugin de schema**: adicionar um bloco exige tocar em `state/types.ts`,
  `markdownToState`, `stateToMarkdown`, `block/index.ts` e o renderer inline, nesta ordem.
- **A engine derivada está em 0.2.0 e o próprio README avisa** que não é recomendada para produção
  (`packages/muya/README.md:5`).
- **Acessibilidade é básica**: `role="textbox"`/`"document"` no contêiner, `aria-label`, alguns
  `aria-expanded`/`aria-hidden`; o teste axe só barra violações de impacto `critical`
  (`e2e/tests/a11y/host-scan.spec.ts:19`).
- **Diagramas não são todos offline**: KaTeX, Mermaid e Vega são locais, mas PlantUML só encoda e
  busca um servidor remoto, por default `https://www.plantuml.com/plantuml`
  (`src/config/index.ts:339`).

## 6. Mapa do dossiê

- `arquitetura.md` — modelo de blocos, parser, renderer, estado/transações, seleção/IME,
  temas, persistência, exportação, a11y e performance, com citações.
- `avaliacao.md` — rubrica 0–5 com evidência por dimensão, matriz de usabilidade (13 eixos) e
  limites do estudo.
- `codigo/` — 6 arquivos reais (MIT) + `INDEX.md` com origem e justificativa.
- `exemplos/` — exemplo mínimo baseado no quick start oficial, com o que não foi executado.
