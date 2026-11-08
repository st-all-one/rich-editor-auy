# Avaliação — MarkText / Muya

Rubrica de `dossie/_RUBRICA.md` (0–5, âncoras fixos). Toda nota tem evidência em `ref/marktext/`.
**Nada foi executado**: o clone não tem `node_modules/` (`ls -d node_modules` → exit 2) e as duas
suítes exigem `pnpm install`. As evidências são leitura de código, leitura de testes e leitura de
documentação. Isso limita a confiança a ~82% (revisão independente).

---

## 1. Notas por dimensão

| Dimensão | Nota | Evidência principal | Comentário |
|---|---|---|---|
| **Usabilidade de integração** | **3** | Quick start de 4 passos em `packages/muya/README.md:31-95`; `Muya.use` global `src/muya.ts:142-152`; `init()` obrigatório `src/muya.ts:246-253`; exemplo canônico `examples/src/main.ts:60-81` | Init previsível: registrar plugins, `new Muya(el, opts)`, `init()`. O atrito é grande antes e depois — o pacote **não é publicado** (só consumo em workspace ou build manual do monorepo Electron), exigem-se 17 plugins e CSS próprios, e 52 opções passadas pelo app (`editor.vue:1724-1779`). É "funciona, com atrito", não "init simples". |
| **API** | **3** | `src/index.ts:1-58` (superfície pública); `getState`/`getMarkdown`/`flush`/`transact`/`setContent`/`replaceContent`/`getSelectionState`/`setSelection` em `src/muya.ts:280-395,651-657`; `IMuyaOptions` em `src/types.ts:9-126` (57 chaves) | Tipada e pequena, mas ainda 0.2.0: o README avisa que **APIs quebram entre minors** e não recomenda produção (`packages/muya/README.md:5`). Arestas concretas: inserir/converter bloco usa *labels* stringly-typed traduzidos por mapa interno (`src/muya.ts:62-92`, `updateParagraph` em `:1176-1178`); `flush()` é pré-requisito de leitura consistente; 57 chaves de opção sem schema de validação. É "utilizável" (3), não "difícil de usar errado". |
| **Neutralidade de estilo** | **1** | 6 folhas importadas no entry (`src/muya.ts:43-48`, 2.555 linhas); 7 arquivos no diretório (2.934 linhas) porque `exportStyle.css` só entra no export (`src/state/markdownToHtml.ts:6`); 16 `@font-face` em `index.css:68-203`; classes `mu-*`, variáveis `--mu-*`/`--editor-color-*` (`index.css:1-66`) e camelCase (`--editorColor`, `--themeColor`) nos 32 temas do desktop | Não é neutro: sem o CSS da engine o documento não tem tipografia, marcadores, código ou tabela apresentáveis. Há caminho de customização por variáveis CSS, mas a estrutura de classes e o layout são contrato. Não chega a 0 porque o consumidor pode sobrescrever as variáveis sem forkar o CSS. |
| **Modelo de dados** | **4** | União `TState` com 21 formas (11 folhas + 10 contêineres, `src/state/types.ts:208-233`; interfaces em `:4-206`); predicados em `:237-265`; caminho JSON como identidade (`src/block/base/content.ts:380-387`); serialização round-trip (`src/state/stateToMarkdown.ts:67-180`) | Árvore tipada, discriminada e fiel ao Markdown, com bom mapeamento para nós. Não chega a 5: não há schema validável em runtime, o texto cru convive com metadados derivados (heading guarda `level` **e** `#`), e definições de referência são "parágrafo com texto especial" (`types.ts:51-62`). |
| **Extensibilidade** | **2** | Blocos exigem 6 pontos de alteração (ver `arquitetura.md` §9.1); `loadBlock` só conhece o registro estático (`src/block/scrollPage/index.ts:33-40`); plugins são apenas de UI (`src/ui/*/index.ts`, `static pluginName`) | Extensão de UI é boa; extensão de documento é fork. Não há schema, mark/node registrável, nem transformação customizável por host. |
| **Performance** | **3** | `updateState` recria toda a árvore (`src/block/scrollPage/index.ts:87-96`); `innerHTML` por bloco a cada edição (`src/inline/index.ts:70-86`); único teste: 10k parágrafos < 60 s (`e2e/tests/stability/perf.spec.ts:17-42`, asserção na linha 35); histórico capado em 100 (`src/history/index.ts:52-53`) | Aceitável para documentos médios em app desktop. Sem virtualização, sem benchmark de digitação, sem métrica de memória. O orçamento de 60 s é frouxo demais para provar qualidade. |
| **Acessibilidade** | **2** | Raiz `role="textbox"`/`document` + `aria-label` (`src/muya.ts:218-222`); links com `role="link"`/`tabindex` no modo leitura (`src/editor/readOnly.ts:5-15`); 19 ocorrências de `aria-`/`role=` em `src/**/*.ts` fora de testes; axe só barra `critical` (`e2e/tests/a11y/host-scan.spec.ts:19`) | Base mínima presente e testada por ferramenta, mas o documento é um único campo editável. Não há árvore navegável por leitor de tela, live regions, nem anúncio de mudança de bloco. |
| **Documentação** | **4** | `packages/muya/README.md` (290 linhas: API, eventos, plugins, arquitetura, FAQ); `packages/muya/AGENTS.md` (12.843 bytes, walkthrough por arquivo); `packages/website/content/docs/{dev,end-user}` (36 arquivos); `test/spec/conformance.md`; `test/inline-spec/README.md` | Boa e atualizada, com limites declarados (o que não é coberto, por que uma suíte mede X e não Y). Não chega a 5: o README é "API at a glance", não referência completa; não há guia de migração entre minors; o `AGENTS.md` fala com o agente, não com o consumidor. |
| **Manutenção** | **4** | Commit `a520bef` de 2026-11-06; CI por pacote (`.github/workflows/`); 361 specs em `src/**/__tests__`, 160 specs Playwright, 338 arquivos em `packages/desktop/test`; `expected-failures.json` como trava de regressão; suíte de sanitização XSS; `scripts/validateLicenses.ts` | Projeto ativo e testado de forma incomum para um editor. Não dou 5 porque a governança é concentrada (sem commitlint; `packages/muya` é consumida só em workspace e não é publicada daqui — `packages/muya/README.md:271`) e a engine derivada está em 0.2.0. |
| **Adequação ao `rich-editor-auy`** | **3** | Modelo + transações: `src/state/types.ts`, `src/state/index.ts:20-156`, `src/editor/transactions/index.ts:38-231`; renderer: `src/inline/index.ts:70-86` + 6 folhas de CSS (2.555 linhas); dependências: `packages/muya/package.json` (28 de runtime, 52 com dev: KaTeX, Mermaid, Vega, PlantUML, Prism, DOMPurify, snabbdom, marked, turndown) | Referência forte **de desenho** (modelo de blocos, transação, snapshot de seleção, índice de referências) e referência nula **de implementação** (DOM, TS, CSS, 28 deps de runtime). Adaptável com custo: copiar as ideias exige reprojetar renderer e formato. |

