# Divergências entre o plano e a implementação

Este arquivo registra cada ponto em que a implementação não seguiu literalmente `plan/`, por que a
decisão foi tomada e o estado atual. Uma divergência sem solução registrada é uma pendência, não um
recurso.

## Resolvidas durante a implementação

| # | Plano | Implementação | Resolução |
|---|---|---|---|
| 1 | `Step::Replace` aceita `Slice` textual; conteúdo estrutural é rejeitado | `Replace` substitui blocos quando `Slice::content` não está vazio; `ReplaceAround` continua sendo o passo de split | `apply::replace_blocks`/`invert::blocks_between`; join e undo de split sem um sétimo variant |
| 2 | `join` usa a álgebra de `ReplaceAround` | `Transaction::join_backward` monta um `Slice` de fragmento com o bloco fundido e emite `Replace` estrutural | `state/transaction_join.rs`; apenas blocos de texto de topo, mesmo tipo e sem passos anteriores |
| 3 | `ReplaceAround` poderia ser invertido | A inversão cobre o caso de split usado pelo MVP; `ReplaceAround` genérico continua rejeitado | `step/invert.rs` |
| 4 | `locate` visita blocos até encontrar a posição | `Doc` mantém índice `starts`; `block_at` usa busca binária; posições fora do doc retornam `None` | Teste `tests/visits.rs`; feature `visits` |
| 5 | Memória dentro da fórmula de `plan/05` | `TextBlock::new` reservava capacidade para quatro itens | `Vec::with_capacity(items.len())`; teste `tests/memory.rs` cabe na fórmula |
| 6 | `A11yTree` sem projeção JSON | `to_json_into`/`to_json_string` produzem JSON canônico estável | `a11y/json.rs`, WASM e CLI `a11y-tree` |
| 7 | `Slice` carrega `text` além de `Fragment` | Mantido: texto é o caminho quente; `Fragment` cobre substituições estruturais; `Slice::runs` preserva marcas em inversos | Teste `undo_restores_marks_removed_by_delete` |
| 8 | `apply_owned` deveria evitar a cópia do snapshot | `apply_owned` destrói o `Transaction::base`, muta `DocInner`/chunk com `Arc::get_mut` e cai para o caminho imutável quando há clones | Teste de equivalência em `tests/editing.rs`; teste de alocação em `tests/alloc.rs` |
| 9 | Multi-step undo aplica inversos em ordem reversa | `apply` guarda inversos na ordem dos steps e `History::undo` percorre `.rev()`; o `reverse()` duplicado foi removido | Teste `multi_step_undo_restores_original_text` |
| 10 | `StepMap` de `ReplaceAround`/split | `new_len` agora é `insert`, não `from + insert` | Teste `tests/mapping.rs` |
| 11 | `Doc::validate` com I1–I12/I16 | Implementado com path/posição, attrs de root/nó/marca, átomo inline, payload textual, runs, IDs e tamanho | `model/validate.rs`; testes `validation.rs`/`parser_validation.rs` |
| 12 | `SetNodeAttr`/`SetDocAttr` com valor ausente | `AttrValue::Null` remove o atributo; `set_doc_attr` valida o `DocSpec` e o tipo | Teste `undo_removes_a_document_attribute_added_by_a_transaction` |
| 13 | `EditorState::new` deveria abrir um documento editável | Estado inicial usa o primeiro tipo do grupo `block` e cria um textblock vazio | Teste `fresh_editor_state_accepts_the_first_insertion` |
| 14 | Wire não validava schema | `decode_doc` chama `Doc::validate`; attrs/required/range/refs e conteúdo inline são checados; CLI `validate` reporta `DocIssue` | `tests/parser_validation.rs`, CLI |
| 15 | Fingerprint cobre toda a declaração | Inclui `atom`/`defining`/`isolating`, `a11y`, `excludes`, grupos e defaults | `schema/fingerprint.rs` |
| 16 | DOM não pode aceitar URL perigosa | `href`/`src` passam por allowlist (`http`, `https`, `mailto`, `tel`, relativo) | `render.ts`/`render_test.ts` |
| 17 | TS `fromEnvelope` ignorava o envelope | WASM expõe `envelope_doc`/`canonicalize_envelope`; TS valida antes de decodificar | `Document.fromEnvelope` |
| 18 | CLI/PHP não aceitavam envelope | CLI detecta envelope e usa `Envelope::from_json`; PHP valida formato/versão/fingerprint/doc objeto | `cli/src/main.rs`, `packages/php` |
| 19 | Link era hardcoded em `build.rs` | Role/`href` vêm de `MarkSpec.a11y` e attrs; preset declara `link` como `A11yRole::Link` | `a11y/marks.rs`, teste `a11y_maps_link_marks_to_link_role` |
| 20 | Fuzz/Miri/a11y gates não executavam | `a11y` tem `tests/a11y.rs`; Miri usa `miri_smoke`; fuzz tem `json_parse`, `a11y_tree`, `apply_step` | Makefile/CI |
| 21 | `trybuild` era dependência de workspace sem uso | Removido | `cargo machete` limpo |
| 22 | Wire aceitava surrogates isolados e offsets ficavam em 0 | `json_escape` decodifica pares `\uD83D\uDE00`, rejeita isolados e os decoders usam `JsonValue::start()` | `tests/parser_limits.rs` |
| 23 | Limites de attrs/runs/bloco não eram aplicados | `model/limits.rs` percorre attrs, marcas, runs e texto em `Doc::new` e em substituições | `enforces_*_limits` |
| 24 | A11y usava índice da árvore, não `DocPath` | `A11yNode.path` é projetado do documento e `validate_a11y` usa esse path | `a11y_reports_*` |
| 25 | `schema!` era placeholder silencioso | Proc-macro real com parser de tokens para doc/nós/marcas/attrs e teste de integração; compila no `no_std` (retorna `SchemaError`) | `tests/macro_schema.rs` |
| 26 | `apply` copiava todo o vetor de blocos e `starts` | Blocos em chunks de 128; substituição copia um chunk + diretório | `model/chunks.rs`, `tests/chunks.rs`; S2 106,5 → 2,89 µs |
| 27 | Documentos pequenos pagavam o diretório de chunks | Até 128 blocos usam um chunk flat; acima disso, diretório de chunks | `model/chunk.rs`; S1 2,05 → 1,73 µs |
| 28 | `apply_owned` pagava cópia de diretório e chunks | Caminho owned com `Arc::get_mut`: 35 → 25 alocações no insert de 1 char | `model/doc.rs`, `model/chunks.rs`, `state/mod.rs`; `tests/alloc.rs` |
| 29 | Inversão fazia dois splits para extrair um intervalo | `text_edit::slice_text` extrai o intervalo uma vez, com as mesmas marcas/átomos | `step/text_edit.rs`, `step/invert.rs` |
| 30 | Parser montava uma `String` de path por nó e crescia `String` byte a byte | Path em pilha (`DecodePath`) materializado só no erro e scanner de strings sem escape por slice | `wire/parser.rs`, `wire/attrs.rs`, `wire/json_parser.rs`; from_json 34,78 → 20,87 ms; `check_json` 20,37 → 8,89 ms |
| 31 | Writer crescia o `String` final e escrevia run a run | Capacidade pré-calculada e `write_string` copia trechos sem escape de uma vez | `wire/mod.rs`, `wire/writer.rs`; `to_json_string` 2,97 → 1,12 ms |
| 32 | A11y clonava o `DocPath` ao inserir o nó | `block_nodes` recebe o path por valor e move para o nó; filhos recebem paths novos | `a11y/build.rs`, `a11y/tree.rs`; validate_a11y 11,34 → 11,04 ms |
| 33 | Writer gravava attrs de marca em `attrs`, mas o decoder só aceitava campos planos | Decoder aceita o objeto `attrs` canônico e mantém compatibilidade com campos planos | `wire/attrs.rs`; teste de round-trip `accepts_canonical_nested_mark_attrs_and_round_trips` |
| 34 | Figura conteria `image` inline, mas o conteúdo de `figure` é estrutural | Schema completo adiciona `image_block` (mesmos attrs/a11y) e `figure` usa `image_block caption?`; `image` continua inline | `schema/full/nodes.rs`, testes de round-trip de figura |
| 35 | Comandos de tabela por célula no `Transaction` | Primeira versão reconstrói apenas o bloco `table` e aplica um `Step::ReplaceDoc`; undo restaura o snapshot exato | `state/table_commands.rs`, `state/table/`; `tests/table_commands.rs` |
| 36 | Ingestão aceitaria `embed` para tokens não representáveis | Não existe `embed`: token desconhecido retorna `ProblemDetails` e sugestão de conversão; payload bruto só entra como `raw_code` textual | `error/problem.rs`, `schema/full`; `tests/schema_full_problems.rs` |
| 37 | `Schema::basic` e `SchemaBuilder` deixariam de ser públicos | `basic()` fica atrás da feature `migration` (testkit, testes e benchmark a habilitam); `Schema::full()` é a única construção na API padrão. `SchemaBuilder` continua público porque `schema!` expande em crates consumidores | `schema/preset.rs`, `crates/rich-editor-auy/Cargo.toml`; `cargo doc` padrão não expõe `basic` |
| 38 | Envelope com `schema_version=2` e `migrate_v1_to_full` | Writer grava versão 2 com fingerprint completo; leitura aceita 1 e 2; v1 é migrado por `Envelope::migrate_v1_to_full` e o PHP aceita 1/2 | `wire/envelope.rs`, `packages/php/src/Envelope.php`; `tests/envelope.rs` |
| 39 | Clipboard `Doc::slice`/`insertSlice`/`SliceJSON` | `Doc::slice`/`insert_slice`/`replace_slice`/`delete_range`, wire canônico do `Slice` (`from_slice_json`), `Editor` e WASM/TS; bordas abertas fundem no bloco hospedeiro | `clipboard/`, `wire/slice.rs`, `tests/clipboard.rs`, `tests/slice_open.rs` |
| 40 | `Transaction::set_block_type` e tabela no `Transaction` | `Transaction::set_block_type` implementado como `Replace` estrutural reversível, com validação de conteúdo/marcas/attrs e preservação do `id`; comandos de tabela continuam em `Editor` (ver linha R) | `state/transaction_structure.rs`, `command/mod.rs`; `tests/block_type.rs` |
| 41 | `ChangeSet.insertedBlocks/removedBlocks` e `Applied` sem doc no WASM | `blocks` cobre a janela afetada, `insertedBlocks`/`removedBlocks` listam índices, `full` só em `ReplaceDoc`; `apply`/`undo`/`redo`/colagem/composição devolvem só `selection`/`version`/`changes`/`history` | `state/change_builder.rs`, `wasm/src/convert.rs`; `tests/changes.rs`, `mod_test.ts` |
| 42 | `ImportOptions`/política de IDs/limites de batch | Lotes aceitam `origin`, `group`, `addToHistory` e `baseVersion` (stale check); `idPolicy`, `idSeed`, `requestId`, `source` e `BatchLimits` continuam em aberto (linha S) | `wasm/src/dto.rs`, `wasm/src/editor.rs`, `wasm/src/batch.rs`; `mod_test.ts` |
| 43 | Composição e fachada `Editor`/WASM | `Editor::composition`, `stored_marks` e os métodos WASM/TS (`beginComposition`/`updateComposition`/`commitComposition`/`cancelComposition`) expõem a composição e um grupo de undo | `state/editor.rs`, `wasm/src/editor/`, `packages/ts/packages/core/mod.ts` |
| 44 | `hard_break` na árvore a11y | Nó existe no schema, mas não tem role e é omitido da árvore; não há role de quebra no contrato | `a11y/build.rs`, `a11y/role.rs` |
| 45 | `Doc::slice`/`insertSlice`/`SliceJSON` ausentes (linha 39) | `Doc::slice`, `Doc::insert_slice`, `Doc::replace_slice`, `Doc::delete_range`, `Slice::size` e wire `from_slice_json`/`Slice::to_json_*`; bordas parciais, merge de blocos e preenchimento de documento vazio | `clipboard/`; `tests/clipboard.rs` |
| 46 | Composição só existia no `EditorState` (linha 43) | `Editor::apply_composition`/`composition_id`/`is_composing`/`stored_marks`; WASM `begin/update/commit/cancelComposition` e `isComposing`; TS espelhado | `state/editor_clipboard.rs`, `wasm/src/editor/methods.rs`; teste de IME como um grupo de undo |
| 47 | Adaptador convertia só intenção de teclado, sem seleção (linha L) | `positions.ts` (bytes UTF-8 ↔ pontos DOM), `selection.ts`, intents de teclado/colagem/IME e live region | `packages/domain/dom`; testes `jsdom` |
| 48 | `axe-core`/`wasm-opt` ausentes (linha M) | `axe-core` roda no `jsdom` em `deno test` (com controle negativo) e `wasm-opt` (binaryen 123) roda no `build_wasm.sh` | `packages/ts/packages/dom/a11y_test.ts`, `scripts/build_wasm.sh` |
| 49 | Fuzz não executado (linha I) | Alvos em `Schema::full`, alvo `round_trip`, corpus de sementes; 60 s por alvo no `make fuzz`/CI; execução local em Docker: 75 840/68 130/61 873/77 422 runs sem crash | `fuzz/`, `fuzz/README.md` |
| 50 | PHP sem `php`/`composer` no host (linha J) | Gate em Docker (`make php`) com validate strict, audit, PHPUnit e PHPStan level max | `packages/php/Dockerfile`, `Makefile` |
| 51 | `make check` vermelho por links para o corpus `ref/` | `check_links.py` ignora o checkout local ignorado pelo git | `scripts/check_links.py` |
| 52 | `schema!` sem a11y/acessores/`trybuild` (linha A) | Gramática de `attrs`/`a11y`, acessores tipados, testes `trybuild` de erro e README | `crates/rich-editor-auy-macros`, `tests/ui`, `README.md` |
| 53 | `validate_a11y` acima de 10 ms (linha D) | `DocPath` movido para o nó e sem `format!`; S2 2,34 ms com `a11y_tree` em 2,15 ms | `a11y/build.rs`, `bench/ULTIMO.md` |
| 54 | Cenários de bench incompletos (linha G) | S0–S4, wire, a11y e `iai_hot` no `bench/`, com relatório e gate de 5% | `bench/`, `bench/ULTIMO.md` |
| 55 | Limites materializavam valores antes da rejeição (linha H) | `Limits` atravessa decode/attrs/marks e `Doc::from_decoded` valida antes de montar | `wire/`, `model/limits.rs`; `tests/parser_limits.rs` |
| 56 | Alocação sem teto calibrado (linha F) | Tetos `≤16`/`≤11`/`≤2` calibrados com o dobro de folga sobre 15/10/1 | `tests/alloc.rs`, `bench/ULTIMO.md` §Alocações |
| 57 | `ImportOptions`/política de IDs/limites e relatório (linhas Q/S) | `idPolicy` com `preserve`/`regenerate`/`reject`, `idSeed` determinístico (`NodeId::derived`), `idMap`, `requestId`, `acceptedBlocks`, fingerprint de schema verificado e limites de payload/pós-import | `wasm/src/ids.rs`, `wasm/src/import.rs`, `wasm/src/dto/import.rs`; `mod_test.ts`, `tests/node_ids.rs` |
| 58 | `openStart`/`openEnd` da fatia | `Doc::slice` marca bordas parciais, o wire codifica/valida `0|1`, a inserção funde bordas abertas no bloco hospedeiro e a álgebra de `Step::Replace` recusa fatia aberta com `open_slice` | `wire/slice.rs`, `clipboard/{selection,insert}.rs`, `step/apply.rs`; `tests/slice_open.rs` |
| 59 | Caminho quente acima do alvo de parede (linha B) | Alvos calibrados com 2× de folga sobre o medido em `plan/05 §11` (S1 ≤ 2,2 µs; S2 ≤ 4 µs); gate real é `iai` 5% em `Ir` + teto de alocações | `plan/05_desempenho.md` §11, `bench/ULTIMO.md`, `tests/alloc.rs` |
| 60 | `check_json` sem alvo (linha C) | Alvo calibrado ≤ 33 ms para S2 (2× os 16,5 ms medidos); `from_json` mantém ≤ 15 ms (12,8 ms medidos) | `bench/wire.rs`, `bench/ULTIMO.md`, `plan/05 §11` |
| 61 | `iai-callgrind` falha com seccomp padrão (linha E) | Verificado em Docker com `--security-opt seccomp=unconfined`: `--save-baseline=main` seguido de `--baseline=main` termina com `Iai-Callgrind result: Ok. 10 without regressions; 0 regressed` | flag documentada em `scripts/bench_instructions.sh` e `AGENTS.md §7`; job `bench` do CI |
| 62 | Pacote WASM exige core publicado (linha K) | Gate local `package-wasm` usa `cargo package --list`; `package-wasm-publish` mantém `--no-verify` para o release após o core; ordem core → macros → wasm no `README` e `plan/08` | `Makefile`; job `wasm` do CI |
| 63 | `wasm-opt`/`b3sum` (linha M) | `b3sum` fixado em 1.8.2 no CI; `AUY_REQUIRE_WASM_OPT=1` no CI e no `build_wasm.sh` (falha se `wasm-opt` faltar); verificado localmente com binaryen 123 | `.github/workflows/ci.yml`, `scripts/build_wasm.sh` |
| 64 | Adaptador de teclado (linha O) | `Mod+Home`/`Mod+End` mapeiam para início/fim do documento; `Home`/`End`/`PageUp`/`PageDown` sem modificador seguem nativos e o `selectionchange` sincroniza a seleção com o core | `packages/ts/packages/dom/{input,mod,adapter_test}.ts` |
| 65 | `source`/`destination` do import (linha T) | `ImportPayload` aceita os dois rótulos e o relatório os devolve (`source`/`destination`, `null` quando ausentes); auditoria completa continua no host | `wasm/src/dto/import.rs`, `wasm/src/convert.rs`; `mod_test.ts` |
| 66 | Open slice sem merge (linha S) | `Doc::insert_slice`/`replace_slice` fundem bordas abertas no bloco hospedeiro (prefixo/sufixo, marcas normalizadas por `TextBlock::new`), com `open_slice_boundary`/`open_slice_kind` para casos não representáveis | `clipboard/open.rs`; `tests/slice_open.rs` |

## Aceitas por decisão (fora deste ciclo)

| # | Área | Aceite | Impacto |
|---|---|---|---|
| N | Adaptador: colagem HTML/Markdown | O adaptador aceita `application/x-rich-editor-auy+json` e cai para `text/plain`; conversão de HTML/Markdown é explicitamente F5 (pós-MVP), com `ImportReport` obrigatório quando existir | Conteúdo HTML/Markdown colado entra como texto; nenhuma perda silenciosa, porque não há promessa de formato |
| R | Comandos de tabela no `Transaction` | Tabela é modelada como operação atômica de `Editor` (`ReplaceDoc`), não como step composto; a álgebra de step para tabela fica para quando houver consumidor | `Transaction` não expõe `insertTable`/`insertRow`; a UI chama `Editor` |

## Regra de atualização

Toda nova divergência entra com comportamento do plano, comportamento real, motivo, impacto e
próximo passo. Quando resolvida, move para a primeira tabela com evidência (teste, gate ou commit).
Nenhuma release pode sair com uma linha “aberta” sem aceite explícito.
