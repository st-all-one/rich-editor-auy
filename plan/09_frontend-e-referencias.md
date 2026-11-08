# 09 — Front-end, zero estilo e referências entre publicações

> **Contrato.** Este documento estende [`06_api-e-integracao.md`](06_api-e-integracao.md); não o
> substitui. O core continua headless. A integração é **dados primeiro**: o documento canônico é a
> fronteira, e PHP, TypeScript, React, canvas ou uma UI nativa são clientes dele.

---

## 1. A exigência e a consequência

Quatro requisitos novos, todos de produto:

1. uso em front-end **PHP** e **TypeScript**;
2. implementação **extremamente fácil**;
3. **nenhuma interferência na estilização** do consumidor;
4. dados profundos o bastante para **referências entre publicações** com facilidade.

Eles não mudam a tese do projeto; mudam a **superfície de integração**. Headless é necessário, mas
não suficiente: sem um adaptador e um contrato de estilo explícitos, "headless" vira "faça você
mesmo" e a promessa de facilidade morre.

A consequência dura é:

- **o core não renderiza, não injeta CSS e não é dono do DOM**;
- **o adaptador é obrigatório para UI e não define estilo**; ele mapeia a `A11yTree` para elementos
  nativos/ARIA, input, IME e eventos; a árvore não pode ser desligada;
- **PHP não reimplementa o modelo**; ele transporta bytes canônicos e delega validação ao CLI;
- **referências são dados**, não âncoras de DOM nem URLs frágeis; a resolução vive em uma camada
  separada (`refs`), não no core.

---

## 2. As superfícies de integração

| Superfície | Quem usa | O que faz | O que não faz |
|---|---|---|---|
| **JSON canônico + envelope** | PHP, TS, Python, Go, servidor | armazena e transporta o documento; versão e fingerprint | não valida schema/steps; não renderiza |
| **WASM/JSR** (`@rich-editor-auy/core`) | TS/JS no browser, Deno 2.0, edge, worker | executa o core; devolve change sets | não toca no DOM; não conhece CSS |
| **Adaptador DOM** (`@rich-editor-auy/dom`) | TS/JS | input, IME, seleção, clipboard, mapeamento da `A11yTree`, render semântico | não define estilo, não vira UI kit, não inventa semântica |
| **Composer PHP** (`rich-editor-auy/php`) | PHP 8.3+ | envelope, bytes preservados, validação via CLI | não reimplementa schema/steps; não abre rede |
| **CLI** (`rich-editor-auy`) | PHP, CI, scripts | validar, formatar, comparar, renderizar com política | não é runtime de editor |
| **`refs`** | servidor, TS, CLI | indexar documentos, resolver referências, backlinks, dangling | não armazena, não faz rede, não é CMS |

### 2.1 Por que PHP não ganha um porte do core

Um porte de schema/steps para PHP seria uma **segunda fonte de verdade**. Duas implementações de
normalização, fingerprint e mapping divergem no primeiro caso de borda; o consumidor descobre com
documento corrompido. PHP também não precisa entender steps para armazenar bytes: o navegador edita,
o PHP persiste. O pacote PHP é fino de propósito:

- valida o **envelope** (formato, versão, fingerprint) e a forma do JSON;
- preserva os **bytes canônicos** sem resserializar;
- opcionalmente chama o **CLI** para validação estrita e renderização com política;
- nunca reimplementa schema, `Step`, `Mapping` ou normalização.

Se o servidor precisa modificar conteúdo, há dois caminhos honestos: editar no cliente (JS/WASM) ou
usar o CLI. Mutar JSON canônico em PHP é o caminho para bytes não canônicos e referências quebradas.

---

## 3. TypeScript: pacotes e caminho fácil

### 3.1 Pacotes

| Pacote | Conteúdo | DOM? | CSS? |
|---|---|---|---|
| `@rich-editor-auy/core` | WASM do core: `Schema`, `Doc`, `EditorState`, `Transaction`, `ChangeSet` | não | não |
| `@rich-editor-auy/schema` | presets (`basic`, `document`) e builder que **emite** schema canônico | não | não |
| `@rich-editor-auy/dom` | adaptador: `mount`/`bind`, input, IME, seleção, clipboard, mapeamento de `A11yTree` | sim, só no host | **nenhum** |
| `@rich-editor-auy/react` etc. | wrappers finos, comunidade/opcional | sim | nenhum |