Soma bruta: **29/50**. Média: **2,9**.

### Leitura das notas

O projeto vale por **três coisas**: o modelo de dados, o pipeline de transação e a disciplina de
testes. Vale pouco pelo que toca a nossa restrição mais dura (zero CSS, headless, Rust) e pela
extensibilidade de documento.

Três notas foram **contestadas na revisão** (ver `revisao.md` §2): usabilidade 4→3 (o pacote não é
publicado; exige monorepo, 17 plugins e CSS), API 4→3 (0.2.0 com quebra anunciada entre minors e
`flush()` obrigatório) e documentação 5→4 (bom "API at a glance", sem referência completa nem guia
de migração). As demais foram confirmadas.

---

## 2. Cobertura da matriz de usabilidade

Meta do guia: ≥85% dos eixos com evidência. Situação: **13/13 (100%)** com evidência
classificada — mas nem toda evidência é de execução. A coluna "tipo" é honesta sobre isso.

| # | Eixo | Status | Tipo | Evidência |
|---|---|---|---|---|
| 1 | Instalação e pré-requisitos | ✅ | verificado em doc | `packages/website/content/docs/end-user/INSTALLATION.md:5-41` (artefatos por SO; macOS 13+ em `:18`); build: `packages/muya/AGENTS.md:30` (Node ≥20.19, pnpm ≥10, target `chrome70`); raiz exige Node ≥22.12 (`ref/marktext/package.json:8-11`) |
| 2 | Inicialização mínima | ✅ | verificado em código + doc | `packages/muya/README.md:31-95`; `src/muya.ts:1618-1646` (substitui o container); `src/muya.ts:246-253` (`init` registra blocos e instancia plugins) |
| 3 | Edição básica (texto, parágrafo, negrito/itálico/link) | ✅ | verificado em código + teste | `src/block/base/format.ts:635-700` (`inputHandler`); `FORMAT_MARKER_MAP`/`FORMAT_TYPES` em `src/config/index.ts:34-70`; testes: `e2e/tests/typing/paragraph-and-headings.spec.ts`, `e2e/tests/inline/*` |
| 4 | Seleção, teclado, undo/redo, clipboard, IME | ✅ | verificado em código + teste | Seleção: `src/selection/types.ts:1-146`; teclado: `src/editor/index.ts:353-445`; undo: `src/history/index.ts:80-196` + `e2e/tests/editing/undo-redo.spec.ts`; clipboard: `src/clipboard/index.ts:49-260` + `e2e/tests/editing/clipboard.spec.ts`; IME: `src/block/base/content.ts:693-780` + `e2e/tests/typing/ime.spec.ts` |
| 5 | Schema e restrições de conteúdo | ✅ | verificado em código | Não há schema: união TS + predicados manuais (`src/state/types.ts:237-265`). Restrição real é o conjunto fechado de `name`s registrados (`src/block/index.ts:41-81`); `loadBlock` só avisa (`src/block/scrollPage/index.ts:33-40`) |
| 6 | Custom node/mark/plugin | ✅ | verificado em código | Plugin de UI: `src/muya.ts:142-152` + 17 classes de plugin em 22 diretórios de `src/ui/` (as 2 bases de float e `utils`/`__tests__` não são plugins). Bloco/mark: fork em 6 pontos (ver `arquitetura.md` §9.1). Sem API de registro de node/mark por consumidor |
| 7 | Persistência e serialização | ✅ | verificado em código | Estado: `src/state/index.ts:131-148`; Markdown: `src/state/stateToMarkdown.ts:67-180`; gravação atômica com encoding e line ending: `packages/desktop/src/main/filesystem/markdown.ts:134-163` e `filesystem/index.ts:126` |
| 8 | Renderização e não interferência no CSS | ✅ | verificado em código | `src/inline/index.ts:70-86` escreve `innerHTML`; 6 folhas de CSS importadas (2.555 linhas; 7 arquivos no diretório = 2.934, pois `exportStyle.css` não é importado) e classes `mu-*`; variáveis `--mu-*`/`--editor-color-*` na engine (`index.css:1-66`) e camelCase nos temas do desktop; container substituído (`src/muya.ts:1618-1646`). Interfere: classes `mu-*` e layout são da engine |
| 9 | Acessibilidade | ✅ | verificado em código + teste | `src/muya.ts:218-222`; `src/editor/readOnly.ts:5-15`; `e2e/tests/a11y/host-scan.spec.ts` (axe, só `critical`); 19 ocorrências de ARIA no total |
| 10 | Colaboração | ✅ | verificado em código + doc | `ot-json1` `invert`/`compose`/`transform` expostos (`src/state/index.ts:21-36`), mas **sem transporte**: `packages/muya/README.md:204`. Não há presença, cursores remotos nem sincronização |
| 11 | Performance e limites | ✅ | verificado em código + teste | Rebuild total (`src/block/scrollPage/index.ts:87-96`); re-render por bloco (`src/inline/index.ts:70-86`); teste 10k parágrafos < 60 s (`e2e/tests/stability/perf.spec.ts:17-42`); histórico 100 (`src/history/index.ts:52-53`) |
| 12 | Erros, migração e diagnóstico | ✅ | verificado em código + doc | Logger por módulo (`src/utils/logger.ts`); fallback rebuild em falha incremental (`src/editor/index.ts:631-637`); rollback = callback de reconstrução passado em `src/editor/index.ts:299` e acionado em `src/editor/transactions/index.ts:102,182`; `expected-failures.json` (`packages/muya/AGENTS.md:22`); sem tipos de erro para o host |
| 13 | Licença e maturidade | ✅ | verificado em doc/código | MIT (`ref/marktext/LICENSE`); `a520bef` 2026-11-06; 0.24.0-dev / `@muyajs/core` 0.2.0; 361 + 160 + 338 arquivos de teste; engine não publicada deste repo (`packages/muya/README.md:271`) |

