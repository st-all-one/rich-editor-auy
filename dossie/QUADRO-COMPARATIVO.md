# Quadro comparativo

> Comparação dos doze projetos de `ref/` para o `rich-editor-auy`. As notas numéricas são as
> notas finais após a rodada de revisão independente; as afirmações têm evidência nos
> `README.md`/`arquitetura.md`/`avaliacao.md`/`revisao.md` de cada pasta.

## 1. Identidade e arquitetura

| Projeto | O que é | Fonte da verdade | Modelo | Render/UI | Estilo próprio | Licença |
|---|---|---|---|---|---|---|
| **Lexical** | framework de editor (Meta) | `EditorState` + `NodeMap` | árvore de nós tipados | DOM/`contenteditable`; React é opcional | sem folha no core, mas **força 3 estilos inline** na raiz; classes são do tema | MIT |
| **ProseMirror** | toolkit de editor estruturado | `Node` tree + `Schema` | árvore + marks + `Step` | `EditorView`/`contenteditable` | **exige** `prosemirror.css`; sem tema | MIT |
| **Quill** | editor com modelo Delta | `Delta` | sequência de ops + blots | DOM/`contenteditable` | `quill.core.css` estrutural + temas `snow`/`bubble` | BSD-3 |
| **Summernote** | WYSIWYG jQuery/Bootstrap | DOM/HTML vivo | nenhum modelo semântico | DOM/`contenteditable` | Bootstrap/less e classes próprias | MIT |
| **Tiptap** | framework sobre ProseMirror | documento ProseMirror | árvore + marks | DOM/`contenteditable`; React/Vue opcionais | injeta `style.ts` (1329 chars) por default; `injectCSS:false` desliga; extensões podem emitir style inline | MIT |
| **taino-edit** | editor Rust/Leptos/Dioxus | árvore tipada | árvore + steps + schema | core headless + ponte DOM + adapters | **zero CSS autoral** | MIT/Apache-2.0 |
| **html-auy** | HTML tipado em Rust | vocabulário/atributos | não é editor | emite HTML | zero CSS | MIT/Apache-2.0 |
| **knudge** | memória/processo para agentes | nota + índice | não é editor | CLI | zero CSS | MIT/Apache-2.0 |
| **MarkText** | app Markdown WYSIWYG | Markdown + block tree (Muya) | blocos + inline | DOM/`contenteditable`; Electron | temas/CSS de produto | MIT |
| **Zettlr** | app Markdown/Zettelkasten | Markdown + CodeMirror | documento/arquivo | CodeMirror 6; Electron | temas/CSS de app | GPL-3.0 |
| **akribie / akribiechart** | compilador de diagramas (não é editor) | `.akrc` canônico → `Chart` resolvido | quatro camadas: `Document`→`Chart`→`Placed`→`Scene` | SVG/PNG/JPG/AVIF/scene/Excalidraw; CLI/MCP | sem CSS de host; paleta/fonte próprias no SVG | MIT/Apache-2.0 |
| **calc-auy** | calculadora exata/auditável (não é editor) | AST em arena + `Canonical` | `Literal`/`Operation`/`Group`/`Control`; `Draft`→`Committed`→`Verified` | CLI, WASM, HTML/KaTeX, SVG/PNG | HTML injeta CSS KaTeX por padrão; `CssMode::External` desliga | MPL-2.0 |

## 2. O que o usuário realmente experiencia

