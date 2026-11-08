# Revisão independente — dossiê Quill 2.0.3

Revisor: agente `QuillReview`. Rodada extra de revisão de [`_REVISAO-CHECKS.md`](../_REVISAO-CHECKS.md).
Alvo revisado: `dossie/quill/` contra `ref/quill/` (commit `539cbffd0a13b18e9c65eb84dd35e6596e403158`,
2025-07-25) e o tarball `quill-delta@5.1.0`.

## Veredito em uma frase

**Init rápido e Delta elegante, mas não é modelo de blocos nem zero CSS.** Confirmado: o Delta
é a fonte de verdade e o DOM/`blots` são projeção; o CSS estrutural (`core.styl`) é obrigatório
e os temas `snow`/`bubble` injetam DOM de chrome; não há schema, não é headless e não há
colaboração embutida.

## Achados (o que a revisão contestou)

1. **Licença do `quill-delta` estava errada — e era erro material de dossiê.**
   O dossiê dizia "MIT, Copyright (c) 2014-2023 Jason Chen". O `package/LICENSE` do tarball
   5.1.0 é **BSD-3-Clause, Copyright (c) 2022, Slab, Inc.** Corrigido em
   `codigo/INDEX.md` (tabela + seção de licença) e no cabeçalho de `codigo/05-quill-delta.ts`.
   Nenhum texto de análise dependia disso, mas a atribuição errada invalidaria a regra de
   proveniência do guia.

2. **Citações de linha do Delta estavam deslocadas** (o arquivo `05-quill-delta.ts` foi copiado
   do tarball, mas as linhas citadas não batiam):
   - `chop()` está em `:165-171`, não `:178-184` (essa faixa cai em `filter/forEach/map`).
   - `changeLength()` está em `:202-211`, não `:206-212`.
   - `length()` termina em `:217`, não `:223` (a faixa antiga invadia `slice`).
   - O comentário "Assume we are given a well formed ops" está em `:59`, não `:51-53`.
   Corrigido em `README.md`, `avaliacao.md` e `arquitetura.md`.

3. **`Block.delta()` vs `blockDelta`.** O `Block.delta()` (`block.ts:19-23`) é um cache que
   chama a função `blockDelta` (`:180-193`); o dossiê atribuía a injeção do `'\n'` a um método
   `Block.delta()` citando só `:183-193`. Corrigido para citar método e função.

4. **Lista de métodos da API com citação trocada.** A frase "`insertText`, `deleteText`,
   `formatText`, `setContents`, `updateContents`: `:304-340`, `:347-455`, `:569-613`..."
   mapeava `insertText` para `:304-340` (que é `deleteText`/`disable`/`enable`/`focus`).
   Substituída por citações explícitas por método (`deleteText :304-322`, `format* :347-455`,
   `insertText :585-613`, `setContents :702-720`, `updateContents :755-767`).

5. **`modify()` termina em `:919`, não `:929`** (a faixa antiga incluía o tipo
   `NormalizedIndexLength`). Corrigido em `README.md` e `arquitetura.md`.

6. **A11y subestimada pela premissa "não há testes".** `test/unit/ui/picker.spec.ts:44-129`
   contém **seis testes dedicados** de `aria-expanded`/`aria-hidden`/`aria-controls`, além das
   asserções de `aria-pressed` da toolbar (`toolbar.spec.ts:180-193`). A frase "testes de a11y
   são praticamente inexistentes" em `arquitetura.md §11` era falsa e foi corrigida. Isso
   sustenta a nota 3 (chrome com ARIA real; nó editável sem `role`/rótulo/`aria-live`).

7. **Números de execução não conferiam com as saídas armazenadas**:
   - Digitação de "Olá mundo": `saida-interact.json` registra **27 ms**, o dossiê dizia ~38 ms.
     Corrigido em `exemplos/README.md` e `arquitetura.md §12`, com nota de variância.
   - IME: `imeDuringComposition` é o documento **vazio** (`[{"insert":"\n"}]`) e `imeCommitted`
     é `[{"insert":"日本\n"}]`; o dossiê dizia `initial`/`initial日本`. O script limpa o editor
     antes do cenário. Corrigido em `exemplos/README.md` e `avaliacao.md §3`.
   - `history.ts` "atalhos `:64-82`": `Mod+Z` está em `:59-62`; corrigido para `:59-82` e
     explicitado que `Mod+Y` só é ligado quando `navigator.platform` casa `/Win/i`.
   - `userOnly` é `false` por default (`history.ts:28`); o texto dizia "`userOnly: true` ignora"
     sem marcar que é opção desligada. Reescrito.

