# Resolver divergências abertas (A–M) de DIVERGENCES.md

## Contexto

`DIVERGENCES.md` tem 13 linhas abertas (A–M). O objetivo é fechar todas com evidência executada, não
com aceite informal: desempenho de wire/a11y nos alvos do `plan/05`, hot path com orçamento de
alocações calibrado e justificado por site, `schema!` com a11y declarativa/acessores tipados/
`trybuild`, limites checados antes da materialização, `iai-callgrind` no CI, fuzz executado, PHP
executado em container, pacote WASM verificável localmente, `wasm-opt` e `axe-core` como gates, e o
adaptador DOM completo (seleção, `beforeinput`, IME agrupado, clipboard rico via `Slice` JSON,
opções `positions`/`semantics`/`manageSelection`/`announce`). As decisões já tomadas: ponte completa
no adaptador, wire/a11y nos alvos com hot path calibrado, gates reais via Docker/CI + npm, crate WASM
publicável com gate local `--no-verify`.

Cada passo abaixo é ordenado para a árvore compilar e a suíte existente passar ao final do passo.
Fases 1→4 medem; fase 5 é independente; fases 6→7 são sequenciais entre si; fases 8→10 dependem do
core/WASM compilando, mas não umas das outras.

## Abordagem

### Fase 1 — Hot path: clone barato e alocações medidas (B, F)

1. **`History` com grupos compartilhados** em `crates/rich-editor-auy/src/state/history.rs`:
   `done`/`undone` passam a `Vec<Arc<Group>>`. `record` empurra `Arc::new(group)`;
   `trim`/`info`/bytes iteram por `Arc<Group>`; `undo`/`redo` movem o `Arc` e clonam só o
   `Arc` para a pilha oposta (sem clonar `steps`/`inverse`); a fusão de grupos consecutivos
   (`record` com o mesmo `id`) usa `Arc::make_mut` no último e `Group::absorb`, documentando que
   só composição paga essa cópia. `EditorState::clone` deixa de clonar todos os `Group`s: fica
   O(grupos) em increments de `Arc` e uma alocação do `Vec` de ponteiros.
   - Não troque `Vec<Arc<Group>>` por `Arc<Vec<Group>>`: com o clone do estado vivo, o
     `Arc::make_mut` do `record` copiaria todos os grupos.
2. **`Editor::apply` mantém a atomicidade por clone** em `state/editor.rs`: continue com
   `let state = self.state.clone(); state.apply_owned(transaction)?;`. O clone barato elimina a
   cópia O(grupos) sem rollback. **Não** introduza `apply_in_place`/`Doc::empty`/rollback:
   `apply_step_owned` consome o `Doc` e, num erro, o documento parcialmente mutado seria perdido
   (o placeholder deixaria o estado corrompido). O `Arc<Doc>` compartilhado com o estado original
   é o que garante que o clone falhe o `Arc::get_mut` e copie só o bloco tocado (copy-on-write),
   preservando o contrato 2 (“transação atômica”).
3. **Teste de clone barato** em `crates/rich-editor-auy/tests/alloc.rs`:
   `state_clone_does_not_copy_history_groups`: crie um `EditorState` com 8 grupos (8
   `insert_text` encadeados via `apply_owned`), `reset()`, `let cloned = state.clone();`, asserte
   `allocations() <= 2` e `cloned.history_info().undo_depth == 8`. O teste falha no desenho
   antigo (cada `Group` clona `Vec<Step>` duas vezes).

4. **Fast path de inserção** em `crates/rich-editor-auy/src/step/text_edit.rs`: quando
   `start == end`, o offset cai dentro de um `Run` com `run.marks.is_empty()` e `insert` não é vazio,
   construa um único `String::with_capacity(run.text.len() + insert.len())` com prefixo + insert +
   sufixo, rode `TextBlock::new` sobre os itens (movendo os não tocados) e retorne antes do caminho
   genérico. Fora do caso (range não vazio, run com marcas, offset em átomo), preserve o código atual.

5. **`TextBlock::normalize` com early return** em `crates/rich-editor-auy/src/model/text.rs`: se todo
   `InlineItem::Text` tem `text.len() <= ceiling` e não é vazio, todo átomo size 1, e não há dois
   `InlineItem::Text` vizinhos com o mesmo `MarkSet`, chame `recompute_len()` e retorne `Ok(())` sem
   reconstruir `items`/`String`. Mantenha o caminho atual quando algum run excede o teto ou há
   fusão/descarte pendente.

6. **`TextBlock::new` com `len` incremental**: mantenha `len` somando `item.size()` em `push` e
   remova a chamada a `recompute_len()` por item (o `normalize` continua recalculando ao final).

7. **`ChangeSet::from_steps` com capacidade exata**: conte os ranges antes de construir o `Vec`
   (`steps` já é conhecido) e use `Vec::with_capacity(count)`; `into_boxed_slice` não deve realocar.
   Em `from_steps`, mantenha a ordem e o `sort_by_key` existentes.

8. **Testes**:
   - `crates/rich-editor-auy/tests/alloc.rs`: depois dos passos 4–7, meça e fixe o menor teto
     estável (`assert!(allocations() <= N, "hot-path insert allocated {}", allocations())`) com um
     comentário listando os sites restantes (String do texto, `Vec`/`Box` de `InlineItem`,
     `TextBlock`, `Arc<Block>`, `ChangeSet`, `Group`). Se `N` não cair abaixo do medido hoje (40),
     registre em `DIVERGENCES.md` o motivo por site; zero alocação exigiria arena/`unsafe`, que o
     core proíbe. Adicione `assert!(owned <= immutable)` no mesmo fixture.
   - Novo teste em `tests/editing.rs`:
     `failed_transaction_leaves_document_selection_and_version_unchanged`: `Editor` com 1 bloco,
     transação com `insert_text` válido seguido de `delete` fora dos limites; `apply` deve falhar,
     `editor.doc().to_json_string`/`selection`/`version` iguais aos de antes.
   - Novo teste `owned_apply_is_no_worse_than_immutable`: compare a contagem de alocações de
     `EditorState::apply_owned` com a de `EditorState::apply` no mesmo fixture e asserte
     `owned <= immutable`; em seguida aplique uma segunda transação ao estado resultante e verifique
     que o documento continua editável.
   - Novo teste `insert_text_applies_stored_marks` (fase 6, junto de stored marks): inserir com
     `add_mark` em range vazio e verificar o JSON do texto com `strong`.

### Fase 2 — Wire: limites cedo e menos passadas (C, H)

1. **Thread de limites**: altere as assinaturas internas em `crates/rich-editor-auy/src/wire/attrs.rs`:
   ```rust
   pub(crate) fn decode_attrs(value: Option<&JsonValue>, specs: &[AttrSpec], path: DecodePath, limits: &Limits) -> Result<Attrs, ParseError>
   pub(crate) fn decode_marks(value: Option<&JsonValue>, schema: &Schema, limits: &Limits) -> Result<MarkSet, ParseError>
   pub(crate) fn decode_value(value: &JsonValue, kind: &AttrType, limits: &Limits, depth: u8) -> Result<AttrValue, ParseError>
   ```
   Atualize as chamadas em `wire/parser.rs` (`decode_doc`, `decode_block`, `decode_inline`) e
   `wire/fragment.rs`.

