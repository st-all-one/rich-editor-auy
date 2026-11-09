# Comparativo de desempenho — documento complexo

## 1. Escopo e método

Comparativo **headless de engine**, não de browser completo (DOM, IME, paint, leitor de tela).
Cada ferramenta foi levada ao limite do que seu modelo representa sem plugins/custom nodes, no
mesmo host.

Ambiente: Linux x86_64, Rust 1.99 `--release`, Node 22.23, ProseMirror 1.25.13, Lexical 0.53.0,
Quill 2.0.3, Tiptap 3.33.0. Foram feitas 2–5 execuções de aquecimento e 10–100 medições por
operação. Números de uma execução; não são gate de CI.

### Corpus

`bench/examples/all_types.rs` gera 500 repetições de um template com 4.000 blocos e 817.979 bytes
de JSON:

- documento com `lang`, `dir` e `title`;
- IDs de nó, títulos e parágrafos;
- runs com `strong`, `em`, `code`, `link` e `ref` entre publicações;
- átomo inline `image` com `alt` e `src`;
- listas, `blockquote`, `code_block`, tabela (`table`/`row`/`header`/`cell`) e separador;
- roles e validação a11y.

O corpus **comum sem refs** (`/tmp/complex-taino.json`) tem 3.000 blocos e 738.570 bytes e serve
para comparar AUY, ProseMirror e taino no mesmo JSON. Quill, Tiptap e Lexical usam subconjuntos
próprios, indicados na tabela.

## 2. Resultado por ferramenta

### 2.1 AUY — todos os tipos, 4.000 blocos, 818 KB

| Operação | Tempo |
|---|---:|
| `check_json` (parse bruto) | 11,30 ms |
| `from_json` (parse + decode + validação) | 26,98 ms |
| `to_json_string` | **0,65 ms** |
| `validate` (estrutura) | 4,44 ms |
| `a11y_tree` | 10,10 ms |
| `validate_a11y` | 10,92 ms |
| `apply` + 1 char | **0,003 ms** |

A AUY é a única da tabela que representa todos os tipos no mesmo schema e faz validação
estrutural + a11y no core. `apply` inclui `ChangeSet` e histórico.

### 2.2 ProseMirror — todos os tipos, 4.000 blocos, 885 KB

| Operação | Tempo |
|---|---:|
| `Node.fromJSON` | **4,43 ms** |
| `toJSON` + `JSON.stringify` | 8,85 ms |
| `state.apply(insertText)` | 0,044 ms |

PM não tem a11y, IDs únicos, refs entre documentos ou validação semântica equivalente no core.
O parse é ~6× mais rápido que o `from_json` da AUY e o `apply` ~15× mais lento.

### 2.3 Corpus comum sem refs — 3.000 blocos, 739 KB

| Ferramenta | Parse / setContent | Serialize / get | Aplicar 1 char |
|---|---:|---:|---:|
| AUY | 20,39 ms | **0,759 ms** | **0,003 ms** |
| ProseMirror | **3,15 ms** | 5,16 ms | 0,038 ms |
| taino-edit | 9,51 ms | 18,22 ms | 0,038 ms |
| Lexical* | 54,43 ms | 15,10 ms | — |
| Quill* | 1.747,5 ms (setContents/DOM) | 2,75 ms (getContents) | 283,4 ms (updateContents/DOM) |
| Tiptap* | 12,67 ms (setContent/DOM) | 3,10 ms (getJSON, objeto) | 15,80 ms (insertContentAt/DOM) |

\* Corpus/subconjunto diferente: Lexical 2.500 nós/1,30 MB sem refs/tabela/imagem; Quill Delta
plano com 12.000 ops/77,5 KB sem tabela/refs/alt; Tiptap 3.000 blocos/646 KB sem refs/tabela/imagem.
Quill/Tiptap incluem construção de DOM; AUY/PM/taino/Lexical são headless. `getJSON` do Tiptap devolve
objeto, não string.

## 3. Limites de cada ferramenta

| Ferramenta | O que suporta no teste | Onde o modelo quebra |
|---|---|---|
| **AUY** | doc attrs, IDs, refs, tabela, imagem, separador, a11y | parse/validação caros; a11y custa ~11 ms |
| **ProseMirror** | árvore + marks + tabela/imagem via schema | a11y/IDs/refs/validação semântica exigem schema/plugins; parse mais rápido por validar menos |
| **taino-edit** | tabela, imagem, blockquote, listas, code, marks | sem ref mark no conjunto de extensões; sem a11y de core; JSON via `serde_json` é lento |
| **Lexical** | heading/parágrafo/lista/link/marks | refs, tabela e imagem exigem nós customizados; formato maior; sem a11y de core |
| **Quill** | texto rico plano, listas por atributo, link, image embed | Delta não expressa árvore aninhada/refs/IDs; image embed não carrega `alt`; sem schema/a11y; DOM obrigatório |
| **Tiptap** | PM + StarterKit (heading, listas, code, blockquote, link) | tabela/imagem/refs/a11y exigem extensões; DOM obrigatório; parse/set inclui render |

## 4. Leitura dos números

### Parse

