# Revisão independente — MarkText / Muya

Revisor: agente independente (segunda rodada). Escopo: `dossie/marktext/` contra
`ref/marktext/` (commit `a520bef`, 2026-11-06, MIT). Nada de `ref/`, `plan/`, `AGENTS.md` ou
outros dossiês foi alterado.

**Método.** Leitura integral do dossiê, depois verificação por comando de cada número e de cada
citação de linha: `wc -l`, `grep -n`, `find | wc -l`, `python3` para contar uniões de tipos, chaves
de interface e para casar os trechos copiados contra os originais (`difflib`). **Build,
`test:spec` e Playwright não foram executados** — ver §4.

> Nota de contexto: uma rodada anterior já havia corrigido várias citações e deixado um
> `revisao.md`. Reli o dossiê nesse estado, revalidei tudo do zero e **corrigi erros que aquela
> rodada introduziu ou deixou passar** (§1, achados A4–A8). Ela também confirmou corretamente
> 21 formas de `TState` e 57 chaves de `IMuyaOptions`.

---

## 1. Achados

Severidade: **A** = afirmação central errada (muda conclusão ou induz a erro de integração);
**B** = citação/contagem errada que não muda a conclusão; **C** = imprecisão de redação.

### Achados A

1. **Contabilidade do CSS estava errada.** O dossiê dizia "6 folhas importadas … o diretório tem 7
   arquivos que somam 2.826 linhas — o sétimo, `exportStyle.css`, entra no export". Medido:
   `src/assets/styles/*.css` (6 arquivos top-level) = **2.826 linhas**, exatamente — o número
   inclui `exportStyle.css` (379 linhas). O sétimo arquivo é
   `prismjs/light.theme.css` (**108 linhas**), que **é importado** (`src/muya.ts:48`). Total dos 7
   arquivos = **2.934**; total realmente importado no entry = **2.555** (`2826 − 379 + 108`).
   O `exportStyle.css` continua não importado, mas só entra no export
   (`src/state/markdownToHtml.ts:6`). Corrigido em `README.md`, `arquitetura.md` e `avaliacao.md`.

2. **Faixa dos `@font-face` errada**: o bloco vai de `index.css:68` a **`:203`** (último
   `@font-face` fecha em 203). O dossiê citava `68-225`, que cai em `@keyframes`/`html,body`.

3. **"8 pesos de Open Sans" está errado tecnicamente**: são **8 `@font-face`** cobrindo **4 pesos**
   (300/400/600/700) × normal+itálico, mais 6 `@font-face` de DejaVu Sans Mono, Noto Music e Noto
   Sans Math = **16 em `index.css`**. (Há mais 2 em `exportStyle.css`; 18 no diretório.)

### Achados B

4. **Temas do desktop: 32, não 35.** `packages/desktop/src/renderer/src/assets/themes/*.theme.css`
   são 32 arquivos, e 32 contêm `--editorColor`. Corrigido nos três documentos.

5. **52 opções no `editor.vue` está certo, a faixa não**: o objeto vai de **1724 a 1779** (51 chaves
   com `:` + `beforeImageDrop,` em shorthand = **52**). `mermaidTheme`/`vegaTheme` entram depois por
   `Object.assign` em 1781-1787. O dossiê citava `1724-1782`.

6. **`packages/desktop/test` tem 338 arquivos, não 314** (`find packages/desktop/test -type f`).

7. **`init()` termina em `:253`**, não em `:258` (246-253). A faixa `246-258` transborda para
   `refreshShortcutHints`.

### Achados C

8. **Parser**: o fallback de documento vazio (`states.length ? states : [{name:'paragraph',…}]`)
   está na **linha 101**, não 102; o laço `while` em 94-99, com a declaração da pilha em 92.
9. **Serializer**: `_convertStatesToMarkdown` começa em **113** (o `generate` em 108-111); o
   `rewrite-required` e o fence de code-block ficam em `_prepareBlockBoundary`, **170-180**
   (decisão na 174). O dossiê citava `110-167` e `169-181`.