2. **Checagens antes de materializar**:
   - Em `decode_attrs`, depois de obter os membros do objeto e antes de decodificar valores: rejeite
     `members.len() > usize::from(limits.max_attrs_per_node)` com `ParseError { code: "limit", ... }`
     no offset do objeto.
   - Em `decode_value`, antes de `Box::from`/`clone` de cada string e antes de coletar `List`/`Map`:
     rejeite `text.len() > limits.max_attr_string_bytes` e `depth > limits.max_depth` (`Any`/`List`/
     `Map` recursivos). Use `code: "limit"`.
   - Em `decode_marks`, antes do laço: rejeite `values.len() > u64::BITS` (`code: "mark_limit"`) e
     ids de marca repetidos (`ParseError::new("duplicate_mark", value.start())`); antes de
     `Attrs::new(decoded)`, rejeite `decoded.len() > usize::from(limits.max_mark_attrs)`
     (`code: "limit"`). Troque a sequência de `marks.with_mark(...)` por coleta em `Vec<Mark>` e uma
     única construção `MarkSet::from_marks(marks)` (novo `pub(crate) fn from_marks(Vec<Mark>) -> MarkSet`
     em `model/marks.rs`, uma passada para máscara e attrs ordenados).
   - Em `decode_inline`, antes de `String::from(text)`: rejeite
     `text.len() > limits.max_block_text_bytes` (`code: "limit"`).
   - Em `decode_block` no ramo inline, acumule bytes de texto e rejeite quando exceder
     `max_block_text_bytes` antes de `TextBlock::new`.
   - Em `decode_blocks`/`decode_block`, passe um `&mut DecodeBudget { nodes: u32, text_bytes: u64 }`,
     incremente por bloco/átomo e rejeite `nodes > limits.max_nodes` antes de `Arc::new(block)`;
     some bytes de texto e rejeite `text_bytes > max_doc_bytes` (`code: "limit"`).
   - Corrija a chamada `text.normalize(options.limits.max_block_text_bytes)` para
     `min(u32::try_from(model::MAX_RUN_BYTES).unwrap_or(4096), options.limits.max_block_text_bytes)`.

3. **`Doc::from_decoded`**: em `model/doc.rs`, adicione
   `pub(crate) fn from_decoded(attrs: Attrs, blocks: Vec<Arc<Block>>) -> Self` que apenas monta
   `DocInner { attrs, chunks: Chunks::new(blocks) }` (sem varredura de limites). Use em
   `decode_doc`/`from_fragment_json` **somente depois** das checagens do passo 2. Mantenha `Doc::new`
   como construtor público checado.

4. **Remover duplicação de conteúdo no decode**: em `decode_block`, decida inline vs container apenas
   por `schema.node(child_kind).is_some_and(|spec| spec.inline)`; remova as chamadas
   `schema.content_allows(...)` do decode. A rejeição de conteúdo inválido passa a ser de
   `Doc::validate` (`I2`), como antes; ajuste só se algum teste depender do código antigo (os testes
   atuais só verificam `is_err`).

5. **Reduzir alocações de `Doc::validate`** em `model/validate.rs`: troque
   `previous = Some(run.marks.clone())` por comparação com o run anterior via índice/`&MarkSet`;
   mantenha `kind_names`/`kinds` (a máquina de conteúdo precisa da lista).

6. **Testes**: estenda `crates/rich-editor-auy/tests/parser_limits.rs` com limites minúsculos e
   asserções de erro para `max_attrs_per_node`, `max_mark_attrs`, `max_nodes`, `max_depth` em attrs
   aninhados e `max_attr_string_bytes` em `List/Map`; adicione um caso de attr string gigante que
   falha sem construir `AttrValue` (asserção: `is_err()` e código `"limit"`). Estenda
   `tests/parser_validation.rs` com round-trip de attr `List`/`Map` e um run >4096 bytes rejeitado
   com `I4`.

7. **Meta e contingência**: rode `cd bench && cargo bench --bench wire -- from_json_s2 validate`.
   `from_json_s2` deve ficar ≤15 ms. Se ficar acima, implemente a validação incremental:
   `Doc::validate_decoded(&self, schema)` que checa apenas `I8` (container vazio), `I16` (IDs
   duplicados), exclusões de marca (`I6`) e tabelas (`I17–I25`), porque `I2/I7/I3/I4/I1` já são
   verificados no decode; use em `check_doc` no lugar de `Doc::validate`, e prove equivalência com um
   teste diferencial em `tests/parser_validation.rs` sobre um corpus determinístico de documentos
   válidos/inválidos (para cada caso, `from_json(...).is_ok()` deve coincidir com
   `doc.validate(schema).is_ok()`); só então considere a meta cumprida.

### Fase 3 — A11y: orçar ≤10 ms (D)

1. **`DocPath::child` sem `format!`** em `crates/rich-editor-auy/src/pos.rs`: conte os dígitos do
   índice em `[u8; 10]` (loop para trás), `String::with_capacity(parent.len() + 1 + digits)`,
   `push_str(parent)`, `push('/')`, `push(char::from(b'0' + digit))`, `into_boxed_str()`. Bytes e
   ordem idênticos aos atuais; sem `unwrap`/`expect`.

2. **`a11y/tree.rs`**: `let root = DocPath::root();` uma vez e reuse em todos os blocos e no nó raiz;
   `Vec::with_capacity(self.blocks().len())` para `children`.

3. **`a11y/build.rs`**: pré-dimensione `children`/`items`; mude `block_nodes`/`child_nodes` para
   escrever em `&mut Vec<A11yNode>` (sink) em vez de retornar `Vec`, eliminando `Vec::from([node])`;
   para `a11y: None`, recurse direto no sink. Construa `text` uma única vez com
   `String::with_capacity(text.len())`/`into_boxed_str`; quando a política é `TextContent`, `name`
   recebe `Some(text.clone())` (a alocação é inevitável enquanto `A11yNode.text`/`name` forem dois
   `Option<Box<str>>`). Não mude o tipo público de `A11yNode`.

4. **Cache de AttrId por tipo de nó**: em novo `crates/rich-editor-auy/src/a11y/attrs.rs`, calcule
   uma vez por `a11y_tree` (a partir de `schema.nodes()`) os ids de `href`, `src`, `alt`, `caption`,
   `transcript`, `code_language`, `scope`, `colspan`, `rowspan`, `description`, `name`, `level` por
   `NodeKind`; `block_nodes` faz uma leitura `attrs.get(id)` por campo em vez de ~9 varreduras de
   nome. Mantenha o comportamento quando um attr não existe (`None`).

5. **Marcas em uma passada**: em `a11y/marks.rs`, adicione
   `marks_role_and_href(marks: &MarkSet, schema: &Schema) -> (A11yRole, Option<Box<str>>)` que
   percorre `schema.marks()` uma vez; em `child_nodes`, se `run.marks.is_empty()`, use
   `A11yRole::Text`/`None` sem varrer. Remova `mark_href` separado.

6. Mantenha `DocPath`, `A11yNode`, ordenação, ranges e goldens atuais; `tests/a11y.rs` e
   `tests/schema_full_a11y.rs` não podem mudar de expectativa. Rode
   `cd bench && cargo bench --bench wire -- validate_a11y_s2`; a estimativa deve ficar ≤10 ms.
   Contingência: se ainda exceder, adote o visitante único (build + validação na mesma travessia)
   descrito em `a11y/validate.rs`, preservando os mesmos `A11yCode`/paths.

### Fase 4 — Bench suite e gate de instruções (E, G)

1. **Reestruture `bench/`** conforme `plan/05 §8`:
   - `bench/src/lib.rs`: `pub enum Scenario { S0, S1, S2, S3, S4 }`,
     `pub struct Fixture { pub schema: Arc<Schema>, pub doc: Doc, pub json: String }`,
     `pub fn fixture(scenario: Scenario) -> Fixture` determinístico. S0 = 1 bloco/100 B; S1 = 100
     blocos/10 KiB/200 runs; S2 = 10 000 blocos/1 MiB/5 000 runs (runs de 200 B, dois por bloco);
     S3 = 1 bloco/1 MiB com ~256 runs de 4 KiB; S4 = 100 000 blocos/10 MiB (smoke). Use
     `Schema::basic()` para o corpus (o schema público é `full`, mas o corpus é sintético e o alvo
     do plano é o modelo; não introduza dependência do `full` no bench).
   - `bench/benches/hot.rs`: criterion com `iter_batched`/`iter_with_setup` para APIs que consomem
     estado/transação (setup fora da medição): insert no início/meio/fim de run, delete 1 char,
     delete 1 KiB, `AddMark` 1 KiB, split, join, transação de 10 steps, undo/redo; cenários
     S0/S1/S2/S3 e S4 smoke. Adicione `apply_owned_insert_s1/s2` com setup que reconstrói estado e
     transação fora do timed closure.
   - `bench/benches/wire.rs`: `to_json_into`, `to_json_string`, `from_json`, `check_json`,
     `Doc::validate`, `normalize` local, `map_pos`; S2.
   - `bench/benches/iai_hot.rs`: `iai-callgrind = "=0.16.1"`, benchmarks pequenos S0/S1 (insert,
     delete, mark, split/join, apply/apply_owned, from_json/to_json_string), com
     `LibraryBenchmarkConfig::default().tool(Callgrind::default().hard_limits([(EventKind::Ir, 5.0)]))`.
     `soft_limits` só reporta; o gate exige `hard_limits` (Ir é determinístico). Use
     `#[library_benchmark(setup = ...)]`/`#[bench::...]` para deixar a construção fora da medição.
   - `bench/Cargo.toml`: adicione `[lib] path = "src/lib.rs"`, `iai-callgrind = "=0.16.1"` em
     `[dev-dependencies]` e `[[bench]] name = "iai_hot" harness = false`.
   - Remova `bench/benches/hot_path.rs` depois de migrar todos os cenários (cutover limpo).