O `core` é a autoridade. O builder de schema em TS é um **serializador**, não um validador: se o
schema for inválido, o WASM devolve `SchemaError` no `init`/`fromJSON`; não há duas regras.

`crates/rich-editor-auy-wasm` é um crate separado, pode usar `std` e o glue do `wasm-bindgen`; ele
não é o core publicado. O core continua `no_std + alloc` e `forbid(unsafe_code)`; nenhuma linha
publicada usa `unsafe`.

### 3.2 Caminho de 10 linhas

```ts
import { init, EditorState } from "@rich-editor-auy/core";
import { mount } from "@rich-editor-auy/dom";
import { basicSchema } from "@rich-editor-auy/schema";

await init();
const schema = basicSchema();
const state = EditorState.fromEnvelope(schema, savedBytes);
const editor = mount(document.querySelector("#editor")!, {
  state,
  onChange: (change) => save(change.toJSON()),
});
// nenhum import de CSS; #editor é o host e continua sendo seu.
```

`mount` devolve `{ destroy, view, dispatch }`. `destroy()` restaura o host ao estado anterior.
`view()` devolve dados de renderização (blocos, runs, ranges, seleção); o consumidor pode ignorar o
renderer embutido e desenhar o que quiser.

### 3.3 Modos do adaptador

| Modo | O adaptador faz | O consumidor faz | Uso |
|---|---|---|---|
| `data` | nada de DOM; só core | renderiza e trata eventos; assume explicitamente a a11y renderizada (a `A11yTree` continua disponível) | controle total, zero DOM da lib |
| `bind` | eventos/seleção/IME sobre um DOM que o consumidor já mapeou | fornece o mapa `data-auy-pos` e garante o mapeamento semântico | avançado, integração com framework |
| `mount` | render **semântico nativo** (`<h1>`, `<ul>`, `<a>`, `<img alt>`) + `A11yTree` + eventos | estiliza o host e os `data-auy-*` | caminho fácil e acessível |
| `mount({ semantics: "neutral" })` | `div`/`span` + ARIA derivada da árvore | assume defaults e testa leitor de tela | zero default visual explícito |

O default é `mount` com **semântica nativa**: `<h1>`, `<ul>`, `<a>` e `<img alt>` são mais
confiáveis que ARIA em `div`/`span`. `semantics: "neutral"` é opt-in para quem precisa de zero
default do user agent; ele não é o default e exige teste real de leitor de tela. `data`/`bind`
transferem a renderização — e a responsabilidade de a11y — de forma explícita.

### 3.4 Erros e tipos

O core devolve `Result`. A borda JS traduz para `AuyError` com `code`, `path` e `offset`; nada de
`any`, nada de erro opaco. `ChangeSet` e `view()` são tipos gerados ou versionados com o WASM;
`strict` no `tsconfig` do consumidor é o alvo.

### 3.5 Publicação em JSR com Deno 2.0

`packages/ts` é um **workspace Deno 2.0**. Cada pacote é publicado no **JSR** (`jsr.io`) com
`deno publish`; não há `node_modules`, bundler ou `tsc` no caminho de publicação. O
`wasm-bindgen` gera `mod.ts` + `.d.ts`; o build copia o `.wasm` para
`packages/ts/packages/core/wasm/`.

```text
packages/ts/
  deno.json                  # workspace + tasks
  packages/core/deno.json    # @rich-editor-auy/core
  packages/schema/deno.json  # @rich-editor-auy/schema
  packages/dom/deno.json     # @rich-editor-auy/dom (browser)
  packages/core/wasm/*.wasm  # artefato gerado por crates/rich-editor-auy-wasm
```

```jsonc
// packages/ts/deno.json
{
  "workspace": ["./packages/core", "./packages/schema", "./packages/dom"],
  "tasks": {
    "check": "deno fmt --check && deno lint && deno check . && deno test",
    "publish:dry": "deno publish --dry-run --allow-dirty"
  }
}
```

