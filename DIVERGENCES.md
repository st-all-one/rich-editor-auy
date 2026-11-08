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
| 8 | `apply_owned` deveria evitar a cópia do snapshot | `apply_owned` delega para `apply` (COW), que copia o vetor de blocos tocado | Semântica idêntica; desempenho registrado abaixo |
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

## Divergências abertas

| # | Área | Estado | Impacto | Próximo passo |
|---|---|---|---|---|
| A | `schema!` | Front-end real, mas cobre doc/nó/marca/content/group/marks/attrs básicos; sem acessores tipados e sem testes compile-fail | Extensões como `a11y` declarativa e `trybuild` ainda não têm contrato | Adicionar a11y declarativa, acessores e `trybuild`; documentar subconjunto |
| B | `apply_owned` | Não é mutação in-place; `replacing_block` já evita revalidar o documento inteiro | S2 ~107,5 µs para 1 char contra alvo de 10 µs | Mutação in-place/`Arc::get_mut`/chunking (D06) |
| C | Parser JSON | S2 ~35,6 ms contra alvo de 15 ms; surrogate pairs resolvidos; `JsonValue` carrega offset inicial, mas spans completos de campos ainda não | Parse acima do orçamento; offsets são do valor, não do token exato | Perfilar alocações e carregar intervalo completo por valor |
| D | `validate_a11y` | S2 ~11,4 ms contra alvo de 10 ms | Validação acima do orçamento | Revisar alocações por nó/cachear árvore no adaptador |
| E | `iai-callgrind`/baseline | Não instalado; `criterion` é informativo | Sem gate determinístico de instruções/regressão de 5% | Adicionar job com valgrind e `bench/baseline.json` |
| F | Alocação do caminho quente | Inserção de 1 char = 20 alocações; alvo do plano é 0 além do step | Custo acima do alvo | Otimizar `Doc::new`/starts/inversão e recalibrar |
| G | Cenários de bench | Só S1/S2; faltam S0/S3/S4 e operações delete/mark/split/join/undo | Cobertura de `plan/05` incompleta | Adicionar `benches/hot.rs`/`wire.rs` e rodar S0–S4 |
| H | Limites | Aplicados em `Doc::new`/`replacing_block` para attrs, marcas, runs, texto e strings; o decode ainda pode materializar valores antes da rejeição | Estrutura acima do orçamento é rejeitada, mas memória pode ser alocada antes | Mover a checagem para `decode_attrs`/`decode_marks` e testar o limite de attrs por nó |
| I | Fuzz | Alvos existem, mas não foram executados neste host (sem `cargo-fuzz`/nightly) | Cobertura de parser/a11y não exercitada localmente | Rodar no CI e anexar corpus |
| J | PHP | `php`, `composer`, PHPUnit e PHPStan ausentes no host | Pacote não executado; validação só por inspeção | CI `php` executa validate/audit/PHPUnit/PHPStan |
| K | WASM package | `cargo package` exige core publicado no crates.io | `make check` publica só core/macros; `make package-wasm` separado | Release ordenado core → macros → wasm |
| L | DOM adapter | Renderiza a árvore semântica e restaura o host; edição é intenção via `onInput`, sem aplicar DOM | Host precisa chamar comandos do core; seleção/IME/clipboard não são convertidos automaticamente | Mapear seleção/entrada no adaptador ou declarar contrato no README |
| M | `axe-core`/`wasm-opt` | Ausentes no host | Auditoria de DOM e otimização final do WASM não comprovadas | Jobs de navegador/release |


## Regra de atualização

Toda nova divergência entra com comportamento do plano, comportamento real, motivo, impacto e
próximo passo. Quando resolvida, move para a primeira tabela com evidência (teste, gate ou commit).
Nenhuma release pode sair com uma linha “aberta” sem aceite explícito.