2. **`scripts/bench_instructions.sh`** (novo, executável):
   ```sh
   #!/usr/bin/env sh
   set -eu
   root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
   if ! command -v valgrind >/dev/null 2>&1; then
       echo "valgrind ausente; bench de instruções pulado"
       exit 0
   fi
   cd "$root/bench"
   if [ -f target/iai/main.json ]; then
       cargo bench --bench iai_hot -- --baseline=main
   else
       cargo bench --bench iai_hot -- --save-baseline=main
   fi
   ```
   `hard_limits` faz o bench falhar (código != 0) quando Ir excede 5% do baseline; o baseline só é
   gravado quando ainda não existe. Não use `--save-baseline` num PR: isso apagaria a regressão.

3. **Makefile**:
   - `bench:` → `cd bench && cargo bench --bench hot --bench wire`.
   - `bench-instructions:` → `./scripts/bench_instructions.sh`.
   - `.PHONY` inclui os dois.

4. **CI `bench`**: instale `valgrind` (`sudo apt-get update && sudo apt-get install -y valgrind`) e
   `iai-callgrind@0.16.1` (taiki-e/install-action); cache:
   - `actions/cache/restore@v4` com `path: bench/target/iai`, `key: iai-${{ hashFiles('bench/Cargo.lock','bench/benches/iai_hot.rs','bench/src/lib.rs') }}`,
     `restore-keys: iai-`.
   - rode `cd bench && cargo bench --bench hot --bench wire` e depois, se o cache restaurou o
     baseline, `cd bench && cargo bench --bench iai_hot -- --baseline=main`; se não restaurou
     (primeiro run), `--save-baseline=main`. O `hard_limits` falha o job em regressão.
   - `actions/cache/save@v4` só em `push` para `main`, depois do sucesso, mesma key.

5. **Relatório e calibração**: regenere `bench/ULTIMO.md` com os números criterion (S0–S4,
   operações) e a contagem de alocações. Atualize `plan/05 §11` com os alvos medidos: se S1 ficar
   acima de 0.1 µs (hipótese), registre o valor real e o alvo calibrado com 2× de folga (ex.: ≤1.5 µs)
   e o motivo (alocações fixas do step/histórico, sem `unsafe`); o gate é o iai + o teto de alocação,
   não o alvo de parede antigo. Isso fecha B sem fingir que 0.1 µs é atingível.

### Fase 5 — `schema!`: a11y, acessores, `trybuild` (A)

1. **Divida a macro** em arquivos ≤300 linhas em `crates/rich-editor-auy-macros/src/`: mantenha
   `lib.rs` (entrada/dispatch), `parse.rs` (tokens), crie `expand.rs` (doc/nó/marca), `attrs.rs`
   (tipos/defaults/required), `a11y.rs` (role/name/alt/level), `accessors.rs` (módulo de constantes).

2. **Gramática de atributos** em `attrs.rs`/`parse.rs`:
   - Tipos: `str`, `bool`, `ref`, `any`, `int(min..=max)`, `enum("a","b",...)`, `list(tipo)`,
     `map(tipo)`. Emita `AttrType::Enum(Box<[Box<str>]>)`, `AttrType::List(Box<AttrType>)`,
     `AttrType::Map(Box<AttrType>)`.
   - Marcador opcional `required` depois do tipo: `href: str required` emite `.required()`.
   - Defaults só para `str`, `bool`, `int`, `enum` (literal); `list`/`map`/`ref`/`any` com `=`
     produzem `compile_error!("unsupported attribute default")`.
   - Rejeite `attrs: { id: ... }` com `compile_error!("`id` is reserved")` e nomes de attr repetidos
     com `compile_error!("duplicate attribute `<name>`")`.

3. **Gramática de a11y** (em `a11y.rs`), aceita em `doc`, nó e marca:
   ```text
   a11y: {
       role: document | text | paragraph | list | list_item | quote | code | code_block |
             table | table_caption | table_row | table_cell | table_header | link | image |
             figure | caption | video | audio | attachment | separator |
             heading(<1..=6>),
       name: none | text | required | attr("<nome>"),
       alt: required | optional | not_applicable,
       level: "<nome-de-attr>"
   }
   ```
   Emita `A11ySpec::new(role)` + `.with_name(NamePolicy::…)` + `.with_alt(AltPolicy::…)` +
   `.with_level(AttrId(n))`. `attr("x")`/`level: "x"` resolvem o `AttrId` pela ordem de declaração
   (nó: attrs começam em 1 por causa do `id` reservado; doc/marca: começam em 0); attr inexistente →
   `compile_error!("unknown attribute `x`")`. Processe `attrs` antes de `a11y` em cada declaração,
   independentemente da ordem textual. `role: heading(n)` exige `level:` (senão
   `compile_error!("heading requires level")`); `custom` é rejeitado. Não revalide consistência na
   macro: `SchemaBuilder::build` já valida via `A11ySpec::error_code`.

4. **Modo módulo/acessores**:
   - Primeira declaração opcional: `pub mod NAME;` ou `mod NAME;`.
   - Sem ela, mantenha o modo expressão atual (`let schema = schema! { ... };`).
   - Com ela, a macro expande em item:
     ```rust
     pub mod NAME {
         pub const NODE_PARAGRAPH: ::rich_editor_auy::model::NodeKind = ::rich_editor_auy::model::NodeKind(0);
         pub const NODE_HEADING: ::rich_editor_auy::model::NodeKind = ...;
         pub const NODE_HEADING_LEVEL: ::rich_editor_auy::model::AttrId = ::rich_editor_auy::model::AttrId(1);
         pub const DOC_LANG: ::rich_editor_auy::model::AttrId = ...;
         pub const MARK_LINK: ::rich_editor_auy::model::MarkId = ...;
         pub const MARK_LINK_HREF: ::rich_editor_auy::model::AttrId = ...;
         pub fn build() -> ::core::result::Result<
             ::rich_editor_auy::schema::Schema,
             ::rich_editor_auy::error::SchemaError,
         > { /* Schema::builder()...build() */ }
     }
     ```
   - Nomes: `NODE_<NOME>` para nó, `NODE_<NÓ>_<ATTR>` para attr de nó, `DOC_<ATTR>`, `MARK_<NOME>`,
     `MARK_<NOME>_<ATTR>`; conversão para `SCREAMING_SNAKE_CASE`. Não gere para o `id` reservado.
   - Ordens: nós recebem `NodeKind` na ordem de declaração (sem `doc`); marcas recebem `MarkId` na
     ordem de declaração; attrs seguem a ordem do builder (`NodeSpec::new` reserva `id` em 0).
   - Rejeite dois `doc { ... }` com `compile_error!("duplicate doc declaration")`.

5. **Testes**:
   - `tests/macro_schema.rs`: mantenha o teste de expressão; adicione um módulo no topo do arquivo
     com `schema! { pub mod kinds; ... }` (com heading a11y e link com href required) e um teste
     `typed_accessors_match_the_builder` que compara `schema.node_kind("paragraph")` com
     `Some(kinds::NODE_PARAGRAPH)`, `heading`/`NODE_HEADING_LEVEL`, `MARK_LINK`/`MARK_LINK_HREF` e
     `kinds::build()`.
   - `tests/trybuild.rs`: `let t = trybuild::TestCases::new(); t.compile_fail("tests/ui/*.rs");`
     dentro de um `#[test]`; `compile_fail` retorna `()`, então não use `assert!`/`unwrap`.
   - `tests/ui/*.rs` + `.stderr` (gerados uma vez com `TRYBUILD=overwrite cargo test --test trybuild`):
     campo desconhecido, tipo `float`, `marks: "strong"`, `int(1..)`, default não-literal,
     `attrs: { id: str }`, role de a11y desconhecido, `heading` sem `level`, `name: attr("missing")`,
     attr duplicado, `doc` duplicado.
   - Adicione `trybuild = "1"` em `[dev-dependencies]` de `crates/rich-editor-auy/Cargo.toml`.
   - Atualize `Cargo.lock` (sem editar à mão).