```jsonc
// packages/ts/packages/core/deno.json
{
  "name": "@rich-editor-auy/core",
  "version": "0.1.0",
  "exports": { ".": "./mod.ts" },
  "publish": {
    "include": ["mod.ts", "wasm/*.wasm", "LICENSE"],
    "exclude": ["**/*_test.ts"]
  }
}
```

- `@rich-editor-auy/core` não importa DOM; `@rich-editor-auy/dom` é browser-only;
  `@rich-editor-auy/schema` é um serializador de presets, não um validador.
- `init()` é idempotente e o loader usa `fetch(new URL("./wasm/…wasm", import.meta.url))`, o que
  funciona em Deno 2.0 e no browser sem `node_modules`.
- `deno task check` e `deno publish --dry-run` são gates; a publicação real só ocorre em tag com
  token. A versão TS acompanha a versão do crate e do pacote Composer.
- O pacote JSR não contém `.css`; o gate de zero estilo vale para o artefato publicado, não só para
  o fonte.

---

## 4. PHP: pacote fino e honesto

```php
use RichEditorAuy\Document;
use RichEditorAuy\Validator;

$doc = Document::fromBytes($request->json);       // preserva os bytes
$doc->envelope()->requireCompatible($schemaFingerprint);

$report = (new Validator('/usr/local/bin/rich-editor-auy'))
    ->withSchema($schemaPath)
    ->validate($doc->bytes());                    // opcional, em CI ou em request sensível
```

| API | Garante | Não garante |
|---|---|---|
| `Document::fromBytes` | JSON bem formado, envelope reconhecido, bytes preservados | schema, normalização, steps |
| `envelope()` | `format`, `version`, `schema_fingerprint` | compatibilidade semântica do schema |
| `Validator` | relatório do CLI (violações, offsets) | execução do editor |
| `Renderer` (opcional) | HTML com política explícita de sanitização | fidelidade visual ou CSS |

`Document::json()` existe para inspeção; **não** use para resserializar e gravar. Se o consumidor
precisa de bytes canônicos, ele guarda `bytes()`; se precisa mudar o documento, usa o cliente ou o
CLI. O pacote não tem dependência de runtime e não abre rede.

### 4.1 Publicação Composer

`packages/php` é o pacote oficial `rich-editor-auy/php`:

```text
packages/php/
  composer.json       # type library; php >=8.3; ext-json; sem require de runtime
  src/                # PSR-4 RichEditorAuy\
  tests/              # PHPUnit
  phpstan.neon        # level max
  phpunit.xml
```

- `composer validate --strict` e `composer audit` são gates; PHPUnit e PHPStan rodam em matriz
  PHP 8.3/8.4/8.5.
- `Document::fromBytes()` preserva os bytes originais; `Envelope` valida `format`, `version` e
  `schema_fingerprint` de 64 hex (BLAKE3); `Validator` chama o CLI e devolve o relatório; `Renderer`
  é opt-in e recebe `SanitizePolicy` explícita.
- O pacote não reimplementa schema, `Step`, `Mapping` ou normalização e não depende de extensão
  além de `ext-json`.
- Publicação é por tag: Composer/Packagist consome o repositório; não há passo de build PHP no
  pacote.

---

## 5. O contrato de zero estilo

### 5.1 Regras duras

1. **Nenhum CSS**: sem arquivo `.css`, sem `<style>`, sem `insertRule`, sem `style=`, sem
   `box-sizing`, sem reset global, sem `all: unset`.
2. **Nenhuma classe**: `class` é gancho global de estilo; o adaptador usa `data-auy-*` estáveis.
3. **Nenhum wrapper**: o host é a raiz; `mount` não reparenta o host nem cria um `div` externo.
4. **Nenhuma suposição visual**: fonte, cor, espaçamento, largura, altura, `outline`, `direction`
   visual e `line-height` são do consumidor.
5. **Mutações documentadas; sem opt-out de a11y**: `contenteditable`, `role`, `aria-*`,
   `data-auy-*`, seleção/foco. O consumidor pode assumir foco/teclado/live region, mas não desligar
   a árvore semântica ou o mapeamento de roles.