| Projeto | Init mínimo | Primeira edição | Persistência | Extensão | A11y | Principal atrito |
|---|---|---|---|---|---|---|
| **Lexical** | `createEditor` + root + `registerRichText` | exige montar comandos/plugins para Enter, negrito etc. | `EditorState.toJSON`/`editor.getEditorState` | nodes, commands, extensions; React “oficial” | boa base ARIA no playground, mas depende do app | comportamento de edição não vem no core; API em `0.x` |
| **ProseMirror** | schema + state + view + CSS + keymap | você implementa quase tudo | `toJSON`/`fromJSON` | plugins/steps; ecossistema separado | **delegada**: sem `role`/`aria-*` no core | custo de montagem alto; CSS obrigatório |
| **Quill** | `new Quill(el, {theme:'snow'})` | funciona rápido, formatação pronta | `getContents()`/Delta | blots, formats, modules | parcial; toolbar e conteúdo | Delta não é árvore semântica; CSS/tema e DOM |
| **Summernote** | `$('#x').summernote()` | funciona em minutos | `code()`/`innerHTML` | plugins por `context` | limitada; toolbar fora do Tab, sem `aria-pressed` | HTML é a verdade; saída varia por browser; XSS |
| **Tiptap** | `new Editor({extensions:[StarterKit]})` | comandos prontos, `chain`/`can` | `getJSON`/`getHTML` | extensões de primeira classe | média; `role=textbox` no root, ARIA depende da UI | “headless” não é “sem CSS”: injeta estilo estrutural |
| **taino-edit** | schema + `EditorState` + `<TainoEditor>` | core funciona; adapter precisa de keymap | JSON/HTML/Markdown | extensões Rust; plugins observadores | **mínima**: `contenteditable` + `tabindex`, sem ARIA | keymap/copy/cut/a11y; performance cresce com doc |
| **html-auy** | `html!` + tipos | não edita; emite HTML | bytes canônicos | tabela de elementos/atributos | **forte por construção** (alt, labels, aria) | não é editor; vocabulário fixo |
| **knudge** | `kd init` | não edita texto rico | notas Markdown + índice | CLI/verbos | n/a | não é editor; é processo |
| **MarkText** | `new Muya(container)` + `init()` | WYSIWYG Markdown rápido | `getMarkdown()` | engine Muya + plugins | boa base ARIA no app; engine ainda depende do host | app Electron; Markdown como verdade; temas |
| **Zettlr** | app instalado | Markdown semi-WYSIWYG + links/citações | arquivos Markdown | app/plugins, sem API de lib | média; app | GPL; não é biblioteca; links frágeis por desenho |
| **akribie** | `akr check` + `akr render` | não edita; compila `.akrc` | `.akrc` canônico; SVG/scene/Excalidraw | sem plugin: 5 statements, 9 formas, 6 layouts fechados | SVG com `role/img`, title/desc, aria-label; sem teclado | `edge a->b` é erro (exige `a -> b`); `--out-dir` precisa existir; batch sai 8 só quando todos os itens falham |
| **calc-auy** | `CalcAuy::new` + `parse` + `commit` | não edita; calcula expressão | JSON canônico + BLAKE3; CLI/WASM | traits (`CustodyVerifier`, `OutputProcessor`); vocabulário fechado | HTML com `aria-label` verbal; SVG sem `title/desc` | exige `CustodyVerifier` até sem custódia; `publish=false` |

## 3. Referências entre publicações

| Projeto | O que existe | O que falta para “linkar publicações” |
|---|---|---|
| **Lexical** | `LinkNode`, decorators, commands; IDs podem ser atributos de nó | não há corpus, backlinks, resolução ou versionamento; é camada de app |
| **ProseMirror** | marks/attrs, `Node` com attrs; positions mapeáveis | nenhum conceito de documento externo; você define o schema/ID e resolve fora |
| **Quill** | formato `link` no Delta | link é atributo de texto; sem identidade de nó, backlinks ou corpus |
| **Summernote** | `<a href>` no HTML | nenhum modelo; resolver/reescrever é manipulação de DOM/HTML |
| **Tiptap** | extensão Link, atributos, commands | não resolve corpus; você adiciona IDs/refs via extensão |
| **taino-edit** | marks/attrs, posições mapeáveis | não tem referências entre documentos; core não resolve |
| **html-auy** | `Href` tipado/validado | só emite URL; não conhece corpus |
| **knudge** | âncoras de arquivo/nota no índice | não é editor; não há seleção/ref no texto |
| **MarkText** | links Markdown; engine mantém Markdown | sem backlinks/IDs estáveis; app abre arquivo |
| **Zettlr** | `LinkProvider`: `file→links[]`, `file→id`; `[[...]]`; inbound/outbound | **modelo mais próximo**, mas: reindexação total, 3 formas de casar, link quebrado descartado, sem versão de alvo, sem ID obrigatório |
| **akribie** | `doc="path#anchor"` no `chart`; checagem opcional com `--docs` | é **proveniência da fonte**, não referência entre documentos; sem `NodeId`, backlinks ou corpus |
| **calc-auy** | nenhum conceito de documento/referência | n/a; `RefTarget` não existe; o análogo é assinatura/custódia de cálculo |

**Lição central:** referência entre publicações não é primitiva de editor; é **identidade estável +
índice de corpus + política de resolução**. Zettlr é o único que chega perto, e ainda assim é
frágil. O `rich-editor-auy` deve separar: o core preserva `RefTarget`; o `refs` resolve; o host
persiste.