6. **Docs**: atualize o rustdoc de `rich-editor-auy-macros/src/lib.rs` (gramática completa, modo
   módulo, a11y) e `README.md` (seção `schema!` com exemplo de acessores). Não prometa `features =
   ["macros"]` no core; a macro é crate separada (remova a afirmação obsoleta de `plan/04:56` ao
   atualizar o plano na fase 11).

### Fase 6 — Clipboard rico e composição no core/WASM/TS (L, resolve #39/#43)

1. **`Doc::slice`** em novo `crates/rich-editor-auy/src/clipboard.rs` (ou split
   `clipboard/{mod,slice,insert,ids}.rs`, cada ≤300 linhas), com `mod clipboard;` em `lib.rs`:
   ```rust
   impl Doc {
       pub fn slice(&self, range: Range) -> Result<Slice, StepError>;
   }
   ```
   Regras exatas:
   - `range.start > range.end`/fora do doc → `StepError::OutOfBounds`.
   - Mesmo bloco de texto: extraia `local_start = start - block_start - 1`,
     `local_end = end - block_start - 1` e use `step::text_edit::slice_text` (re-exporte
     `pub(crate)`) para produzir um `TextBlock` com marcas/átomos; devolva `Slice::from_textblock`.
   - Range multi-bloco: classifique cada endpoint com
     `enum Edge { DocEnd, BlockStart(usize), Inside(usize, usize) }`:
     `pos == doc.size()` → `DocEnd`; `block_index_at(doc, pos, true)` (de `step/structural.rs`,
     torne `pub(crate)`) retorna índice quando `pos` é o início de um bloco → `BlockStart(idx)`;
     senão `locate`/`block_at` dentro de bloco de texto → `Inside(idx, local)` onde
     `local = pos - block_start - 1`; qualquer outro caso → `Unsupported("slice_token")`.
     Monte os blocos assim:
     - `Inside(a, off)` (bloco de texto) vira `slice_text(block_a, off, text.len())` como primeiro
       bloco parcial;
     - `BlockStart(a)` como início inclui o bloco `a` inteiro;
     - blocos entre o primeiro e o último (exclusive os parciais) entram inteiros;
     - `Inside(b, off)` vira `slice_text(block_b, 0, off)` como último bloco parcial;
     - `BlockStart(b)` como fim exclui o bloco `b` (o range termina no início dele); `DocEnd` inclui
       até o último bloco.
     Se um bloco parcial não for `BlockContent::Text` → `Unsupported("slice_structural")`. Um range
     que produza zero blocos retorna `Slice` vazio.
   - Remova `AttrId::NODE_ID` recursivamente dos blocos/átomos copiados (novo
     `fn strip_block_ids(block: &Arc<Block>) -> Arc<Block>`; para `BlockContent::Text`, reconstrua os
     `InlineAtom` sem `id`; para `Blocks`, recurse).
   - Adicione `Slice::size(&self) -> u32` em `step/mod.rs` (conteúdo → `content.size()`, runs →
     `runs.len()`, senão `text.len()`).

2. **`Doc::insert_slice`/`Doc::replace_slice`** no mesmo módulo:
   ```rust
   impl Doc {
       pub fn insert_slice(&self, at: Pos, slice: &Slice, schema: &Schema) -> Result<Doc, StepError>;
       pub fn replace_slice(&self, range: Range, slice: &Slice, schema: &Schema) -> Result<Doc, StepError>;
   }
   ```
   - Slice inline (sem `content`): `apply_step(self, schema, &Step::Replace { from: at, to: at, slice: slice.clone() })`
     para insert, ou `from/to = range` para replace. Deixe o erro de boundary de `apply` propagar.
   - Slice de blocos: `replace_slice` exige ambos endpoints em boundary de bloco (senão
     `Unsupported("slice_not_block_aligned")`); `insert_slice` aceita boundary (insere no índice) ou
     posição dentro de bloco de texto (aplica `Step::ReplaceAround` de split e usa o boundary em `at`
     no doc resultante); monte `Vec<Arc<Block>>` com `strip_block_ids`, `Doc::from_blocks(attrs,
     blocks)` e propague `LimitError` como `StepError::Limit`.
   - Slice vazio → clone do doc.

3. **`Doc::delete_range`/`Editor::delete_range`** (necessário para seleção multi-bloco e para o
   adaptador):
   ```rust
   impl Doc {
       pub fn delete_range(&self, range: Range, schema: &Schema) -> Result<Doc, StepError>;
   }
   impl Editor {
       pub fn delete_range(&mut self, range: Range) -> Result<AppliedInfo, ApplyError>;
   }
   ```
   - Range vazio → clone do doc. `range.start > range.end`/fora do doc → `OutOfBounds`.
   - Se o range está dentro de um único bloco de texto (ambos os offsets em `1..=len` do mesmo
     bloco), aplique `Step::Replace { from, to, slice: Slice::plain_text("") }` (caminho quente).
   - Caso contrário, monte os blocos resultantes: mantenha os blocos antes do range; se
     `range.start` cai dentro de um bloco de texto, guarde o prefixo `[0, local_start)`; se
     `range.end` cai dentro de um bloco de texto, guarde o sufixo `[local_end, len)`; se ambos os
     lados existem e são text blocks do mesmo `kind`, funda prefixo+sufixo em um único `TextBlock`
     com os attrs do bloco inicial via `TextBlock::new`; se os kinds diferirem, mantenha dois blocos
     (prefixo e sufixo) separados; se um lado parcial for estrutural →
     `Unsupported("delete_structural")`; mantenha os blocos depois. Construa com
     `Doc::from_blocks(self.attrs().clone(), blocks)` e propague `LimitError` como `StepError::Limit`.
   - `Editor::delete_range` usa o novo doc + `transaction.replace_document(doc)` e
     `transaction.set_selection(Selection::caret(range.start))`; um único `ReplaceDoc` (undo exato).
     `Doc::delete_range` sozinho não cria histórico.
   - **Bloco de texto garantido**: se, depois da exclusão, a posição `range.start` não estiver no
     conteúdo de um bloco de texto (início de bloco, bloco estrutural ou documento vazio), insira um
     `TextBlock` vazio no boundary com o primeiro `NodeKind` textblock do schema e attrs vazios; o
     caret de `Editor::delete_range` passa a ser o início do conteúdo desse bloco
     (`block_start + 1`), não `range.start`. Se `range.start` já está dentro de um textblock, o caret
     continua `range.start`. Isso evita documento sem bloco editável e permite `insert_text`/
     `split_block` logo após uma seleção multi-bloco.
   - Em `crates/rich-editor-auy-wasm/src/stage.rs`, `Op::DeleteRange`: se o range está no mesmo bloco
     de texto, mantenha `transaction.delete(range)` (step granular); senão calcule
     `doc.delete_range(...)` e enfileire `transaction.replace_document(new_doc)` +
     `transaction.set_selection(Selection::caret(caret))`. Use o helper interno
     `Doc::same_text_block(range) -> bool` para a decisão e `Doc::caret_after_delete(range)` para o
     caret.