6. **Render por dados**: o default é dado + DOM **semântico**; `semantics: "neutral"` é opt-in
   explícito. `innerHTML` de documento não confiável é proibido. Sanitização é do consumidor ou de
   um crate opcional, nunca implícita.
7. **Removível**: `destroy()` restaura o host; nada de listeners ou atributos órfãos.
8. **Teste de estilo**: snapshot de DOM, grep de `style=`/`class=`/`<style`/`insertRule`, pacote sem
   CSS e regressão visual com o CSS do consumidor.

### 5.2 A nuance honesta

"Zero estilo" significa **zero estilo autoral da lib**, não "zero default do navegador". Qualquer
elemento tem default do user agent. O renderer default usa **semântica nativa** (`<h1>`, `<ul>`,
`<a>`, `<img alt>`) porque ela é mais confiável para leitores de tela que ARIA em `div`/`span`; os
defaults visuais do navegador são documentados e o consumidor os controla com o CSS dele. Quem
precisa de zero default visual usa `semantics: "neutral"` e assume o custo de testar ARIA/leitor;
quem quer controle absoluto usa `data` e renderiza sozinho — nos dois casos, a responsabilidade de
a11y é explicitamente transferida, nunca fingida. O adaptador nunca remove `outline` no foco: foco
visível é responsabilidade do CSS do consumidor, e removê-lo seria uma regressão de acessibilidade.

A garantia é **de mão única**: a lib não injeta estilo, mas o CSS global do consumidor ainda alcança
o host. Se o consumidor precisa de isolamento, monte em um `ShadowRoot`; o adaptador pode oferecer
`shadow: true` como opção, não como default, porque shadow DOM afeta seleção, foco, ARIA e eventos
e precisa de teste próprio. "Não interferir" não é "imunizar contra o CSS da página".

### 5.3 O que o adaptador pode tocar

| Permitido | Por quê | Como desligar |
|---|---|---|
| `contenteditable` no host | é o mecanismo de edição | `mount({ editable: false })` |
| `role="textbox"`, `aria-multiline` | semântica de campo derivada da `A11yTree` | **não desligável**; não existe `aria: false` |
| `role`/`aria-level` dos blocos | ponte entre árvore e DOM | **não desligável**; `semantics: "neutral"` só troca o elemento, não remove semântica |
| `data-auy-pos-*` | mapa DOM ↔ posição | `mount({ positions: false })` (modo `data`) |
| seleção/foco | input e teclado | `mount({ manageSelection: false })` só se o host assumir |
| `aria-live` | anúncios de IME/remote | `mount({ announce: "host" })` quando o host já tem live region; nunca silencia conteúdo |

Nada de `class`, `style`, `id` global, `tabindex` em cada bloco, `aria-hidden` em conteúdo
focável, ou `role="application"` por padrão.

---

## 6. Dados profundos: identidade e referências

### 6.1 Identidade estável

Referência entre publicações só funciona se o alvo tiver identidade estável. Índice de array muda
com split/join; hash de conteúdo muda a cada edição; âncora de DOM é frágil. A identidade é um ID
opaco, atribuído pelo host:

```rust
pub struct DocId(/* string opaca, 1..=128 bytes ASCII */);
pub struct NodeId(/* string opaca, 1..=128 bytes ASCII */);
pub struct VersionTag(/* string opaca, 0..=64 bytes ASCII */);
```

- `NodeId` é **opcional** por nó de bloco (átomos inline são endereçados pelo bloco pai + range).
- O `id` é um **atributo reservado** (`AttrId::NODE_ID`) alterado pelo step `SetNodeAttr`; o helper
  `tx.set_node_id(...)` emite esse step. O core valida forma, tamanho e **unicidade dentro do
  documento** (`DuplicateId`), mas não gera IDs (sem RNG) e não conhece o corpus.
- `split`: o bloco original mantém o ID; o novo recebe `new_id: Option<NodeId>` do host. Sem
  `new_id`, o novo bloco não é âncora.
- `join`: política explícita. Default `KeepFirst`; o ID do segundo é descartado e devolvido em
  `JoinReport { dropped_ids }` para o host reatribuir. `set_node_id` permite restaurar.