O ProseMirror ganha no corpus completo (4,43 ms contra 26,98 ms da AUY). Parte da diferença é
arquitetural — `Node.fromJSON` constrói a árvore diretamente, enquanto a AUY materializa
`JsonValue`, decodifica, cria `Doc` e valida. Parte é escopo: PM não valida a11y, IDs únicos,
refs entre documentos e os mesmos invariantes. O parse bruto da AUY (11,30 ms) ainda é 2,5× o PM;
o próximo ganho grande é decode incremental sem árvore JSON intermediária.

O taino parseia o corpus sem refs em 9,51 ms, ~2,1× mais rápido que a AUY no mesmo JSON. Ele usa
uma árvore/`serde_json` própria e não faz a validação a11y da AUY.

### Serialização

A AUY ganha com folga: writer streaming, capacidade pré-calculada e cópia de trechos sem escape.
Ela escreve 0,65 ms no corpus completo, contra 8,85 ms do PM e 18,22 ms do taino. O Delta do Quill
serializa em 2,75 ms, mas é uma representação plana e perde estrutura; o `getJSON` do Tiptap em
3,10 ms devolve objeto, não bytes canônicos.

### Edição

A AUY aplica 1 char em ~3 µs no corpus misto; PM ~38–44 µs, taino ~38 µs. O PM e o taino constroem
novos nós/objetos; a AUY copia o chunk de 128 blocos e o diretório, mantendo o custo independente
do tamanho total do documento. Quill (283 ms) e Tiptap (15,8 ms) pagam o DOM e a reconciliação da
view; esses números não são comparáveis a um core headless.

### A11y

A AUY é a única com árvore e validação a11y no core: ~10,1 ms e ~10,9 ms no corpus completo. PM,
Lexical, Quill e Tiptap delegam acessibilidade ao app/DOM, com cobertura e custo dependentes do
host. Isso é vantagem funcional e custo explícito; deve ser lazy/opt-in no adaptador.

### Lexical

Lexical foi o mais lento no parse do subconjunto: 54,4 ms para 2.500 nós/1,30 MB, contra 46,6 ms
na rodada anterior e 33 µs/KB da AUY no corpus completo. O formato carrega muitos metadados por nó
(`detail`, `format`, `mode`, `style`, `version`, `direction`, `indent`) e `parseEditorState`
reconcilia estado de editor, não apenas uma árvore. A vantagem é o modelo de nós/extensões e a
integração React, não throughput de wire.

### Quill e Tiptap

Quill é o caso extremo do modelo Delta: `setContents` de 3.000 blocos levou 1,75 s no happy-dom e
`updateContents` 283 ms. O `getContents` é rápido (2,75 ms), mas a representação não tem árvore
semântica, refs, tabela ou `alt`. Tiptap ficou em 12,7 ms de `setContent`, 3,1 ms de `getJSON` e
15,8 ms de inserção com DOM. Ele é PM + extensões + view; os números sem DOM do PM são o piso, não
o Tiptap completo.

## 5. Veredito frente à AUY

| Dimensão | Vencedor | Comentário |
|---|---|---|
| Parse canônico | ProseMirror | ~6× mais rápido no corpus completo, mas valida menos |
| Parse com validação completa | AUY | único com a11y/IDs/refs no core |
| Serialização | **AUY** | ~13,6× mais rápida que PM no corpus completo |
| Edição | **AUY** | ~3 µs vs 38–44 µs; chunks valem |
| Modelo completo | **AUY** | refs + tabela + imagem + a11y no mesmo schema |
| A11y | **AUY** | PM/Lexical/Quill/Tiptap delegam |
| Extensibilidade/React | Lexical/Tiptap | ecossistema JS maduro |
| DOM/browser | Quill/Tiptap | medidos com happy-dom, não browser real |

A AUY ganha em **serialização, edição e completude semântica**. Perde em **parse** para PM e
taino, e paga **a11y** no core. O próximo experimento correto é decode incremental + benchmark de
browser/IME separado; estes números não medem paint, teclado ou leitor de tela.

## 6. Reprodução

```sh
cargo run --release --manifest-path bench/Cargo.toml --example all_types
cargo run --release --manifest-path bench/Cargo.toml --example complex -- /tmp/complex-taino.json

cd bench/reference
npm install
node make_basic.mjs
node pm-basic.mjs /tmp/complex.json
node pm-all.mjs
node lexical.mjs
N=500 node quill.mjs
N=500 node tiptap.mjs
(cd taino && cargo run --release)
```

Arquivos: `bench/examples/all_types.rs`, `bench/examples/complex.rs`, `bench/reference/`.

## 7. Estado persistente vs JSON por operação

O harness `bench/examples/stateful.rs` compara o caminho legado (parse + apply + serialize a cada
tecla) com o `Editor` stateful introduzido no plano 11. Documento: 1.000 parágrafos, 100 bytes
cada, 159 KB de JSON canônico, schema completo.

| Cenário | ms/op | Ops |
|---|---:|---:|
| Legado: `from_json` + `apply` + `to_json_string` | 2,120 | 200 |
| Stateful: `apply` sem JSON | 0,0306 | 2.000 |
| Stateful: serializar uma vez ao final | 0,114 | 100 |
| Importar documento (`Editor::from_json`) | 2,093 | 200 |
| Importar fragmento no início | 2,411 | 200 |

Ganho: **~69×** por edição sem JSON e **~18,7×** no round-trip completo. Importação continua sendo
o custo dominado por parse + validação, como esperado; ela é uma operação de lote, não de tecla.

Reprodução:

```sh
cargo run --release --manifest-path bench/Cargo.toml --example stateful
```