8. **Citações de CSS com faixas imprecisas.** Conferido linha a linha em `core.styl`:
   `.ql-container` `:12-18`, `.ql-editor` `:40-52` (`white-space` em `:51`),
   `li`/`list-style-type` `:71-110` (não `:72-110`), `.ql-ui` `:205-206` (não `:202-204`),
   placeholder `:208-215` (correto). `snow.styl`/`bubble.styl` importam em `:8-10`, não
   `:9-11`. Corrigido em `README.md`, `arquitetura.md` e `avaliacao.md`.

9. **Anchor quebrado** de `README.md` para a seção de CSS (`#4-css-neutralidade` não existe).
   Corrigido para `#4-css-neutralidade-resposta-direta`.

10. **"`quill.core.css` é um artefato pequeno" sem número.** Adicionada a contagem real
    (`core` 215, `base` 309, `snow` 18+79, `bubble` 45+63, total 729) e a medição do probe
    (`font-family: Helvetica`, `font-size: 13px`, `p{margin:0}`, `li{list-style:none}`,
    `ol{padding-left:19.5px}`), para o custo do CSS estrutural ficar verificável.

## Confirmações (o que resistiu à checagem)

- **Delta como fonte de verdade**: `Editor.getDelta()` `editor.ts:162-166` concatena
  `line.delta()`; `Editor.update` tem o atalho `characterData` (`:279-308`) e o caminho geral
  com `oldDelta.diff(this.delta)` na linha 312; `applyDelta` normaliza CRLF e parte em linhas
  (`normalizeDelta` `:451-463`, `splitOpLines` `:465-482`); `setContents`/`updateContents`
  `quill.ts:702-720`/`:755-767`; `getSemanticHTML` `:543-552` é projeção, não storage.
- **Blots/Parchment**: `Scroll.allowedChildren`/`defaultChild` `scroll.ts:31-36`; `Block`
  tag `P` `:126-129`; `Inline.order` `inline.ts:8-18` (ordem canônica real);
  `Break.length() === 0` e remoção por `optimize` `break.ts`; `ListItem.requiredContainer`
  `list.ts:56`; `ParentBlot.uiClass = 'ql-ui'` `quill.ts:34`.
- **Seleção/IME**: `Range {index,length}` `selection.ts:31-36`; `normalizedToRange`
  `:272-296`; `rangeToNative` `:341-356`; `setNativeRange` `:358+`; `selectionchange` global
  com `setTimeout(...,1)` em `:63-67`; cursor blot `:150-176`; `Composition` com
  `batchStart()`/`batchEnd()` e `queueMicrotask` por bug do WebKit `composition.ts:15-45`.
- **Histórico**: pilhas de `{delta, range}` alimentadas por `EDITOR_CHANGE` (`:36-57`),
  `record` com `invert` + fusão por `delay: 1000` e `maxStack: 100` (`:111-136`),
  `change()` com `invert(getContents())` (`:85-101`), `transformStack` (`:161-180`).
- **Clipboard**: cópia usa `getSemanticHTML`; colagem prefere `text/html`, normaliza
  Word/Google Docs e sanitiza URL (`link.ts:5-27` → `about:blank`) e imagem
  (`image.ts:37-40` → `//:0`).
- **CSS/temas**: `core.styl` só, `snow`/`bubble` importam core+base; build publica
  `quill.core.css` como entry separado (`webpack.common.cjs:41-43`); `snow` adiciona
  `ql-snow` ao container (`snow.ts:103`) e injeta tooltip (`:112`) — observado no DOM salvo.
- **A11y**: `rootRole=null`, `rootAriaLabel=null`, `contenteditable=true` em
  `saida-probe.json`; toolbar com `role=toolbar` `toolbar.ts:34` e `aria-pressed` sincronizado;
  picker com `role=button`/`aria-*`; `uiNode` com TTL de 100 ms (`uiNode.ts:8`, deadline
  `:83-100`).
- **Métricas de código**: 64 `.ts`, 8.795 linhas; 8 blots, 11 módulos, 21 formatos; 729 linhas
  de Stylus; 521 `test(` em `test/unit`, 23 em `test/e2e`; 107/45/48/21 por spec
  (editor/selection/clipboard/history) — todos conferidos por contagem própria.
- **Cópias de código**: `01`–`04` são idênticas à fonte após o cabeçalho (verificado por
  `diff`); `05` é o `src/Delta.ts` do tarball (573 linhas + 9 de cabeçalho); `06` marca as
  faixas de `clipboard.ts` que copiou.
- **Exemplos**: `delta-steps.cjs` reexecutado em Node 22.23.2 produziu saída **idêntica** a
  `saida-delta.json`; `saida-probe.json` e `saida-interact.json` são coerentes com os números
  citados (exceto os itens 7 acima), inclusive `bench` (1.000 parágrafos ≈37 ms,
  `getContents` ≈0,1 ms, 200 inserts ≈334,6 ms, 1.000 nós) e `noCss.*` (que o próprio dossiê
  já marcava como não medindo ausência de CSS).

## Notas após a revisão