- `DocId` é estável por publicação e **não faz parte do `Doc` canônico**: é metadado de publicação
  no envelope/host. O envelope pode carregar `doc_id` opcional para persistência/corpus; ele não
  entra no hash do `Doc`. `(DocId, NodeId)` é a chave global; o mesmo `NodeId` em dois documentos é
  permitido, o par não é.
- Steps que removem blocos carregam o fragmento removido no inverso; por isso undo restaura o `id`.
  Mudança explícita de `id` é operação de publicação, não edição comum; se o inverso colidir com
  outro nó, o core devolve `DuplicateId` e o host resolve (não há "ID mágico" no undo).

### 6.2 Alvo de referência

```rust
pub struct RefTarget {
    pub doc: DocId,
    pub node: Option<NodeId>,
    pub version: Option<VersionTag>,
    pub fragment: Option<Range>, // sub-range opcional dentro do nó
}
```

`RefTarget` é o **único objeto aninhado** permitido no wire (chaves fixas, ordem canônica), exposto
como `AttrValue::Ref` e usado pela marca `ref` ou pelo átomo `citation`. O core:

- valida a **forma** (IDs, tamanhos) e o range de origem; o range do alvo é opaco e fica para o
  `refs`;
- serializa canonicamente e preserva o alvo através de edições;
- mapeia o range de origem;
- **não** resolve se o alvo existe, não abre rede, não acessa banco, não reescreve referências;
- expõe `doc.references() -> Vec<Reference>` e `doc.anchors() -> Vec<Anchor>`.

`Reference { source: Range, target: RefTarget }`; `Anchor { id: NodeId, range: Range }`. É o dado
que a UI precisa para desenhar link, tooltip, backlink e relatório de dangling.

### 6.3 Camada `refs` (opcional, fora do core)

`rich-editor-auy-refs` é um índice de corpus, não um banco:

```rust
pub struct Corpus { /* BTreeMap<DocId, DocEntry> */ }

impl Corpus {
    pub fn upsert(&mut self, id: &DocId, doc: &Doc) -> Result<(), RefError>;
    pub fn remove(&mut self, id: &DocId);
    pub fn resolve(&self, target: &RefTarget) -> Resolution;
    pub fn backlinks(&self, doc: &DocId, node: &NodeId) -> Vec<Reference>;
    pub fn dangling(&self) -> Vec<Reference>;
    pub fn version_mismatches(&self) -> Vec<VersionMismatch>;
}
```

- `Resolution::{Found, MissingDoc, MissingNode, MissingFragment, VersionMismatch}`.
- `CorpusStore` é um trait de persistência; a implementação é do consumidor (SQL, arquivo, KV).
- Atualização incremental a partir de `ChangeSet` é otimização; `upsert(id, doc)` é o baseline correto.
- Sem HTTP, sem checagem de link externo, sem auth, sem publicação. Link externo é `link` mark e
  política do consumidor.
- Transclusão/embed é um nó com resolver próprio, pós-MVP; não é referência e não entra no core.

### 6.4 O fluxo que fica fácil

1. Publicação B tem `DocId("post-42")` e a seção 3 tem `NodeId("sec-3")`.
2. A autora insere um `ref` em A apontando para `{doc: "post-42", node: "sec-3", version: "v7"}`.
3. O JSON de A guarda o alvo; nenhuma URL, nenhum `href` frágil.
4. `refs` resolve para B/sec-3; a UI desenha o link; B vê backlinks de A.
5. B move a seção: o ID sobrevive; o link continua válido.
6. B apaga a seção: `MissingNode`; a UI mostra dangling e a autora decide.
7. B publica v8: `VersionMismatch` se a referência fixou v7; a política é do consumidor.

O que continua sendo trabalho do consumidor: atribuir IDs, persistir o corpus, decidir política de
dangling/versão e renderizar o link. O que a lib entrega é o **dado estável** e a **resolução
determinística** — a parte que não deveria ser reinventada em cada projeto.

---

## 7. Fronteiras: o que é da lib, o que é opcional, o que é do consumidor