## 4. Onde cada um ganha e perde

| Projeto | Ganha | Perde | Para nós |
|---|---|---|---|
| **Lexical** | modelo e comandos; ecossistema React; ARIA | API instável; comportamento no core; inline styles | boa referência de modelo/comandos, não de neutralidade |
| **ProseMirror** | arquitetura canônica; steps/mapeamento; schema | custo de montagem; CSS obrigatório; a11y delegada | **melhor fonte para o modelo**; não copiar o custo de UI |
| **Quill** | init rápido; Delta elegante; histórico | Delta não é árvore; CSS/tema; limites semânticos | boa referência de op-based editing, não de modelo de bloco |
| **Summernote** | integração trivial; HTML interoperável | sem modelo; XSS; saída não determinística; a11y fraca | exemplo do que **não** fazer como dado |
| **Tiptap** | DX de extensões/comandos; headless opt-in | injeta CSS; dependência ProseMirror; pro features | melhor referência de **API de extensão**, com ressalva de estilo |
| **taino-edit** | Rust, core headless, steps, JSON/HTML/MD | keymap/copy/a11y; performance; adapter | **mais próximo do nosso core**; copiar conceitos, não o Leptos |
| **html-auy** | tipagem/escape/a11y por construção | não é editor | referência para **renderer/serializer** |
| **knudge** | rigor, gates, decisões, testes | não é editor | referência de **processo** |
| **MarkText** | engine Muya reutilizável; Markdown WYSIWYG | app Electron; temas; sem ID/backlinks | referência de engine Markdown, não de core Rust |
| **Zettlr** | UX de links/citações; fluxo de publicação | GPL; app; links frágeis | referência de **UX de referências**, não de código |
| **akribie** | pipeline puro; diff semântico; schema/a11y derivados; determinismo medido | sem editor, sem extensão, sem referências; 1 commit | referência de **contrato de artefato/diff/schema** |
| **calc-auy** | hash `Trace`×`Value`; typestate (`Draft`→`Committed`→`Verified`); limites na leitura; erros estáveis | MPL; calculadora; sem documento/UI; `publish=false` | referência de **hash, validação e erro**, não de edição |

## 5. O que o `rich-editor-auy` deve fazer com isso

1. **Não prometer “zero CSS” sem definir o mínimo estrutural.** Até ProseMirror/Tiptap precisam de
   `white-space`, seleção e cursor. A promessa defensável é: **nenhum CSS autoral, classe ou style
   inline injetado**; o host fornece o mínimo documentado, ou o adaptador prova que funciona sem.
2. **Copiar o modelo, não a UI.** `Node`/`Fragment`/`Step`/`Mapping`/`Schema` do ProseMirror e a
   separação core↔DOM do taino são as melhores referências.
3. **IDs e referências são dados.** `NodeId`/`DocId` opacos, `RefTarget` tipado; o core preserva e
   mapeia; o corpus resolve. Não colocar banco/rede no core.
4. **A11y é invariante de core, não camada nem slogan.** O core possui `A11yTree`, roles, nomes,
   `alt`, `lang`/`dir`, ordem de leitura, ranges e `validate_a11y`; o adaptador mapeia para
   nativo/ARIA, foco, teclado e anúncios; o host garante contraste/foco visível/auditoria WCAG.
   A11y não entra “depois” nem é opcional: é critério de aceite em F1–F4. Nenhum dos editores JS
   entrega WCAG “de graça”, e vários delegam a11y inteiramente ao app — é exatamente o que não
   faremos.
5. **Performance é contrato.** O taino mostra que o custo por tecla cresce com o documento; nosso
   orçamento de 100–200 ns e snapshots de 10 µs precisa de benchmark desde o F1.
6. **Rigor de processo é vantagem.** knudge/html-auy mostram que gates, decisões e testes de
   contrato são tão importantes quanto o modelo.
7. **Separe hash de conteúdo de hash de traço.** O calc-auy mostra `value_hash` (sem metadados,
   para cache/dedup) e `trace_hash`/assinatura (com metadados, para integridade). Hash não é
   identidade: `NodeId` continua sendo opaco (D53).
8. **Diff e schema são operações de primeira classe.** O akribie compara o modelo resolvido, não
   geometria, e deriva `schema`/`prime` da mesma tabela; o calc-auy mostra o custo de introspecção
   escrita à mão quando ela diverge. Ver [`IDEIAS-akribie-calc.md`](IDEIAS-akribie-calc.md).

## 6. Notas (0–5)

