# Avaliação do ProseMirror

Escopo: pacotes publicados (model 1.25.12, transform 1.12.5, state 1.4.4, view 1.42.9,
schema-basic 1.2.5, history 1.5.1, collab 1.3.1). Fontes e hashes em `codigo/INDEX.md`.
Execuções: `exemplos/demo.js` (Node 22, sem DOM) e comandos registrados em `exemplos/README.md`.

## 1. Rubrica (0–5)

| dimensão | nota | evidência | comentário |
|---|---:|---|---|
| **usabilidade** | **2** | `prosemirror-commands@1.7.2/src/commands.ts:752-760` (baseKeymap com 8 bindings, sem Mod-b/Mod-i/Mod-z); `ref/prosemirror/demo/index.html:6-10` (5 folhas de estilo); `view/src/index.ts:519-533`; `npm view prosemirror` → 0.11.1 `deprecated` | Como biblioteca, a edição é excelente quando montada (IME, colagem, undo). Como "editor", não existe: o usuário precisa de schema, keymap, menu, CSS e a11y. Nota 2 é para o produto que o usuário recebe ao instalar, não para a qualidade interna. |
| **API** | **4** | `node.ts:22-36`; `step.ts:16-69`; `transaction.ts:42-60`; `plugin.ts:7-47` | Coerente, tipada (TS), imutável, com nomes estáveis há anos. Perde 1 por superfície ampla (dezenas de exports por pacote) e por exigir entender `assoc`/`openStart` antes de produzir algo correto. |
| **neutralidade de estilo** | **2** | `prosemirror-view-1.42.9/src/README.md:5-6` ("Make sure you load style/prosemirror.css"); `codigo/prosemirror.css:1-30`; `index.ts:519` (classe global `ProseMirror`) | Não é um tema, mas exige CSS próprio e classes globais; sem o arquivo, quebras de linha e seleção de nó ficam erradas. O CSS é pequeno e funcional, mas o contrato "zero CSS" não existe. **Ressalva de rubrica:** o descritor de 2 fala em "tema pesado", que não é o caso; a nota 2 se sustenta pelo requisito vinculante de stylesheet + seletores globais `^\.ProseMirror`, e não pelo peso visual. Se a rubrica fosse lida só pelo peso do tema, 3 seria defensável. |
| **modelo de dados** | **5** | `node.ts:54` (nodeSize), `replace.ts:13-41` (Slice), `resolvedpos.ts:12-200`, `mark.ts:22-47`, `schema.ts:558-620` | Árvore imutável com compartilhamento, marcas com rank/exclusão, schema validado por autômato, slices com lados abertos. É o motivo de ProseMirror ser referência. |
| **extensibilidade** | **4** | `plugin.ts:95-127` (StateField), `state.ts:137-170` (appendTransaction), `step.ts:53-60` (jsonID), `decoration.ts:123-300` | Quatro eixos independentes (schema, plugin, step, decoração). Perde 1 porque não há composição genérica de schema (só funções específicas como `addListNodes`; cada app remonta o schema) e porque `appendTransaction` pode rodar em laço entre plugins. |
| **performance** | **4** | `node.ts:138-157`/`fragment.ts:73-104` (persistência), `viewdesc.ts:773-930` (reconciliação), `domobserver.ts:83-97`; tamanhos gzip medidos (29,9/20,4/9,4/61,2 KB) | Estruturas persistentes e atualização incremental são sólidas. **Nota estrutural, não medida**: não medi throughput; benchmarks oficiais (`ref/prosemirror/demo/bench/`) não foram executados. |
| **acessibilidade** | **1** | `grep -rn "aria" prosemirror-view-1.42.9/src/` → nenhum uso real (só "variants"/"variant" em `decoration.ts:122` e `viewdesc.ts:1499`); emissão apenas de `class`, `contenteditable`, `translate="no"` em `index.ts:519-533` | Sem `role`, `aria-*`, labels ou árvore acessível própria. `contenteditable` não basta para leitores de tela; o integrador deve suprir. |
| **documentação** | **4** | comentários JSDoc geram a API (`src/README.md` de cada pacote, ex. `prosemirror-view-1.42.9/src/README.md`); demo funcional em `ref/prosemirror/demo/` | API documentada nos fontes e site com guia/exemplos. Não há guia de a11y nem "receita" de produção no repositório; e o guia vive fora do `ref/`. |
| **manutenção** | **5** | `npm view prosemirror-model time.created` → 2016-09-21; `prosemirror-model-1.25.12/CHANGELOG.md:1` → 2026-09-21; séries `1.x` estáveis; MIT (`codigo/LICENSE`) | Dez anos, releases frequentes, changelog por pacote, mantenedor ativo, sem reescritas quebradas. |
| **adequação ao nosso caso** | **3** | contraste em `README.md:§7` | Copiar model/transform/state/schema é altíssimo valor. A view é o oposto do nosso contrato (headless, zero CSS): seu valor é como *catálogo de problemas* que o front-end terá de resolver. |