| Preocupação | Core | Pacote opcional | Consumidor |
|---|---|---|---|
| Modelo, schema, posições, steps, histórico | **sim** | — | — |
| JSON canônico, envelope, fingerprint | **sim** | — | — |
| IDs estáveis e extração de referências | **sim** | — | atribui IDs e persiste |
| Resolução de referências/backlinks | não | `refs` | armazena corpus, política |
| Renderização, layout, pintura | não | renderer semântico | CSS, tema, geometria |
| Input/IME/clipboard/seleção | não | adaptador DOM | integração com framework |
| A11y semântica (`A11yTree`, nomes, `alt`, `lang`/`dir`, ordem) | **sim** | — | — |
| ARIA renderizada/foco/teclado/live regions | não | adaptador | auditoria final, contraste |
| Persistência, auth, rotas, publicação | não | — | **sim** |
| HTML/Markdown | não | `html`/`markdown` | política de sanitização |
| Colaboração (OT/CRDT/transporte) | steps/mapping | `collab` | servidor, presença, auth |
| Sanitização de HTML | não | sanitizer explícito | **sempre** |
| i18n estrutural (Unicode, `lang`/`dir`, bidi lógico, ordem) | **sim** | — | — |
| Tradução editorial, spellcheck, strings de UI, PDF | não | — | **sim** |

**Nunca no core:** CSS, componentes visuais (toolbar, bubble menu), CMS, banco, auth, rede,
sanitizador por default, servidor de colaboração. Se algum entrar, o projeto vira framework e perde
a garantia de "não interfere".

**Pode ser incorporado como pacote separado:** WASM/TS, adaptador DOM, SDK PHP, `refs`, codecs
HTML/Markdown, CLI, wrappers de framework, presets de schema, testkit de a11y do adaptador. Cada um
com seu dono e seu gate; o core não os importa. Auditoria WCAG/AT do host não é pacote do core: é
responsabilidade do consumidor.

**Nunca opcional:** a árvore semântica e a validação de a11y no core. O adaptador pode ser separado,
mas uma UI sem mapeamento da árvore não é uma integração suportada.

---

## 8. Acessibilidade

### 8.1 Três camadas, uma responsabilidade cada

| Camada | Garante | Não garante |
|---|---|---|
| **Core** | `A11yTree` determinística: roles, nomes, `alt`, `lang`, `dir`, níveis, ordem de leitura, ranges, IDs, `validate_a11y` | ARIA renderizada, foco, contraste, leitor de tela |
| **Adaptador** | mapeia a árvore para elementos nativos/ARIA; `role="textbox"`, `aria-multiline`, `aria-readonly`/`aria-disabled`, `aria-invalid`, `aria-labelledby`/`describedby`, teclado, foco, seleção, anúncios de IME/remote | inventar semântica, CSS, contraste, auditoria final, todos os leitores |
| **Consumidor** | contraste, foco visível, alvo de toque, zoom/reflow, rótulos, auditoria WCAG, testes com leitores | semântica do documento |

O core **não promete WCAG sozinho**, mas **promete a árvore e a validação** que tornam a
conformidade possível; o gate final de contraste/reflow/AT é do consumidor. A a11y semântica não é
opcional: sem árvore, o adaptador não tem o que mapear.

### 8.2 `contenteditable` é o caminho recomendado

O adaptador usa `contenteditable` no host e deixa a seleção nativa exposta ao leitor de tela. Isso
é menos glamouroso que um canvas, mas é o caminho com melhor suporte real. O renderer:

- coloca `role="textbox"` e `aria-multiline="true"` no host;
- usa `aria-labelledby`/`aria-describedby` fornecidos pelo consumidor;
- expõe blocos com elementos nativos no `mount` default e com `role`/`aria-level` derivados da
  árvore no modo `neutral`;
- não usa `role="application"` por default (só se o consumidor implementar todo o teclado);
- não coloca `aria-hidden` em conteúdo focável;
- anuncia composição de IME, mudanças remotas e erros via `aria-live`, sem anunciar cada tecla;
- mantém o foco visível; nunca aplica `outline: none`.

O default é **semântica nativa** para priorizar acessibilidade; `semantics: "neutral"` é opt-in para
quem precisa de zero default visual e assume o custo de ARIA em `div`/`span`. Nenhum dos dois modos
promete WCAG; o modo `neutral` exige testes de leitor de tela mais rigorosos. Em ambos, o
mapeamento vem da `A11yTree` e não pode ser desligado.