4. **Wire do `Slice`** em novo `crates/rich-editor-auy/src/wire/slice.rs`:
   - JSON canônico: `{"inline":[...]}` para inline (itens no formato de `content` de bloco:
     `{"type":"text","text":"…","marks":[…]}` / `{"type":"<atom>","attrs":{…}}`) ou
     `{"blocks":[...]}` para estrutura. Campos `"openStart"`/`"openEnd"` são emitidos só quando
     diferentes de 0; o decoder rejeita valores diferentes de 0 com
     `ParseError::new("open_slice_unsupported", value.start())`. Slice com `inline` e `blocks`
     não vazios → `ParseError::new("slice_mixed", value.start())`.
   - `pub fn from_slice_json(input, schema, options) -> Result<Slice, ParseError>`: parse, decodifica
     inline com `decode_inline` (torne `pub(super)`) + `TextBlock::new`/normalize com o teto
     `min(MAX_RUN_BYTES, max_block_text_bytes)`; blocos com `decode_blocks` + `Doc::from_decoded` +
     `check_doc` (reuse o caminho de `fragment.rs`); limites de attrs/marks via `decode_marks(..., limits)`.
   - `impl Slice { pub fn to_json_into(&self, schema, out) -> fmt::Result; pub fn to_json_string(&self, schema) -> String; }`
     usando `write_inline_items` (extraia de `writer.rs` como `pub(super)`) e `write_block` (torne
     `pub(super)`); escreva campos na ordem `inline`, `blocks`, `openStart`, `openEnd`, omitindo
     vazios/zero. Quando `runs` é `None`, `content` está vazio e `text` não é vazio, emita um único
     item inline `{"type":"text","text":"…"}` (sem marcas); quando `runs` é `Some`, emita os itens do
     `TextBlock`.

5. **`Editor` (core)** em `state/editor.rs`:
   ```rust
   pub fn slice(&self, range: Range) -> Result<Slice, StepError>;
   pub fn insert_slice(&mut self, at: Pos, slice: &Slice) -> Result<AppliedInfo, ApplyError>;
   pub fn replace_slice(&mut self, range: Range, slice: &Slice) -> Result<AppliedInfo, ApplyError>;
   pub fn composition(&mut self, event: CompositionEvent) -> Result<AppliedInfo, ApplyError>;
   pub fn stored_marks(&self) -> &MarkSet;
   ```
   - `insert_slice`/`replace_slice`: calcule o novo `Doc` com `Doc::insert_slice`/`Doc::replace_slice`,
     crie transação, `transaction.replace_document(doc)`, seleção `Selection::caret(Pos(at + slice.size()))`
     para slice inline ou `Selection::caret(Pos(at + slice.size() - 1))` para slice de blocos
     (último token de fechamento; use `saturating_sub(1)`), e `self.apply(transaction)`. Um único
     `Step::ReplaceDoc` por operação, undo restaura o snapshot exato.
   - `composition`: `let applied = self.state.apply_composition(event)?;` incremente `version` se o
     doc mudou (`!applied.state.doc().ptr_eq(self.state.doc())`), `self.state = applied.state;`,
     retorne `self.info(applied.changes)`.
   - `Transaction::insert_text` (em `state/transaction.rs`): quando `self.stored_marks` é
     `Some(marks)` e `!marks.is_empty()`, crie
     `Slice { text: Box::from(text), runs: Some(TextBlock::new(vec![InlineItem::Text(Run::new(text.into(), marks.clone()))])), content: Fragment::default(), open_start: 0, open_end: 0 }`
     em vez de `Slice::plain_text(text)`; senão mantenha o caminho atual.
   - WASM `Op::ToggleMark` (`stage.rs`): passe `editor.stored_marks()` para `stage` (ou troque o
     parâmetro `doc: &Doc` por `editor: &CoreEditor` e use `editor.doc()`); para range vazio,
     consulte o `MarkSet` guardado e chame `remove_mark` se a marca já estiver guardada, senão
     `add_mark`. Ambos os caminhos com range vazio só atualizam `stored_marks` e não geram step;
     `apply` deve devolver o estado mesmo com `changes` vazio.

6. **WASM** (`crates/rich-editor-auy-wasm/src/editor.rs`, `dto.rs`):
   - Métodos `#[wasm_bindgen]` na classe `Editor`:
     `slice(&self, from: u32, to: u32) -> Result<String, JsValue>`; `insert_slice(&mut self, at: u32, slice: &str) -> Result<String, JsValue>`;
     `replace_slice(&mut self, from: u32, to: u32, slice: &str) -> Result<String, JsValue>`;
     `composition(&mut self, payload: &str) -> Result<String, JsValue>`.
   - `CompositionJson` em `dto.rs`:
     ```rust
     #[derive(Deserialize)]
     #[serde(tag = "event", rename_all = "snake_case")]
     pub(crate) enum CompositionJson {
         Start { from: u32, to: u32 },
         Update { text: String },
         Commit,
         Cancel,
     }
     ```
   - Use `wire::from_slice_json`/`Slice::to_json_string`, `problem::from_edit_error`/
     `from_step_error`/`from_parse_error` para mapear erros; retorne o mesmo JSON de estado de
     `apply` (`state_json`/`applied_json`).

7. **TS core** (`packages/ts/packages/core/mod.ts`):
   - Tipos `CompositionJSON` (união com `event`), e métodos em `Editor`:
     `slice(from: number, to: number): string`;
     `insertSlice(at: number, slice: string): EditorStateJSON`;
     `replaceSlice(from: number, to: number, slice: string): EditorStateJSON`;
     `composition(event: CompositionJSON | string): EditorStateJSON`.
   - Chame os novos exports do WASM via `guarded`.

8. **Testes Rust** `crates/rich-editor-auy/tests/clipboard.rs`:
   - `slice_round_trip_preserves_marks_and_blocks`: doc com `strong` + link, copie inline e bloco,
     `to_json_string` → `from_slice_json` → mesmo `Slice` (bytes canônicos iguais).
   - `slice_strips_node_ids`: bloco com `id` → JSON do slice não contém `"id"`.
   - `insert_slice_pastes_marks_and_blocks`: `Editor::insert_slice` inline e de bloco; verifique JSON
     e undo.
   - `replace_slice_requires_block_boundaries`: range parcial + slice de blocos retorna
     `Unsupported("slice_not_block_aligned")`.
   - `slice_rejects_structural_partial`.
   - `composition_through_editor_records_one_undo_group` (use `Editor::composition` start/update/
     commit/undo).
   - Wire golden: JSON canônico exato de um slice inline com strong/link.

### Fase 7 — Adaptador DOM completo (L)

1. **`render.ts`**:
   - Assinatura `renderTree(host: HTMLElement, tree: A11yNode, options: RenderOptions = {})` com
     `RenderOptions { positions?: boolean; semantics?: "native" | "neutral" }`; `positions` default
     `true`, `semantics` default `"native"`.
   - `positions: true`: envolva o resultado de `renderText` em um `span` com
     `data-auy-from`/`data-auy-to` (o `span` fica por fora dos wrappers de marca); átomos já têm os
     atributos no elemento; blocos mantêm `data-auy-role/from/to`.
   - `semantics: "neutral"`: para `paragraph`, `heading`, `list`, `list_item`, `quote`, `code_block`,
     crie `div` com `role` ARIA (`paragraph`, `heading` + `aria-level`, `list`, `listitem`,
     `blockquote`, `code`) em vez de `p/hN/ul/ol/li/blockquote/pre`; os demais roles continuam
     nativos. Nunca emita `class`/`style`/`<style>`/`insertRule`/`.css`.
   - `positions: false` não emite `span` de texto.

2. **`selection.ts`** novo:
   ```ts
   export function readSelection(host: HTMLElement): { from: number; to: number } | undefined;
   export function writeSelection(host: HTMLElement, from: number, to: number): void;
   ```
   - `readSelection`: use `host.ownerDocument.getSelection()`; se o range não estiver contido no
     host, retorne `undefined`. Para nó de texto, ache o ancestral mais próximo com
     `data-auy-from/to` (o `span` do run), calcule o offset em bytes com
     `TextEncoder().encode(text.slice(0, offset)).length` e retorne `from + bytes`; some o mesmo
     para o endpoint `to`. Para elemento, use o filho no offset (`from` do filho) ou o fim
     (`to` do elemento/último filho). Se não houver âncora, retorne `undefined`; se o range não for
     colapsado e os endpoints caírem em âncoras diferentes, retorne os dois.
   - `writeSelection`: colete todos os elementos com `data-auy-from/to`; escolha o de menor
     `to - from` que contém `pos` (preferindo o `span` de texto ao bloco); dentro dele, percorra os
     nós de texto em ordem, acumulando `TextEncoder().encode(text).length`, e pare no nó cujo
     intervalo contém `pos`, convertendo byte→UTF-16 com `for (const ch of text)` e `ch.length`;
     se `pos` for o fim do elemento, use `(últimoText, últimoText.length)`; para posição estrutural
     (sem nó de texto), use `(elemento, 0)` ou `(elemento, childNodes.length)`. Depois
     `selection.removeAllRanges(); selection.addRange(range)`.
   - `selectionchange`/`beforeinput` usam essas funções; não leia `window.getSelection` global.
   - Teste de ida e volta (`writeSelection`→`readSelection`) para ASCII, `é` (2 bytes), emoji
     (4 bytes/2 UTF-16), início/fim de bloco e seleção multi-bloco.