Notas finais da revisão; ver `avaliacao.md` e `revisao.md` de cada projeto para a evidência.

| Projeto | Usab. | API | Neutr. estilo | Modelo | Ext. | Perf. | A11y | Docs | Manut. | Fit |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Lexical | 2 | 4 | 4 | 5 | 4 | 4 | 3 | 4 | 3 | 3 |
| ProseMirror | 2 | 4 | 2 | 5 | 4 | 4 | 1 | 4 | 5 | 3 |
| Quill | 4 | 4 | 2 | 4 | 4 | 4 | 3 | 4 | 4 | 3 |
| Summernote | 4 | 3 | 1 | 1 | 3 | 3 | 2 | 3 | 3 | 1 |
| Tiptap | 4 | 4 | 4 | 5 | 5 | 3 | 3 | 3 | 3 | 3 |
| taino-edit | 3 | 4 | 3 | 5 | 4 | 2 | 1 | 5 | 4 | 3 |
| html-auy | — | — | — | — | — | — | — | 5 | 4 | 4 |
| knudge | — | — | — | — | — | — | — | 4 | 4 | 3 |
| MarkText | 3 | 3 | 1 | 4 | 2 | 3 | 2 | 4 | 4 | 3 |
| Zettlr | 4 | 1 | 2 | 4 | 2 | 3 | 3 | 3 | 4 | 3 |
| akribie | 4 | 4 | — | 5 | 1 | 4 | 4 | 4 | 4 | 2 |
| calc-auy | — | 4 | — | 5 | 3 | 3 | — | 4 | 4 | 4 |

`Fit` é adequação ao `rich-editor-auy`, não qualidade geral. `html-auy`/`knudge`/`calc-auy` não são
editores; suas notas de modelo/editor não se aplicam e servem só como referência de
render/processo/arquitetura (`html-auy`: 0 como editor, 4–5 como renderer; `knudge`: n/a como
editor, alto valor de processo; `calc-auy`: 0 como editor, 4 como referência de hash/erro).
`akribie` é um compilador de diagramas: `Neutralidade` é `—` (não há host), mas API/modelo/diff/
schema são avaliados como referência; `A11y` 4 é a do SVG, não de contenteditable.

## 7. Confiança da revisão

Cada dossiê passou por um revisor independente. A confiança mede **evidência verificada**, não
entusiasmo; as lacunas estão em `revisao.md`.

| Projeto | Confiança | Cobertura de usabilidade | Principal limite |
|---|---:|---|---|
| Lexical | 87% | 15/15 com evidência; 8 com execução | sem navegador real (seleção/IME/clipboard/Yjs) |
| ProseMirror | 88% | 18/18; 10 com execução | view/CSS/IME não executados; benchmark não medido |
| Quill | 87% | 13/13; execução em Chromium | só Chromium; sem leitor de tela; sem estresse >1k parágrafos |
| Summernote | 86% | 13/13; 88,5% | um browser; colagem/IME reais e leitor de tela não testados |
| Tiptap | 88% | 13/13 | monorepo não construído; IME/colagem/Yjs/leitor de tela |
| taino-edit | 85% | 13/13; 8 execuções | 96 testes de browser não rodados; adapter/a11y 78% |
| html-auy | 88% | 12/13 + 1 parcial | performance, MSRV 1.92, fuzz/Miri e audit real |
| knudge | 88% | 10/10 aplicáveis; 5 N/A de editor | suíte completa/bench/fuzz não executados |
| MarkText | 82% | 13/13 por leitura; execução não rodada | sem `node_modules`; Playwright e `test:spec` não executados |
| Zettlr | 88% | ~92% | app Electron/Pandoc/IME/leitor de tela não executados |
| akribie | 90% | 13/13; 8 com execução | a11y de leitor de tela, `make check`/release/install, 1 commit |
| calc-auy | 87% | 12/13 + 1 parcial (performance) | benchmark próprio, MSRV 1.98, wasm32, fuzz/Miri |

A meta de 85% de **confiança/evidência** foi atingida em onze dos doze dossiês. Isso não significa
85% de execução: a cobertura executada varia (Lexical 8/15, ProseMirror 10/18, Quill 13/13,
Tiptap 13/13 por leitura, MarkText 13/13 sem execução). O MarkText ficou em **82% de confiança**
porque a execução não foi possível (`node_modules` ausente e instalação travou), mas cobre
**13/13 itens por leitura verificada**; é a única exceção declarada, não maquiada.