### 8.3 Canvas e renderização custom

Um canvas não é acessível por si. Se o consumidor escolher canvas, ele precisa fornecer uma
**árvore DOM espelhada** ou uma live region com o texto e a seleção; o adaptador não resolve isso.
Isso é modo avançado, não o caminho fácil, e deve ser declarado como limitação.

### 8.4 O que testar

- `A11yTree` + `validate_a11y`: goldens e casos por código no CI do core.
- `axe-core` nos exemplos do adaptador, em CI, **depois** do teste da árvore.
- Teclado: setas, Home/End, Ctrl/Cmd+seta, Shift+seleção, PageUp/Down, Tab sai do editor, Escape.
- IME: composição, cancelamento, commit, undo.
- Leitores: NVDA/Firefox, VoiceOver/Safari, JAWS/Chrome; matriz manual, não "passou no axe".
- Zoom 200%/400%, reflow, RTL/bidi, alto contraste.
- Foco visível com CSS do consumidor; nada de `outline: none` do adaptador.

**Risco assumido:** ARIA-only em rich text é fácil de errar. Por isso o default é semântica nativa;
`semantics: "neutral"` é opt-in e precisa de testes reais de leitor de tela, não só de árvore ARIA.
Se o custo do modo `neutral` crescer, ele sai — a semântica nativa e a `A11yTree` não.

---

## 9. Até onde o projeto pode ir

| Nível | Entrega | Depende de | É o quê |
|---|---|---|---|
| **L0** | core + JSON canônico | F1–F4 | engine |
| **L1** | WASM + JSR/Deno 2.0 + Composer PHP | L0 | bindings oficiais |
| **L1.5** | adaptador DOM + mapeamento da `A11yTree` | L1 | editor utilizável e acessível |
| **L2** | `refs` + CLI + SSR opcional | L1 | publicação e referências |
| **L3** | wrappers React/Vue/Svelte, plugins CMS | L1.5 | ecossistema/comunidade |

O projeto pode ir longe como **camada de dados de um ecossistema de edição e publicação**: docs,
bases de conhecimento, CMS, anotações, citações, backlinks. Não pode ir longe como **produto
visual**: toolbar, tema, layout, menu e CMS são outro produto. A linha de parada é clara: no dia em
que o core ou o adaptador precisar de uma folha de estilo para funcionar, a promessa de "não
interfere" acabou.

---

## 10. Impacto no roadmap

| Fase | Mudança |
|---|---|
| F1 | `DocId`/`NodeId`/`VersionTag` e o atributo reservado `id` (`set_node_id`) entram no modelo e no wire; IDs opcionais, sem geração |
| F2 | `AttrValue::Ref`, marca `ref`/átomo `citation`, `references()`/`anchors()`, política de split/join |
| F3 | comandos `set_ref`/`remove_ref`, `set_node_id`, input rules de referência (opcional) |
| **F3.5** | **novo**: crate WASM + workspace Deno 2.0/JSR (`core`/`schema`/`dom`) + pacote Composer PHP + CLI mínimo + gate de zero estilo, publicação e a11y (MVP) |
| F4 | performance do core; overhead do adaptador medido à parte |
| F5 | `refs`, HTML/Markdown, CLI `render`, SSR com política |
| F6 | colaboração (inalterado) |

### 10.1 Aceite de F3.5

- Quick start TS/Deno em ≤ 20 linhas, sem import de CSS, com host do consumidor.
- `mount` não cria wrapper, não usa `class`/`style`, não remove `outline`; `destroy()` restaura.
- Snapshot de DOM prova ausência de `<style>`, `style=`, `class=` e `insertRule`.
- `deno task check` e `deno publish --dry-run` verdes; pacote JSR sem `.css` e sem export de estilo.
- `wasm32-unknown-unknown` compila com `--locked`; artefato WASM reprodutível e medido.
- PHP: `fromBytes($bytes)->bytes() === $bytes` byte a byte; envelope incompatível falha tipado;
  `composer validate --strict`, `composer audit`, PHPUnit e PHPStan verdes.