Média simples: **3,4/5**. A nota alta em modelo/manutenção e baixa em a11y/estilo é exatamente o
perfil de "fundação para produto", não de produto.

## 2. Matriz de usabilidade (18 categorias)

Legenda de evidência: **EX** = executado aqui; **CÓD** = verificado no código-fonte;
**TESTE** = verificado na suíte (leitura, não execução); **DOC** = documentação/registro;
**NÃO** = não verificado.

| # | categoria | status | evidência |
|---:|---|---|---|
| 1 | instalação e pré-requisitos | **EX** | `npm install prosemirror-model@1.25.12 prosemirror-transform@1.12.5 prosemirror-state@1.4.4 prosemirror-history@1.5.1 prosemirror-schema-basic@1.2.5 prosemirror-commands@1.7.2` → 9 pacotes, 2,5 MB (`exemplos/README.md:§1`) |
| 2 | inicialização mínima | **EX** (headless) + **CÓD** (view) | `EditorState.create` + `schema.topNodeType.createAndFill()` executados; `ref/prosemirror/demo/demo.ts:1-16` mostra os 6 imports + `EditorView` |
| 3 | edição básica (texto/parágrafo) | **EX** | `exemplos/demo.js`: replace 1..6 → `"Howdy world"`; `AddMarkStep` strong aplicado |
| 4 | negrito/itálico/link | **EX** (strong) + **CÓD** (em/link) | marca `strong` executada; `schema-basic.ts:109-160` define `link`/`em`/`strong`/`code` |
| 5 | seleção | **EX** | seleção `1..6` mapeada para `2..7` após inserção (`exemplos/demo.js`); `selection.ts:9-95` |
| 6 | teclado / keymap | **CÓD** | `keymap.ts:76-108`; `commands.ts:752-786` (baseKeymap mínimo) |
| 7 | undo/redo | **EX** | `undoDepth=2` após dois passos; undo removeu marca e depois o replace; duas digitações adjacentes agruparam em 1 (`exemplos/README.md:§3`) |
| 8 | clipboard | **CÓD** | `clipboard.ts:6-121` (HTML+texto, `data-pm-slice` em `:35`); não executado (exige DOM) |
| 9 | IME/composição | **CÓD** + **TESTE** | `input.ts:527-620`; suíte `webtest-composition.ts` (307 linhas) existe no clone git, **não executada** |
| 10 | schema e restrições de conteúdo | **EX** | `RangeError: Invalid content for node paragraph: <paragraph>`; `content.ts:23-110` (NFA/DFA, `fillBefore`, `findWrapping`) |
| 11 | custom node/mark/plugin | **EX** (plugin) + **CÓD** | `history()` é um plugin com `StateField` (`history.ts:392-435`); novos nós/marcas por `NodeSpec`/`MarkSpec` (`schema.ts:358-530`) |
| 12 | persistência/serialização | **EX** (JSON doc) + **CÓD** (estado/HTML) | `doc.toJSON()` executado; `state.ts:217-260` (`EditorState.toJSON/fromJSON`); `to_dom.ts:27-140` e `from_dom.ts:182-340` |
| 13 | renderização e CSS | **CÓD** + **DOC** | `codigo/prosemirror.css`; `src/README.md:5-6`; demo com 5 stylesheets (`demo/index.html:6-10`); **não executado em browser** |
| 14 | acessibilidade | **CÓD** | ausência de `aria`/`role` em `view/src/*` (`grep` registrado em `arquitetura.md:§7.4`) |
| 15 | colaboração | **CÓD** | `collab.ts:14-32` (`rebaseSteps`), `collab.ts:162-180` (`sendableSteps`), plugin com `historyPreserveItems` (`collab.ts:95`) |
| 16 | performance e limites | **CÓD** + **DOC** (não medido) | `viewdesc.ts:773-930`, `domobserver.ts:83-97`; benchmarks `demo/bench/` existem, **não executados** |
| 17 | erros/diagnóstico/migração | **EX** + **CÓD** | `RangeError` observado; `ReplaceError` (`replace.ts:8`), `StepResult.fail` (`step.ts:71-91`); migração documentada nos CHANGELOGs por pacote |
| 18 | licença e maturidade | **DOC** | MIT (`codigo/LICENSE`), npm 2016-09-21, release 2026-09-21 (`CHANGELOG.md:1`) |