| Dimensão | Antes | Depois | Motivo |
|---|---:|---:|---|
| Usabilidade | 4 | **4** | Mantida: init de uma chamada, mas exige Delta/registry/CSS. |
| API | 4 | **4** | Mantida: pequena, tipada, sobrecargas; sem schema. |
| Neutralidade de estilo | 2 | **2** | Mantida e justificada (2 vs 3 vs 1) em `avaliacao.md §4`. |
| Modelo de dados | 4 | **4** | Mantida; só as linhas citadas foram corrigidas. |
| Extensibilidade | 4 | **4** | Mantida. |
| Performance | 3 | **4** | Subiu: 48.890 caracteres em ~37 ms, `getContents` ~0,1 ms, cache por bloco e caminho rápido de digitação; o custo O(n) só aparece no padrão patológico de 200 inserts de API. Nota 5 exigiria benchmark oficial/estresse, que não existe. |
| Acessibilidade | 2 | **3** | Subiu: ARIA do chrome é real **e testado** (toolbar + 6 testes de picker), teclado e `uiNode`; o nó editável continua sem `role`/rótulo/`aria-live` e não há teste com leitor de tela. É "contenteditable básico" (3), não "boa base com casos faltando" (4). |
| Documentação | 3 | **4** | Subiu: o site/guia vive versionado no monorepo (`packages/website`), há CHANGELOG detalhado e `.d.ts` publicados; falta guia de migração no repo e não auditei página por página (por isso não é 5). |
| Manutenção | 4 | **4** | Mantida: 2.0 GA 2024, releases 2.0.1–2.0.3, CI, testes em browser real, E2E e fuzz. |
| Adequação ao `rich-editor-auy` | 3 | **3** | Mantida: ótimo modelo, stack de execução inapropriada (DOM-bound, CSS, não headless). |

**Média:** 3,3 → **3,6/5**.

Sobre a pergunta da rubrica para neutralidade: a nota 2 **não é generosa nem severa**, é a
única sustentável. 3 exigiria que trocar o CSS fosse um caminho suportado; na prática os
seletores estruturais (`.ql-ui`, `li[data-list]`, `ql-code-block-container`) vêm dos blots e
precisam ser reimplementados junto com a folha. 1 seria severo porque existe `quill.core.css`
oficial e `theme: 'default'` sem chrome.

## Confiança

**87%.** Motivos para não ser maior:

- O build/`npm ci` do monorepo **não foi executado** (sem `node_modules`); a verificação
  comportamental usa o pacote publicado 2.0.3 (mesma versão declarada), via CDN.
- Só Chromium 156 headless; nada de Firefox/Safari.
- Sem leitor de tela (NVDA/VoiceOver) e sem axe; a11y julgada por atributos, testes e código.
- Sem IME de sistema (a composição foi simulada por CDP, como nos E2E do próprio Quill).
- Sem teste de estresse/virtualização (1.000 parágrafos é "médio").
- Colaboração não exercitada (só primitivas no código).
- Migração 1.x→2 não auditada; site de docs não lido página a página.

## Não executado nesta revisão (e por quê)

- `npm ci && npm run build -w quill && npm test -w quill` — exigiria instalar todo o
  devDependencies e browsers; o custo não muda a conclusão sobre modelo/CSS, mas impede
  declarar confiança ≥90%.
- Reexecução de `probe.html`/`interact.cjs`/`minimal.html` — as saídas armazenadas
  (`saida-*.json`, `saida-minimal-dom.html`) foram conferidas contra o texto e são internamente
  consistentes; o revisor só reexecutou o exercício de Delta (Node puro, sem browser).
- Auditoria de compatibilidade de navegadores, leitor de tela e IME real.

## Arquivos alterados nesta revisão

- `README.md` — Delta/lines, `blockDelta`, métodos da API, `modify`, faixas de `core.styl`,
  custo do CSS em números, anchor corrigido.
- `arquitetura.md` — invariantes do Delta, `modify`, `uiNode`, atalhos de histórico,
  `userOnly`, `getSemanticHTML`, faixa de `.ql-ui`/placeholder, tempo de digitação, parágrafo
  de testes de a11y.
- `avaliacao.md` — notas 3→4 (performance, docs) e 2→3 (a11y), média 3,6, evidências
  corrigidas, justificativa da neutralidade 2, IME, item de licença da matriz, lacunas e
  confiança.
- `codigo/INDEX.md` e `codigo/05-quill-delta.ts` — licença BSD-3-Clause do `quill-delta`.
- `exemplos/README.md` — 27 ms, IME real, nota de reprodução do `delta-steps.cjs`.
- `revisao.md` — este arquivo.

Fora do escopo (não alterado): `ref/`, outros dossiês e `dossie/QUADRO-COMPARATIVO.md`. Se o
quadro repetir as notas antigas de Quill (performance 3, a11y 2, docs 3), ele precisa ser
sincronizado pelo dono do arquivo.