10. **`codigo/INDEX.md` afirma que "cada arquivo preserva o cabeçalho com origem, linhas, commit e
    licença"**. Falso para 3 dos 6: `state-types.ts`, `json-state.ts` e `edit-transactions.ts` são
    **cópias verbatim** (sem cabeçalho), justamente para bater linha a linha com a fonte. Só os
    três trechos parciais têm cabeçalho. Redigido para distinguir os dois casos.
11. **`diagram/render.ts:20-52`**: o que dá nome acessível é `accessibleName` (20-44) +
    `applyFinalizations` (46-56, que seta `role="img"`/`aria-label`). Faixa ajustada para `20-56`.

### Achados que se **confirmaram** na fonte

- **app × engine**: `packages/desktop` (Electron, Vue) consome `packages/muya` por `workspace`;
  `@muyajs/core` 0.2.0 declara `publishConfig` mas o README diz que **não é publicado deste
  repositório** (`packages/muya/README.md:269-271`) e avisa que não é produção (`:5`).
- **Markdown no disco × `TState[]` em memória**: o desktop só persiste Markdown
  (`json-change` → `getMarkdown()` → gravação atômica); `getState()` devolve clone profundo
  (`src/state/index.ts:135-136`). O JSON nunca é gravado.
- **`flush()`/`getMarkdown()`**: `flush()` em `transactions/index.ts:167`; `getMarkdown()` em
  `state/index.ts:139-141`.
- **renderer `contenteditable`/`innerHTML`**: raiz trocada por `div[contenteditable]` em
  `getContainer` (`src/muya.ts:1618-1646`) e cada folha reescrita por `innerHTML` em
  `src/inline/index.ts:70-84`.
- **União `TState` = 21 formas** (11 folhas em `types.ts:208-219`, 10 contêineres em `:221-231`,
  união em `:233`); predicados em `:237-265`.
- **57 chaves em `IMuyaOptions`** (`src/types.ts:9-126`).
- **`PARSE_AFFECTING_OPTIONS` = 8** (`src/muya.ts:121-130`).
- **17 classes de plugin** (`static pluginName` menos 4 specs) em **22 diretórios** de `src/ui/`.
- **13 locales** (+ `index.ts` = 14 arquivos).
- **6 extensões associadas** no app (`electron-builder.yml:82-118`; `INSTALLATION.md:14`).
- **PlantUML remoto por default** (`src/config/index.ts:339`).
- **Conformidade 593/652 (91,0%) e 600/672 (89,3%)** (`test/spec/conformance.md:19-20`).

---

## 2. Notas da rubrica: confirmações e contestações

Notas originais: usabilidade 4, API 4, neutralidade 1, modelo 4, extensibilidade 2, performance 3,
a11y 2, docs 5, manutenção 4, fit 3 (32/50).

| Dimensão | Antes | Agora | Decisão |
|---|---|---|---|
| Usabilidade | 4 | **3** | **Contestada.** O init em si é curto, mas "instalar" não existe: a engine não é publicada; consumir exige clone + `pnpm install` no monorepo (Electron, módulos nativos) ou build manual. Somam-se 17 plugins a registrar e CSS obrigatório. Rubrica 4 = "init simples e API clara"; 3 = "funciona, com atrito" — é o caso. |
| API | 4 | **3** | **Contestada.** Tipada e pequena, mas: versão 0.2.0 com aviso explícito de quebra entre minors (`README.md:5`), inserção de bloco por *labels* stringly-typed (`muya.ts:62-92`, `updateParagraph`), `flush()` obrigatório antes de ler. É "utilizável" (3), não "difícil de usar errado" (4). |
| Neutralidade | 1 | **1** | Confirmada. CSS não é opcional. |
| Modelo | 4 | **4** | Confirmada. União discriminada + `ot-json1`. |
| Extensibilidade | 2 | **2** | Confirmada. Documento = fork em 6 pontos; plugins só de UI. |
| Performance | 3 | **3** | Confirmada. Único orçamento é 60 s para `setContent` de 10k. |
| A11y | 2 | **2** | Confirmada. `textbox` único, axe só `critical`. |
| Documentação | 5 | **4** | **Contestada.** 290 linhas de README com "API at a glance", arquitetura e limites — bom (4). Não há referência de API completa nem guia de migração, e o próprio pacote se diz instável. 5 exige "migração e limites" documentados. |
| Manutenção | 4 | **4** | Confirmada. 14 workflows em `.github/workflows/`, 361 specs unit, 160 Playwright, suíte de conformidade e sanitização. |
| Fit | 3 | **3** | Confirmada. Referência de desenho, não de implementação. |