**Cobertura:** 18/18 categorias com **alguma** evidência (100%); **10/18** com **execução real**
(56% — itens 1, 2, 3, 4, 5, 7, 10, 11, 12, 17); as categorias dependentes de DOM (clipboard, IME,
renderização, a11y, colaboração) estão verificadas por código/testes, não por execução — declarado
explicitamente. A meta de 85% de *cobertura de evidência* é, portanto, atingida; a meta de execução
não, por limite de ambiente.

## 3. O que o usuário sente (síntese qualitativa com evidência)

- **"Instalei e não tenho editor."** Verdadeiro, e pior do que parece: `npm view prosemirror`
  devolve **0.11.1** (2015-09-03), marcada `deprecated` — "This library has been split up into
  separate modules… no longer maintained". Quem instala o pacote de nome óbvio recebe a API
  monolítica antiga, não as peças atuais; os pacotes úteis são `prosemirror-view`/`-state`/`-model`
  etc. `ref/prosemirror/README.md:27-31` diz explicitamente para parar de ler se você só *usa*.
- **"Não consigo negritar."** Esperado: `baseKeymap` não tem Mod-b
  (`commands.ts:752-760`); é preciso `toggleMark` + `keymap`.
- **"Meu Enter não cria parágrafo direito."** Só funciona com `splitBlock`/`splitListItem`
  encadeados (`commands.ts:752`, `example-setup/src/keymap.ts:85-87`).
- **"Colar perdeu a lista."** A estrutura depende de `data-pm-slice` e do `Fitter`
  (`clipboard.ts:35`, `replace.ts:54-120`); colar de fora do ProseMirror perde `openStart/openEnd`.
- **"Undo desfez demais/pouco."** Agrupamento por tempo e adjacência (`history.ts:377-394`);
  `closeHistory`/`addToHistory: false` são as válvulas (`history.ts:367`, `history.ts:278`).
- **"Leitor de tela não anuncia."** Sem `role`/`aria`; precisa `attributes`
  (`index.ts:791`) e labels no host.
- **"Funciona em documento grande?"** Bem, graças a persistência e reconciliação
  (`node.ts:138`, `viewdesc.ts:773`), mas não há número medido neste dossiê.

## 4. Limites deste dossiê / não verificado

1. **View não executada em navegador.** Sem browser/jsdom no ambiente; toda afirmação de
   renderização, seleção DOM, IME e clipboard vem de leitura de código e da existência das suítes
   `webtest-*.ts` (3.271 linhas em `prosemirror-view/test/`; 8.932 somando os testes de todos os
   pacotes clonados), **não executadas**.
2. **Benchmarks não executados** (`demo/bench/*` exige browser + build do meta-repo).
3. **Módulos periféricos não analisados a fundo**: `prosemirror-tables` (não obtido: o clone não
   concluiu no espelho usado, e a revisão não conseguiu repeti-lo — o acesso pediu credenciais),
   `prosemirror-menu`, `prosemirror-dropcursor`, `prosemirror-gapcursor`, `prosemirror-markdown`.
   Citados só como inventário em `bin/pm.js:9-13`.
4. **Divergência tarball × git HEAD**: o HEAD do git tem commits pós-release (ex.: `v8InternHack`
   em `node.ts`, ausente no tarball; doc de `validate` em `schema.ts` difere). Todas as citações
   são do **tarball publicado**, conforme instrução de coordenação.
5. **`ref/prosemirror/` não contém `src/`** — nenhuma citação aponta para `ref/` além de
   `README.md`, `package.json`, `bin/pm.js` e `demo/`. Isso é registrado em `README.md:§1`.
6. **Números de linha dos testes** não foram usados como evidência (não copiados nem executados);
   cito apenas contagem de linhas e nomes de arquivo.

## 5. Confiança

- Sobre **modelo/transform/state/schema**: alta (leitura integral das peças centrais + execução
  real de posições, mapeamento, passos, marcas, undo/redo).
- Sobre **view/IME/clipboard/renderização**: média-alta (código lido, testes existentes, nenhuma
  execução).
- Sobre **performance**: média-baixa (arquitetura entendida, nenhuma medição).
- Sobre **maturidade/licença**: alta (metadados npm + changelog + LICENSE).