3. **`input.ts`** novo, com `attachInput(host, editor, options, renderAfter)`:
   - `beforeinput`: leia a seleção; `event.preventDefault()` sempre que tratar:
     `insertText`/`insertReplacementText` → `delete_range` (se range) + `insert_text` **na posição
     `applied.selection.from` devolvida pela exclusão**, não no `from` antigo; sem range, use o caret
     atual;
     `deleteContentBackward` → range (o novo `delete_range` cobre multi-bloco) ou, com caret no
     início do conteúdo de um bloco (`pos == block_start + 1`), `join_backward(block_start)` (no
     primeiro bloco, no-op); caso contrário apague o grapheme anterior (`Intl.Segmenter` sobre o
     texto do run; fallback scalar) com `delete_range`; `deleteContentForward` → range ou, no fim do
     conteúdo de um bloco (`pos == block_end - 1`), `join_backward(block_end)` (no último bloco,
     no-op); caso contrário o grapheme seguinte; `insertParagraph`/`insertLineBreak` →
     `delete_range` (se range) + `split_block(applied.selection.from)`;
     `historyUndo`/`historyRedo` → `undo`/`redo`;
     `insertCompositionText` → ignora (não previne) durante composição; demais → previne e não muta.
     Depois de aplicar, chame `options.onChange?.(applied)`, `options.onError?.(error)` em falha e
     `renderAfter(applied.selection)`; em erro, anuncie `error.detail` e não muta o DOM.
   - `keydown`: durante composição só trate `Escape` (cancela composição); `Ctrl/Cmd+Z`,
     `Ctrl/Cmd+Shift+Z`/`Y` → undo/redo; `Ctrl/Cmd+B/I/U` → `toggle_mark` na seleção (ou stored
     mark com range vazio); `Tab` não previne (sai do editor).
   - `copy`/`cut`: selecione `from/to`; `text/plain` = `selection.toString()`; rich =
     `editor.slice(from,to)` no MIME `application/x-rich-editor-auy+json`; `event.preventDefault()`;
     no `cut`, aplique `delete_range` e re-renderize.
   - `paste`: leia `application/x-rich-editor-auy+json`; se houver, `replaceSlice` (range não vazio)
     ou `insertSlice` (caret); se o core rejeitar (boundary), aplique `delete_range` e depois
     `insertSlice` no caret devolvido; se não houver rich, aplique `delete_range` (se range) e
     `insert_text` com `text/plain` no caret devolvido (stored marks aplicam). `preventDefault()`
     sempre.
   - `selectionchange`: se `composing`, ignore; senão `readSelection` → `editor.setSelection(from,to)`
     (sem criar histórico) e atualize o cache de seleção.
   - Composição: `compositionstart` captura `from/to`; se o range atravessa blocos ou não está
     dentro de um único textblock, colapse para `from/from` antes de chamar
     `editor.composition({event:"start",from,to})` (o core exige um textblock); defina
     `composing = true`; `compositionupdate` não re-renderiza; `compositionend` define
     `composing = false`, chama `editor.composition({event:"update",text:event.data ?? ""})` seguido de
     `editor.composition({event:"commit"})`, re-renderiza e anuncia o texto; `Escape` durante
     composição chama `{event:"cancel"}`, `composing = false`, e re-renderiza. `beforeinput` de
     composição não previne.
   - Erros de apply: chame `onError` (se houver), anuncie `error.detail` e não muta o DOM.

4. **`announce.ts`** novo:
   ```ts
   export function createAnnouncer(host: HTMLElement, mode: "local" | "host" | "off"):
     { announce(text: string): void; destroy(): void };
   ```
   `local`: cria `div` com `data-auy-live`, `aria-live="polite"`, `aria-atomic="true"` após o host e
   remove no destroy; `host`: usa `host.querySelector('[aria-live]')` (no-op se ausente); `off`:
   no-op. Anuncie undo/redo, commit de composição e erros; não anuncie cada tecla.

5. **`mod.ts`**:
   ```ts
   export interface EditorLike extends StateProvider {
     apply(payload: BatchPayload | string): EditorStateJSON;
     toJSON(): EditorStateJSON;
     undo(): EditorStateJSON & { readonly applied: boolean };
     redo(): EditorStateJSON & { readonly applied: boolean };
     setSelection(from: number, to: number): EditorStateJSON;
     version(): number;
     slice(from: number, to: number): string;
     insertSlice(at: number, slice: string): EditorStateJSON;
     replaceSlice(from: number, to: number, slice: string): EditorStateJSON;
     composition(event: CompositionJSON | string): EditorStateJSON;
   }
   export interface MountOptions {
     readonly state: EditorLike;
     readonly editable?: boolean;              // default true
     readonly label?: string;                  // default "Document"
     readonly semantics?: "native" | "neutral";// default "native"
     readonly positions?: boolean;             // default true
     readonly manageSelection?: boolean;       // default true
     readonly announce?: "local" | "host" | "off"; // default "local"
     readonly onChange?: (state: EditorStateJSON) => void;
     readonly onError?: (error: DocumentError) => void;
   }
   ```
   - Remova `onInput`. `mount` salva/restaura também `aria-label` e `aria-readonly`; `editable:false`
     remove listeners de edição, seta `aria-readonly="true"` e mantém render/dispatch.
   - Cache da árvore por versão: `render()` compara `state.version()` com a última; se igual,
     reutiliza o `A11yNode`; senão `state.a11yTree()`. Não chame `toJSON()` só para ler a versão.
   - `render()` → `renderTree(host, tree, {positions, semantics})`; depois `writeSelection` se
     `manageSelection && positions && doc.activeElement === host`.
   - `destroy()` remove todos os listeners (`beforeinput`, `keydown`, `copy`, `cut`, `paste`,
     `compositionstart`, `compositionend`, `selectionchange`), o announcer, restaura atributos/html.
   - `dispatch(next)` troca `state`, zera o cache, renderiza e restaura seleção.
   - `positions:false` força `manageSelection` a false.

6. **`packages/ts/packages/dom/deno.json`**: `publish.include` passa a listar `mod.ts`, `render.ts`,
   `selection.ts`, `input.ts`, `announce.ts`, `LICENSE`. Atualize `packages/ts/deno.json` task
   `check` para `deno test --allow-read --allow-env`.

7. **Testes** (jsdom):
   - `selection_test.ts`: render com posições; crie `Range` em jsdom; `readSelection`/`writeSelection`
     round-trip em ASCII e acentos/emoji.
   - `input_test.ts`: mount com `Editor.fromJSON` real + `await init()`; dispare `InputEvent`
     `beforeinput` (`insertText`, `deleteContentBackward`, `insertParagraph`), confira
     `editor.doc().plainText()`, JSON e caret; undo/redo por `keydown`.
   - `clipboard_test.ts`: eventos `copy`/`cut`/`paste` com `clipboardData` fake (`getData`/`setData`);
     confira MIME rich, marcas preservadas, plain fallback e undo.
   - `composition_test.ts`: `compositionstart`/`compositionend`; confira um único grupo de undo e
     texto final.
   - `render_test.ts`: atualize o fake DOM para `span`/`dataset`/`attributes`; asserções de âncoras,
     `positions:false` sem `span`, e `semantics:"neutral"` (`div[role=heading][aria-level]`, sem
     `h1`/`ul`/`li`).
   - `mod_test.ts`: mount/destroy restaura `innerHTML`/atributos; `aria-readonly` em `editable:false`;
     `announce` cria/remove `[data-auy-live]`.
   - `a11y_test.ts`: jsdom + `npm:axe-core@4`; crie `new JSDOM("<!doctype html><body></body>")`,
     atribua `globalThis.window`/`document` e use `dom.window.document` para o `host`; obtenha a
     árvore de um `Editor` com WASM (`await init()`), renderize com
     `renderTree(host, tree, { positions: true })` em um `div` simples (sem `role="textbox"`), e
     rode `axe.run(host, { rules: { "color-contrast": { enabled: false } } })`;
     `assertEquals(results.violations, [])`. Audite a projeção semântica, não o shell de edição.
     Sem `class=`/`style=` literais no arquivo.
   - Mantenha os testes existentes verdes; migre o fake DOM se necessário.