**Cobertura: 100% dos eixos com algum tipo de evidência.**
**Execução real: 0 eixos.** Nenhum comando de build, teste ou demo foi rodado (sem dependências
instaladas). Os eixos 3, 4, 9 e 11 têm evidência de **teste escrito**, não de teste observado.

---

## 3. O que foi verificado e o que não foi

### Verificado (leitura direta, com linha)

- Estrutura de `TState` e o conjunto de 21 formas de bloco.
- Pipeline `enqueue → flush → _complete → apply → _updateBlocks → json-change`.
- Que a seleção é snapshot serializável com unidades explícitas.
- Que o renderer inline substitui `innerHTML` do bloco inteiro.
- Que o formato persistido é Markdown e a escrita usa `writeFileAtomic` com verificação de encoding.
- Que há 39 blocos registrados, 17 classes de plugin de UI (em 22 diretórios de `src/ui/`), 13
  locales (14 arquivos, um deles `index.ts`) e 32 temas no app.
- Que o export HTML usa um CSS próprio (`exportStyle.css`) inlinado por `src/state/markdownToHtml.ts:6`.
- Que PlantUML depende de servidor remoto (`src/config/index.ts:339`), ao contrário de KaTeX/Mermaid/Vega.
- Que a conformidade CommonMark/GFM é 91,0% / 89,3% pelos números versionados no repositório.
- Que `packages/muya` não é publicado a partir deste repositório.