**Soma: 29/50, média 2,9** (antes 32/50, 3,2). As três contestações se sustentam em evidência de
instalação/versionamento, não em gosto.

---

## 3. Correções aplicadas

| Arquivo | Correção |
|---|---|
| `README.md` | CSS: 6 importadas (2.555 linhas), 7 arquivos (2.934), `exportStyle` só no export; `@font-face` em `68-203` com 4 pesos × 2 estilos; 32 temas; notas 3/3/4 e soma 29/50; `getContainer`/perf mantidos |
| `arquitetura.md` | mesmas correções de CSS/fontes/tema; parser `90-101` (fallback na 101, laço 94-99); serializer `113-167`/`170-180`; `readOnly.ts:5-15` e `diagram/render.ts:20-56`; 32 temas |
| `avaliacao.md` | tabela de notas com 3/3/1/4/2/3/2/4/4/3 e soma 29/50; matriz de 13 eixos com CSS corrigido; 338 arquivos em `desktop/test`; `68-203`; 32 temas |
| `codigo/INDEX.md` | esclarece que 3 arquivos são cópias verbatim sem cabeçalho e 3 trechos parciais têm cabeçalho; faixas do serializer/parser corrigidas |
| `exemplos/README.md` | status re-checado (sem `node_modules`), faixa de opções `1724-1779` |
| `codigo/*.ts` | **não alterados** — casam literalmente com a fonte nas faixas declaradas (só marcadores de início/fim em `content-block.ts`) |

**Nenhum trecho copiado foi editado.** `difflib` confirma: `state-types.ts` 289/289,
`json-state.ts` 159/159, `edit-transactions.ts` 232/232 idênticos; `content-block.ts` idêntico a
`content.ts:356-434` fora os dois comentários-marcador.

---

## 4. O que não foi verificado (e por quê)

Nenhuma dependência está instalada (reconfirmado: `node_modules` ausente na raiz e em
`packages/*`). `pnpm-lock.yaml` existe, mas instalar o monorepo Electron está fora do escopo.

| Não executado | Por quê | Como verificar |
|---|---|---|
| `pnpm --filter @muyajs/core test:spec` | sem `node_modules` | `pnpm install`; comparar com `test/spec/conformance.md` |
| Playwright (160 specs) | sem deps nem navegador | `pnpm -C packages/muya/e2e e2e:install && pnpm -C packages/muya/e2e e2e` |
| `minimal.ts` / demo | sem `vite`/resolução de `@muyajs/core` | `pnpm --filter muya-examples dev:demo` |
| 593/652 e 600/672 | lidos de `conformance.md`, não recalculados | `test:spec` |
| IME real de SO | a suíte simula `composition*` no DOM | teste manual pinyin/japonês |
| Leitor de tela e axe `serious` | sem ambiente | rodar axe sem filtro `critical` |
| Latência de digitação em doc grande | o repo não mede | profiling com 10k parágrafos |

---

## 5. Confiança final

**82%** — faixa 75–89% da rubrica ("código e docs fortes; execução parcial ou ausente em um eixo").
Todas as afirmações numéricas e citações de linha do dossiê foram revalidadas por comando, e os 6
trechos de `codigo/` casam literalmente com a fonte. Não subo para 90% porque **nada foi
executado**: usabilidade, IME, a11y com leitor de tela, latência por tecla e round-trip em corpus
real continuam inferidos de código e de testes escritos.

**Teto se `test:spec` + Playwright rodarem: ~92%.**

---

## 6. Lacunas que permanecem

1. **Latência de edição** não medida; o único orçamento é `setContent` de 10k em < 60 s.
2. **A11y observada** ausente; axe ignora `serious`/`moderate`.
3. **Round-trip em arquivo real** não medido (só fixtures e a suíte de conformidade).
4. **Custo de `getState()`** (clone profundo) em documento grande não medido.
5. **Publicação**: `publishConfig` existe no pacote, mas o README diz que a release vive no upstream
   `marktext/muya`; não há tag/release local para conferir.