### Fase 8 — Fuzz executado (I)

1. Em `fuzz/fuzz_targets/`, use `Schema::full()` em `json_parse`, `a11y_tree` e `apply_step` (o
   boundary oficial usa o schema completo; cobre tabela/mídia).
2. Diferencie:
   - `json_parse`: `check_json` + `from_json` → `to_json_string` (sem a11y).
   - `a11y_tree`: `from_json` → `a11y_tree` → `validate_a11y` → `to_json_string`, e verifique
     determinismo chamando duas vezes e comparando o JSON (sem panic).
   - `apply_step`: como hoje, com `Schema::full()`.
3. Novo alvo `round_trip` em `fuzz/fuzz_targets/round_trip.rs` + `[[bin]]` em `fuzz/Cargo.toml`:
   `from_json` válido → `to_json_string` → `from_json` → `to_json_string`; asserte igualdade de bytes
   (use `assert_eq!`).
4. Adicione `fuzz/corpus/json_parse/seed.paragraph.json` e
   `fuzz/corpus/json_parse/seed.marks.json` (documentos canônicos pequenos, válidos no schema
   completo, sem floats); copie os mesmos seeds para `a11y_tree`, `apply_step` e `round_trip`.
5. `Makefile fuzz` inclui `round_trip`; CI `fuzz` inclui `cargo +nightly fuzz run round_trip -- -max_total_time=30`.
6. Execute localmente 60 s por alvo (`cd fuzz && cargo +nightly fuzz run … -- -max_total_time=60`);
   nenhum crash. O corpus gerado em `fuzz/corpus/<alvo>/` deve ser commitado (seeds + achados
   válidos); `fuzz/artifacts/` entra no `.gitignore`.

### Fase 9 — PHP executado (J)

1. **Imagem de gate** `packages/php/docker/Dockerfile`:
   ```dockerfile
   FROM composer:2 AS composer
   FROM php:8.3-cli
   RUN apt-get update \
    && apt-get install -y --no-install-recommends git unzip libonig-dev \
    && docker-php-ext-install mbstring \
    && rm -rf /var/lib/apt/lists/*
   COPY --from=composer:2 /usr/bin/composer /usr/bin/composer
   WORKDIR /app
   ```
   `php:8.3-cli` já traz `dom`, `xml`, `xmlwriter`, `tokenizer`, `json`; `mbstring` cobre o PHPUnit 11.
2. **Makefile** novo alvo `php` (e `check-ecosystem` chama `$(MAKE) php`):
   ```sh
   docker build -t rich-editor-auy-php packages/php
   docker run --rm -u "$$(id -u):$$(id -g)" -e COMPOSER_HOME=/tmp/composer \
     -v "$(CURDIR)/packages/php":/app -w /app rich-editor-auy-php \
     sh -c 'composer update --no-interaction --prefer-dist && composer validate --strict \
       && composer audit && vendor/bin/phpunit && vendor/bin/phpstan analyse'
   ```
   Sem `docker`/`composer`: mensagem `php/composer/docker ausente; pulado` (consistente com os
   outros alvos opcionais locais; o CI tem o job `php` com a imagem acima). Com `composer` local,
   use o mesmo comando (`composer update` primeiro; não há lock versionado).
3. `.gitignore`: adicione `/packages/php/vendor/` e `/packages/php/composer.lock`.
4. Rode `make php`; PHPUnit (4 testes) e PHPStan level max devem passar; `composer validate
   --strict`/`audit` limpos. Registre os comandos e a saída no `DIVERGENCES.md`.

### Fase 10 — Pacote WASM, `wasm-opt` e `axe-core` (K, M)

1. **Pacote WASM**: `Makefile package-wasm` passa a `cargo package -p rich-editor-auy-wasm --locked
   --allow-dirty --no-verify`; adicione `package-wasm` ao alvo `check` (após `package`). O build real
   continua no CI `wasm`; a ordem de publicação `rich-editor-auy` → `rich-editor-auy-macros` →
   `rich-editor-auy-wasm` fica documentada em `README.md` e `plan/08`.
2. **`scripts/build_wasm.sh`**:
   - Se `command -v wasm-opt` e `AUY_REQUIRE_WASM_OPT=1` ausente: comportamento atual (opcional).
   - Se `wasm-opt` existir: copie o wasm para `"$out/.wasm-unoptimized"`, rode `wasm-opt -Oz … -o`
     e falhe (`exit 1`) se `wasm-opt` retornar erro; compare os tamanhos e emita um aviso (não
     falha) se o otimizado for maior; remova o temporário.
   - Se `AUY_REQUIRE_WASM_OPT=1` e `wasm-opt` ausente: `exit 1` com `wasm-opt ausente`.
3. **CI `wasm`**: instale `wasm-bindgen-cli@0.2.130,wasm-opt` (taiki-e/install-action) e rode
   `AUY_REQUIRE_WASM_OPT=1 ./scripts/build_wasm.sh`; o job `check` roda
   `cargo package -p rich-editor-auy-wasm --locked --allow-dirty --no-verify`.
4. **`axe-core`**: o teste `a11y_test.ts` da fase 7 é o gate; `cd packages/ts && deno task check` roda
   jsdom+axe no CI. Contingência: se `axe.run` não funcionar sob jsdom, troque a implementação do
   teste para `npm:happy-dom` mantendo as mesmas asserções; se ainda assim não rodar, adicione um job
   CI com Playwright/Chromium (`deno run -A npm:playwright@1 install chromium`) e o mesmo `axe.run`
   em página real. Registre a troca em `DIVERGENCES.md`.
5. **Verificação local do wasm-opt**: `npm install --prefix /tmp/auy-binaryen binaryen@latest` e
   `PATH=/tmp/auy-binaryen/node_modules/.bin:$PATH AUY_REQUIRE_WASM_OPT=1 ./scripts/build_wasm.sh`
   (requer `wasm-bindgen` CLI 0.2.130 instalado: `cargo install wasm-bindgen-cli --version 0.2.130 --locked`).
6. **Artefato commitado**: como `packages/ts/packages/core/wasm/*` é versionado, rode
   `./scripts/build_wasm.sh` depois de qualquer mudança de exports WASM e commite os quatro arquivos
   regenerados (`*.js`, `*.d.ts`, `*_bg.wasm`, `*_bg.wasm.d.ts`); o `.blake3` só é gerado quando
   `b3sum` existe.

### Fase 11 — Registro e documentação (DIVERGENCES, CHANGELOG, plan, README)

1. **`DIVERGENCES.md`** (português):
   - Remova as linhas A–M da tabela “Divergências abertas” e substitua a seção por
     `Nenhuma divergência aberta neste ciclo.`
   - Adicione na tabela “Resolvidas durante a implementação” uma linha por item (45–57), com
     `Plano | Implementação | Resolução` e evidência (teste/comando), incluindo:
     - A: gramática completa, acessores, `trybuild`; `tests/macro_schema.rs`/`tests/trybuild.rs`.
     - B/F: `History` com `Vec<Arc<Group>>` (clone O(1) em grupos), teto de alocações calibrado em
       `tests/alloc.rs` e justificativa por site; `bench/ULTIMO.md`.
     - C/H: limites cedo, `Doc::from_decoded`, normalização única; `from_json_s2` ≤15 ms;
       `tests/parser_limits.rs`.
     - D: `DocPath`/a11y; `validate_a11y_s2` ≤10 ms.
     - E: `iai_hot` + `hard_limits(Ir, 5%)` + baseline `main` em cache de CI;
       `scripts/bench_instructions.sh`.
     - G: S0–S4 e operações em `bench/{hot,wire}.rs`.
     - I: fuzz full-schema + `round_trip` + corpus; comandos e nenhum crash.
     - J: container `composer:2`/`php:8.3-cli`; PHPUnit/PHPStan/composer.
     - K: `--no-verify` + CI wasm; ordem de publicação.
     - L: adaptador completo; testes jsdom/axe.
     - M: `wasm-opt` obrigatório no CI + axe.
   - Atualize as linhas já resolvidas #38 (envelope v1 sem migração), #39 (agora `Doc::slice`/
     `Slice` JSON/`insert_slice`) e #43 (composição exposta por `Editor`/WASM/TS) para refletir a
     implementação nova.