### Não verificado (e por quê)

| Item | Motivo | Como verificar |
|---|---|---|
| `pnpm --filter @muyajs/core test` | Sem `node_modules` | `pnpm install` na raiz, depois o comando |
| Demo `examples/` | Idem | `pnpm --filter muya-examples dev:demo` |
| Suíte Playwright (160 specs) | Idem + precisa navegador | `pnpm -C packages/muya/e2e e2e:install` e `e2e:chromium` |
| Conformidade real (593/652 etc.) | Números lidos de `test/spec/conformance.md`, não recalculados | `pnpm --filter @muyajs/core test:spec` |
| IME real de sistema operacional | Automação do próprio projeto simula eventos DOM (`e2e/tests/typing/ime.spec.ts:5-27`) | Teste manual com pinyin/japonês em Chromium/Firefox/WebKit |
| Comportamento com leitor de tela | Sem ambiente | NVDA/VoiceOver + axe completo (sem filtro `critical`) |
| Performance de digitação em documento grande | Nenhum teste mede isso no repo | Profiling com Performance panel em documento de 10k blocos |
| Qualidade do round-trip em documentos reais | Só specs e fixtures do repo | Rodar a engine em corpus Markdown real e diffar antes/depois |

---

## 4. Adequação concreta ao `rich-editor-auy`

### Copiar (ideia, não código)

1. **Folha com Markdown cru + metadados mínimos** (`types.ts:4-43`). Nosso formato pode guardar o
   source exato por nó sem virar AST byte-preserving completo.
2. **Estado canônico imutável + `preview()` de candidato** (`state/index.ts:50-52`). É exatamente o
   modelo que queremos para transações sem side effect.
3. **Transação com escopo aninhado, rollback por exceção e fronteira de histórico explícita**
   (`transactions/index.ts:75-107,167-204`). O enum `merge|isolated|ignore|separate` é um bom
   vocabulário.
4. **Regras de edição pós-operação** (`EDITING_RULES`, `:31`) — ponha normalizações de lista/esquema
   aqui, não nos comandos.
5. **Snapshot de seleção serializável** com unidades documentadas (`selection/types.ts:56-86`).
6. **Índice de referências cacheado por identidade do documento** (`referenceDefinitions.ts:41-71`) —
   base do nosso resolvedor entre publicações.
7. **Falha de reconciliação → reconstrução total** (`editor/index.ts:617-638`). Barato de escrever e
   elimina uma classe inteira de bugs.

### Adaptar

- **Serializador autorizado a normalizar** (§3 de `arquitetura.md`): bom para legibilidade, ruim para
  "não mexer no arquivo do usuário". No nosso caso, expor política explícita (`preserve` vs `normalize`).
- **Plugins nomeados e deduplicados** (`muya.ts:142-152`): copiar o *padrão de registro*, mas no
  backend, para comandos/handlers, não para floats.
- **Opções que forçam reparse** (`PARSE_AFFECTING_OPTIONS`, `muya.ts:121-130`, 8 opções): manter essa
  distinção entre opção de render e opção de estrutura.

### Evitar

- **`contenteditable` + `innerHTML` por bloco**: é a raiz do custo de IME, caret, a11y e performance.
- **CSS embutido com fontes**: viola diretamente o requisito zero CSS.
- **Extensão por fork** (6 arquivos por bloco novo).
- **`name` string sem schema**: no Rust queremos enum fechado com `serde` e validação.
- **Persistir só Markdown**: sem um canal para metadados/estrutura, referências entre publicações
  terão de ser reinventadas fora do documento.

---

## 5. Confiança

**82%.** Código e documentação são abundantes e coerentes entre si; os testes existem e são
específicos (não são placeholders). Os 20% restantes: nada foi executado, os números de conformidade
foram lidos e não recalculados, e comportamento de IME/leitor de tela/performance em documento grande
só pode ser confirmado rodando. Se a revisão executar
`pnpm --filter @muyajs/core test:spec` e `pnpm -C packages/muya/e2e e2e:chromium`, a confiança sobe
para ~92%.

A rodada de revisão (`revisao.md`) recontou por comando todas as afirmações numéricas e conferiu
linha a linha as citações deste arquivo: 6 folhas importadas (2.555 linhas) num diretório de 7
arquivos (2.934), 16 `@font-face` em `68-203`, 21 formas de `TState`, 39 blocos, 17 plugins, 13
locales, **32 temas**, 57 opções, 8 opções que forçam reparse, 28 deps de runtime,
361/160/338 arquivos de teste e 593/652 + 600/672 de conformidade. As correções foram de citação,
de contagem **e de três notas** (usabilidade 4→3, API 4→3, documentação 5→4), justificadas em
`revisao.md` §2. O teto de confiança continua sendo a ausência de execução.