- CLI valida um documento inválido com `path`/`offset`, sem panic.
- IDs/`RefTarget` round-trip e `references()`/`anchors()` com corpus mínimo no testkit; o crate
  `refs` completo é F5.
- `axe-core` verde nos exemplos; teclado e IME cobertos por testes de navegador.
- `Doc::a11y_tree` + `validate_a11y` verdes no corpus; roles/nomes/`alt`/`lang`/`dir` conferidos.

### 10.2 Orçamento adicional (estimativa)

| Peça | LOC |
|---|---:|
| `crates/rich-editor-auy-wasm` + tipos TS | ~800 |
| `packages/ts/packages/dom` (render semântico, mapeamento `A11yTree`, input) | ~1.500 |
| `packages/php` | ~400 |
| `refs` | ~1.200 |
| testes de integração/a11y/estilo | ~1.000 |
| **total adicional MVP** | **~3.700** |
| **total adicional pós (refs)** | **~1.200** |

O MVP sobe de ~19,0k para **~22,7k LOC**; com `refs` pós-MVP, ~23,9k; com HTML/Markdown e
colaboração, ~32,7k. Se o adaptador DOM passar de ~2k LOC sem entregar o aceite, o escopo está
errado: corte `neutral`/extras e preserve `mount` semântico + `A11yTree`; `data`/`bind` só
transferem a responsabilidade explicitamente.

---

## 11. Decisões relacionadas

As linhas completas estão em [`02_decisoes.md`](02_decisoes.md) (D51–D62) e as fechadas em
[`03_decisoes-fechadas.md`](03_decisoes-fechadas.md) (F26–F29). Em resumo:

| ID | Decisão | Recomendação |
|---|---|---|
| D51 | Contrato PHP/TS | JSON canônico é a ABI; WASM para TS; PHP fino; sem schema em SDK |
| D52 | Escopo do adaptador | `data`/`bind`/`mount`; `mount` semântico default; `neutral` opt-in; zero CSS |
| D53 | IDs estáveis | `DocId`/`NodeId` opcionais, host-assigned, unicidade no doc |
| D54 | Referência | `RefTarget` tipado, `ref`/`citation`, core valida forma e não resolve |
| D55 | Split/join de IDs | original mantém; novo recebe `Option`; join `KeepFirst` + relatório |
| D56 | `refs` | corpus em memória + trait de persistência; sem rede/DB |
| D57 | Render/SSR no PHP | CLI com política explícita; nunca default (envelope/bytes são D51) |
| D58 | A11y | invariante de core: `A11yTree` + validação; adaptador mapeia; host audita |
| D59 | Presets de schema | `basic`/`document` em TS/PHP emitem JSON; Rust valida |
| D60 | Wrappers de framework | comunidade/opcional; não no core |
| D61 | Árvore semântica | `A11yTree`/`A11yRole` derivada, determinística, testada; sem DOM |
| D62 | Validação de a11y | `validate_a11y` com códigos/severidade/ranges no `make check` |

---

## 12. Checklist de integração do consumidor

- [ ] O host é meu; nenhum CSS da lib foi importado.
- [ ] Escolhi `data`, `bind` ou `mount` e sei quem renderiza.
- [ ] Se usei `data`/`bind`, assumi explicitamente a a11y renderizada; se usei `mount`, o
      mapeamento da `A11yTree` está ativo e testado.
- [ ] Rodei `validate_a11y` e tratei `A11yReport`; não há `alt`/nome obrigatório vazio.
- [ ] Defini schema/preset e tratei `SchemaError`.
- [ ] Atribuí `DocId`/`NodeId` para o que precisa ser referenciável.
- [ ] Persisto `Envelope.bytes()`, não JSON resserializado.
- [ ] Resolvo referências no meu corpus (`refs` ou equivalente) e tenho política de dangling.
- [ ] Forneci `aria-labelledby`/`aria-describedby`, foco visível e contraste.
- [ ] Testei teclado, IME, zoom e leitor de tela.
- [ ] Não assumo que HTML importado é seguro; sanitizo com política explícita.
- [ ] Se uso canvas, forneci espelho DOM/live region; canvas sozinho não é acessível.