2. **`CHANGELOG.md`** (português, `## [Não publicado]`): adicione as entradas de `Slice`/clipboard,
   composição no `Editor`/WASM/TS, adaptador DOM (seleção/input/IME/clipboard/opções), `schema!`
   a11y/acessores/trybuild, limites no decode, otimizações de wire/a11y/hot path, `iai-callgrind`,
   fuzz `round_trip`, PHP em container e gates de wasm-opt/axe.
3. **Docs**: atualize `README.md` (adaptador, clipboard MIME, IME, `schema!`), `examples/ts/main.ts`
   (use `Editor` e as novas opções), `examples/host/index.html` (troque `EditorState` por `Editor`;
   `mount` agora exige `EditorLike`), `plan/04` (macro: crate separada, acessores;
   remova `features=["macros"]` e a frase de que a macro não existe), `plan/05 §3.3/§8` (baseline
   nativo `target/iai` + cache de CI no lugar de `bench/baseline.json`; `hot.rs`/`wire.rs`/
   `iai_hot.rs`/`src/lib.rs`), `plan/07` (job bench com valgrind/iai), `plan/08` (ordem de
   publicação) e `AGENTS.md` §0 (estado atual sem as lacunas resolvidas). Rode
   `python3 scripts/check_links.py`.

## Arquivos críticos e âncoras

- `crates/rich-editor-auy/src/state/history.rs` — `done`/`undone` passam a `Vec<Arc<Group>>`; o
  `EditorState::clone` deixa de copiar `steps`/`inverse` de cada grupo.
- `crates/rich-editor-auy/src/state/editor.rs:160-167` — mantém o clone do `EditorState` como
  mecanismo de atomicidade; o `Arc<Doc>` compartilhado força o copy-on-write do bloco tocado.
- `crates/rich-editor-auy/src/wire/{parser.rs,attrs.rs}` — decode; threads de `Limits` e
  `DecodeBudget`, remoção de `content_allows` e de `Doc::new`.
- `crates/rich-editor-auy/src/clipboard.rs` (novo) — `Doc::slice`/`insert_slice`/`replace_slice`,
  strip de IDs e regras de boundary.
- `packages/ts/packages/dom/{mod,render,selection,input,announce}.ts` — ponte DOM↔core; cada arquivo
  ≤300 linhas e sem `class`/`style`.

## Verificação

Pré-requisito: `wasm-bindgen` CLI 0.2.130 (`cargo install wasm-bindgen-cli --version 0.2.130 --locked`),
`deno` 2.x, `docker`, `cargo-fuzz`/nightly e rede para crates/npm.

1. **Gate do repositório**:
   ```sh
   make check
   ```
   Deve passar fmt, clippy `-D warnings`, `cargo test --workspace --locked`, `visits`, doc, a11y,
   file-length, links, zero-style, `cargo package` core/macros e `package-wasm --no-verify`.
2. **Comportamento novo (core)**:
   ```sh
   cargo test -p rich-editor-auy --locked --test clipboard
   cargo test -p rich-editor-auy --locked --test macro_schema
   cargo test -p rich-editor-auy --locked --test trybuild
   cargo test -p rich-editor-auy --locked --test alloc
   cargo test -p rich-editor-auy --locked --test parser_limits
   cargo test -p rich-editor-auy --locked --test history
   ```
   Esperado: clipboard round-trip/corte/colagem, acessores `Kinds::NODE_*`, `*.stderr` do trybuild,
   teto de alocação calibrado, limites com códigos estáveis, composição em um grupo de undo.
3. **Wire/a11y nos alvos** (criterion):
   ```sh
   cd bench
   cargo bench --bench wire -- from_json_s2 validate_a11y_s2
   ```
   Esperado: `from_json_s2` ≤15 ms e `validate_a11y_s2` ≤10 ms (registre os valores em
   `bench/ULTIMO.md`).
4. **Gate de instruções** (Docker, já que o host não tem valgrind):
   ```sh
   docker run --rm -v "$PWD":/src -w /src/bench rust:1.99-bookworm bash -lc \
     'apt-get update && apt-get install -y valgrind && \
      cargo install iai-callgrind-runner --version 0.16.1 --locked && \
      cargo bench --bench iai_hot -- --save-baseline=main'
   docker run --rm -v "$PWD":/src -w /src/bench rust:1.99-bookworm bash -lc \
     'cargo bench --bench iai_hot -- --baseline=main'
   ```
   Rode o primeiro comando uma vez (cria o baseline) e o segundo para comparar; a saída termina com
   `Iai-Callgrind result: Ok` (exit 0); uma mudança >5% em `Ir` sai com código != 0.
5. **TS/adaptador + axe + wasm**:
   ```sh
   cd packages/ts && deno task check && deno task publish:dry
   cd ../..
   cargo install wasm-bindgen-cli --version 0.2.130 --locked   # se ausente
   cargo build -p rich-editor-auy-wasm --target wasm32-unknown-unknown --release --locked
   ./scripts/build_wasm.sh
   npm install --prefix /tmp/auy-binaryen binaryen@latest
   PATH=/tmp/auy-binaryen/node_modules/.bin:$PATH AUY_REQUIRE_WASM_OPT=1 ./scripts/build_wasm.sh
   ```
   Esperado: testes de seleção/input/clipboard/IME/axe verdes; `wasm-opt` roda sem erro (o tamanho
   é reportado; um aumento apenas emite aviso); `publish:dry` aceita os novos arquivos.
6. **PHP**:
   ```sh
   make php
   ```
   Esperado: `composer validate --strict`, `composer audit`, PHPUnit (4 testes) e PHPStan level max
   verdes no container.
7. **Fuzz**:
   ```sh
   cd fuzz
   cargo +nightly fuzz run json_parse -- -max_total_time=60
   cargo +nightly fuzz run a11y_tree -- -max_total_time=60
   cargo +nightly fuzz run apply_step -- -max_total_time=60
   cargo +nightly fuzz run round_trip -- -max_total_time=60
   ```
   Esperado: nenhum crash/panic; `fuzz/corpus/` com seeds.
8. **Ecosistema**:
   ```sh
   make check-ecosystem
   ```
   Deve cobrir Deno (`check`/`publish:dry`) e PHP (via `make php`).

## Premissas e contingências

- O host tem Docker, Deno, cargo-fuzz e nightly; valgrind/PHP/wasm-bindgen/wasm-opt não estão
  instalados. O gate de instruções roda em container/CI; PHP roda em container; `wasm-opt` roda via
  npm/CI.
- `rust:1.99-bookworm` existe; se o pull falhar, use `rust:bookworm` e
  `rustup toolchain install 1.99 && rustup override set 1.99` antes do `cargo bench`.
- `trybuild` gera `.stderr` sensível à versão; gere com o toolchain 1.99 (`TRYBUILD=overwrite
  cargo test --test trybuild`) e só commite depois de `make check` verde.
- jsdom+axe: se `axe.run` falhar sob jsdom, troque para `happy-dom`; se ainda falhar, use Playwright/
  Chromium no CI. Registre a implementação escolhida em `DIVERGENCES.md` (M).
- `from_json_s2` acima de 15 ms após a fase 2: implemente `Doc::validate_decoded` + teste
  diferencial (passo 7 da fase 2); não relaxe o alvo.
- `validate_a11y_s2` acima de 10 ms após O1–O5: adote o visitante único de build+validação (passo 6
  da fase 3), mantendo `A11yCode`/paths.
- Se `iai-callgrind 0.16.1` não compilar no toolchain 1.99, fixe a última versão compatível e
  atualize `scripts/bench_instructions.sh`/CI juntos (a versão do runner deve casar com a do crate).
- `cargo package --no-verify` atualiza o índice do crates.io; em ambiente offline ele falha — o gate
  local exige rede, como os demais `cargo package` já exigem.
