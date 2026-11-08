# Brainstorm 02 — Bordas, invariantes e o teste que trava cada uma

> **Status:** catálogo de bordas. Vira o `DIVERGENCES.md` do projeto quando houver código. Regra:
> toda borda listada aqui tem uma **mitigação** e um **teste** que a trava; se o teste não existe,
> a mitigação é promessa, não garantia. Os nomes de teste são propostas para F1–F4.

---

## 1. Unicode e texto

| # | Borda | Risco | Mitigação | Teste (a criar) |
|---|---|---|---|---|
| U01 | **Fronteira de escalar UTF-8** | `Pos` no meio de um caractere; slice panic | `Pos` só é construído/validado em fronteira de escalar; `text::slice` devolve `Option`/`Result` | `pos_rejects_non_scalar_boundary` |
| U02 | **Grapheme cluster** | caret/delete no meio de `e` + combining acute, emoji ZWJ, bandeira, tom de pele | `motion::snap_to_grapheme`; delete/move usam grapheme; o core não divide grapheme em edição de alto nível | `delete_backward_removes_whole_grapheme`, `caret_snaps_to_grapheme` |
| U03 | **Marcas combinantes no início** | texto começa com combining mark; caret antes/depois | `snap_to_grapheme` trata como cluster; não normaliza | `leading_combining_mark_is_one_cluster` |
| U04 | **ZWJ / ZWSP / variação** | emoji composto, zero-width space | preservar bytes; motion/segmentação decide; não colapsar | `zwj_sequence_is_not_split` |
| U05 | **Bidi** | texto RTL/LTR misturado; ordem visual vs lógica | o core guarda ordem lógica; `dir` é atributo; a UI faz bidi visual | `bidi_text_keeps_logical_order` |
| U06 | **CRLF/CR** | bytes diferentes para o mesmo texto visual | normalizar `\r\n`/`\r` para `\n` na importação; documentar que o core usa `\n` | `import_normalizes_line_endings` |
| U07 | **Tab** | largura variável; posição visual | armazenar `\t`; a UI expande; o core não tem colunas | `tab_is_a_single_text_byte` |
| U08 | **NBSP / espaços especiais** | colapso indevido | preservar bytes; nenhuma normalização de whitespace no texto | `nbsp_is_preserved` |
| U09 | **NFC vs NFD** | `café` composto vs decomposto serem docs diferentes | **não normalizar** texto (mudaria o conteúdo do usuário); documentar a borda; busca/import podem normalizar em cópia | `nfc_and_nfd_are_distinct_documents` |
| U10 | **Surrogate isolado em JSON** | `\uD800` sem par | parser rejeita `ParseError::InvalidEscape` | `json_rejects_unpaired_surrogate` |
| U11 | **UTF-8 inválido** | impossível em `&str`, mas bytes de storage | `from_bytes` valida `str::from_utf8` e devolve erro | `from_bytes_rejects_invalid_utf8` |
| U12 | **Contagem de tamanho** | bytes vs chars vs UTF-16 | `Size` documenta bytes; conversões explícitas para UTF-16/chars | `size_matches_utf8_len`, `utf16_conversion_round_trip` |
| U13 | **Texto vazio** | run vazio, textblock vazio | run vazio não existe; textblock vazio é permitido se o schema permitir | `empty_run_is_normalized_away` |
| U14 | **Texto gigante** | bloco > 1 MiB | limite + runs de 4 KiB; política de split documentada | `block_text_limit_is_enforced` |

## 2. Posições, seleção e mapeamento

| # | Borda | Risco | Mitigação | Teste (a criar) |
|---|---|---|---|---|
| P01 | **Doc vazio** | posição 0 inválida para caret; root sem bloco | schema `block+` preenche um parágrafo vazio; caret vai para 1 | `empty_doc_has_one_paragraph` |
| P02 | **Caret em 0** | 0 é o token de abertura, não texto | `resolve_caret(0)` normaliza para o primeiro textblock | `caret_at_zero_normalizes_to_first_text` |
| P03 | **Caret em `size`** | posição após o último bloco | normaliza para o fim do último texto | `caret_at_end_normalizes_to_last_text` |
| P04 | **Seleção entre blocos** | marcas em runs de blocos diferentes | aplicar marca por textblock; não marcar blocos | `mark_across_blocks_applies_per_run` |
| P05 | **Seleção de nó** | átomo/bloco selecionado; delete substitui o nó | `Selection::Node`; delete substitui por slice vazio | `delete_node_selection_removes_node` |
| P06 | **Seleção all** | range 0..size inclui tokens | normalizar para conteúdo do root | `select_all_covers_root_content` |
| P07 | **Associação em inserção** | posição no ponto de inserção pode ficar antes/depois | `assoc -1/+1` em `Selection`; teste nas bordas | `assoc_controls_boundary_position` |
| P08 | **Posição deletada** | mapear para dentro de conteúdo removido | `MapResult::deleted`; seleção clampa para a borda | `deleted_position_clamps_to_edge` |
| P09 | **Mapeamento inverso** | undo/redo perder a posição | `Mapping` com mirror/recover; proptest de ida/volta | `map_invert_round_trip_for_positions` |
| P10 | **Posição de outro doc** | `Pos` é u32; usar no doc errado | documentar como erro de lógica; `resolve` valida limites; opcional `DocId` em debug | `resolve_rejects_out_of_bounds` |
| P11 | **Seleção stale** | UI guardou seleção de estado antigo | mapear ou descartar; `Selection::map` | `stale_selection_must_be_mapped` |
| P12 | **Range invertido** | `from > to` | `Range::new` normaliza ou erro; testar | `range_rejects_inverted` |
| P13 | **Limite de profundidade** | 64 níveis; stack overflow | travessia iterativa; limite no schema/parse | `depth_limit_is_enforced` |

## 3. Estrutura e schema

| # | Borda | Risco | Mitigação | Teste (a criar) |
|---|---|---|---|---|
| S01 | **Expressão de conteúdo não produtiva** | `block+` onde todo block exige block | checagem estática na construção; erro `UnproductiveContent` | `schema_rejects_required_cycle` |
| S02 | **Autofill recursivo** | primeiro tipo é recursivo | mesma checagem; filler escolhido entre alternativas terminantes | `filler_is_terminating` |
| S03 | **Contêiner obrigatório vazio** | list_item sem paragraph | autofill determinístico; se impossível, erro | `empty_container_gets_filler` |
| S04 | **Split no início/fim** | bloco vazio antes/depois | permitido se o schema permitir; conteúdo vazio normalizado | `split_at_start_creates_empty_block` |
| S05 | **Join de tipos diferentes** | heading + paragraph | política explícita: converter para o tipo do alvo ou recusar; testar | `join_incompatible_types_is_defined` |
| S06 | **Wrap/lift** | estrutura muda, posições mudam | `ReplaceAround`; mapping cobre o gap | `wrap_and_lift_preserve_content` |
| S07 | **Lista aninhada** | indentar/desindentar | modelo de blocos aninhados; comandos testados | `nested_list_round_trip` |
| S08 | **Tabela** | célula com blocos; spans | pós-MVP; modelo permite blocos aninhados; teste de schema | `table_shape_round_trip` |
| S09 | **Átomo em borda** | caret antes/depois; delete | átomo ocupa 1 posição; seleção de nó | `atom_boundary_positions` |
| S10 | **Marcas não permitidas** | mark em nó que não aceita | normalização remove; add_mark falha ou ignora com erro | `mark_not_allowed_is_rejected` |
| S11 | **Attr obrigatório sem default** | autofill não consegue criar nó | schema rejeita em posição preenchível | `required_attr_without_default_is_rejected` |
| S12 | **Duplicata de nome** | dois nós com mesmo nome | `SchemaError::DuplicateName` | `duplicate_node_name_is_rejected` |
| S13 | **Grupo desconhecido** | content expr cita grupo inexistente | erro na construção | `unknown_group_is_rejected` |
| S14 | **>64 marcas** | máscara u64 estoura | schema rejeita; mensagem clara | `too_many_marks_is_rejected` |
| S15 | **Mark `excludes` a si mesma** | conflito eterno | schema rejeita | `mark_cannot_exclude_itself` |
| S16 | **Attr default de tipo errado** | default string em attr int | schema rejeita | `attr_default_type_must_match` |
| S17 | **Fingerprint instável** | ordem de declaração muda o hash | fingerprint inclui ordem; teste em duas construções iguais | `fingerprint_is_stable` |
| S18 | **Schema mismatch no load** | doc de outro schema | `SchemaMismatch { path, kind }` com os dois fingerprints | `load_rejects_schema_mismatch` |

## 4. Marcas e runs

| # | Borda | Risco | Mitigação | Teste (a criar) |
|---|---|---|---|---|
| M01 | **Toggle com seleção vazia** | nada acontece | `stored_marks` no estado; digitar usa | `toggle_mark_with_caret_sets_stored_marks` |
| M02 | **Toggle com seleção mista** | metade tem bold | se todos têm, remove; senão adiciona | `toggle_mark_mixed_selection` |
| M03 | **`inclusive`** | digitar no fim de um link dentro/fora | `inclusive` do schema decide; teste nas duas bordas | `inclusive_mark_extends_on_typing` |
| M04 | **`excludes`** | link dentro de link; code + bold | remover marcas excluídas ao adicionar | `exclusive_marks_remove_each_other` |
| M05 | **Igualdade com attrs** | dois links com href diferentes | `Mark` igual inclui attrs; canonicalizar | `mark_equality_includes_attrs` |
| M06 | **Remoção de marca** | run no meio | dividir runs e remover; normalizar | `remove_mark_splits_runs` |
| M07 | **Marca em átomo** | imagem com bold | schema decide; default não | `atom_marks_follow_schema` |
| M08 | **Marca entre blocos** | seleção atravessa | aplicar por textblock; sem marca em bloco | `cross_block_mark_is_per_textblock` |
| M09 | **Ordem de marcas** | duas ordens para o mesmo conjunto | ordem do schema; `MarkSet` canônico | `mark_order_is_schema_order` |
| M10 | **Run no teto de 4 KiB** | texto adjacente com mesma marca mas dois runs | canonical split determinístico; UI mescla | `run_split_is_canonical_at_limit` |

## 5. Steps, transform e histórico

| # | Borda | Risco | Mitigação | Teste (a criar) |
|---|---|---|---|---|
| T01 | **Step que não muda nada** | histórico poluído | `Transform` descarta no-op; `apply` aceita mas não registra | `noop_step_is_not_recorded` |
| T02 | **Replace com slice aberto inválido** | open depth > árvore | validar; `StepError::InvalidSlice` | `open_slice_too_deep_is_rejected` |
| T03 | **Replace do doc inteiro** | root vazio | autofill respeita schema; se não, erro | `replace_entire_doc_respects_root` |
| T04 | **Inversão após normalização** | inverse calculado no doc errado | calcular inverse contra o doc normalizado; proptest | `inverse_uses_normalized_doc` |
| T05 | **Map através de deleção** | step inteiro deletado | `Step::map` devolve `None`; transação descarta | `step_fully_deleted_maps_to_none` |
| T06 | **Rebase de steps sobrepostos** | pós-MVP | `Mapping` com mirror/recover; testes de convergência | `rebase_overlapping_steps` |
| T07 | **Falha no meio da transação** | estado parcial | `apply` atômico; `apply_owned` restaura inversos | `transaction_failure_is_atomic` |
| T08 | **Transação só de seleção** | entra no undo | `add_to_history=false` se não há steps | `selection_only_tx_is_not_undoable` |
| T09 | **Undo após redo** | redo stack inconsistente | novo edit limpa redo | `new_edit_clears_redo` |
| T10 | **Undo de mark** | bytes não voltam | inverse Add/RemoveMark; golden | `undo_mark_restores_bytes` |
| T11 | **Undo de estrutura** | wrap/lift | inverse ReplaceAround; golden | `undo_wrap_restores_bytes` |
| T12 | **Grupo de digitação** | cada tecla vira undo | `GroupId` + política; teste de rajada | `typing_burst_is_one_undo_group` |
| T13 | **IME** | cada update vira undo | `Origin::Composition`; commit consolida | `composition_is_one_undo_group` |
| T14 | **Cancelar composição** | texto parcial fica | restaurar original; testar | `composition_cancel_restores_text` |
| T15 | **Histórico sem memória** | deleção de 1 MiB | orçamento + evicção; testar | `history_evicts_oldest_group` |
| T16 | **Step serializado desconhecido** | load quebra | `ParseError::UnknownStep`; sem panic | `unknown_step_type_is_rejected` |

## 6. Wire (JSON canônico)

| # | Borda | Risco | Mitigação | Teste (a criar) |
|---|---|---|---|---|
| W01 | **Ordem de chaves** | bytes instáveis | ordem do schema/emitida fixa | `json_key_order_is_canonical` |
| W02 | **Escapes** | `"`/`\`/controle | escape mínimo; `/` não | `json_escapes_minimally` |
| W03 | **Unicode cru** | `á` virar `\u00e1` | UTF-8 cru; teste de bytes | `json_keeps_raw_utf8` |
| W04 | **Inteiros** | `1.0`/`1e3` | parser rejeita float; writer só int | `json_rejects_floats` |
| W05 | **Campos desconhecidos** | forward compat | estrito por default; `lossy` com report | `unknown_field_is_rejected_by_default` |
| W06 | **Chaves duplicadas** | ambiguidade | parser rejeita | `duplicate_json_key_is_rejected` |
| W07 | **Dados após o objeto** | lixo silencioso | parser rejeita trailing | `trailing_json_is_rejected` |
| W08 | **`attrs: null`** | attr inválido | omitir; `null` só se o schema declarar `Null` | `null_attr_is_rejected_unless_declared` |
| W09 | **`marks: []`** | ruído | omitir | `empty_marks_is_normalized_away` |
| W10 | **Texto vazio** | nó inútil | omitir/normalizar | `empty_text_node_is_normalized_away` |
| W11 | **Runs adjacentes iguais** | duas representações | fundir até 4 KiB | `adjacent_equal_runs_are_merged` |
| W12 | **Ordem de marcas** | duas representações | ordem do schema | `marks_are_emitted_in_schema_order` |
| W13 | **Profundidade** | 64 níveis; stack overflow | limite + parser iterativo | `json_depth_limit_is_enforced` |
| W14 | **Array gigante** | OOM | limite de nós antes de alocar | `json_node_limit_is_enforced` |
| W15 | **Envelope** | schema errado | versão + fingerprint; erro | `envelope_rejects_wrong_fingerprint` |
| W16 | **Versão do envelope** | formato futuro | erro claro; sem “tentar mesmo assim” | `envelope_rejects_unknown_version` |

## 7. Limites e input hostil

| # | Borda | Risco | Mitigação | Teste (a criar) |
|---|---|---|---|---|
| L01 | **Doc > 64 MiB** | OOM | `LimitError` antes de alocar | `doc_size_limit_is_enforced` |
| L02 | **Bloco > 1 MiB** | memmove/ruído | limite; política de split documentada | `block_text_limit_is_enforced` |
| L03 | **Profundidade > 64** | stack | limite no parse/apply | `depth_limit_is_enforced` |
| L04 | **Nós > 1M** | memória | limite | `node_limit_is_enforced` |
| L05 | **Attrs > 32** | memória | limite | `attr_limit_is_enforced` |
| L06 | **Attr string > 64 KiB** | memória/hash | limite | `attr_string_limit_is_enforced` |
| L07 | **Runs > 4096** | scan | impossível com 4 KiB/1 MiB; validar | `run_limit_is_enforced` |
| L08 | **JSON bomb** | aninhamento/arrays | limites antes de alocar; `try_reserve` | `fuzz_json_never_oom` |
| L09 | **Slice hostil** | profundidades/conteúdo inválido | validar; erro tipado | `hostile_slice_is_rejected` |
| L10 | **Step hostil** | posições fora do doc | validar; erro tipado; fuzz | `fuzz_apply_step_never_panics` |

## 8. Concorrência, snapshots e API

| # | Borda | Risco | Mitigação | Teste (a criar) |
|---|---|---|---|---|
| C01 | **Snapshot durante edição** | COW copia; doc antigo válido | `Arc`; teste de strong_count/bytes | `snapshot_survives_edit` |
| C02 | **Resultado de background stale** | decoração em range antigo | mapear pelo `Mapping` ou descartar | `stale_decoration_is_mapped_or_dropped` |
| C03 | **`Send + Sync`** | tipo público deixa de ser | `assert_send_sync::<Doc>()` etc. | `public_types_are_send_sync` |
| C04 | **`Rc`/`RefCell` acidental** | quebra thread-safety | lint `disallowed_types` | `clippy_gate` |
| C05 | **`Pos` stale** | usar posição antiga | documentar; `resolve` valida; mapear | `stale_pos_must_be_mapped` |
| C06 | **Índice de bloco stale** | split/join muda índice | usar `ResolvedPos`/range | `block_index_is_not_identity` |
| C07 | **Run boundary stale** | UI assumiu um run | change set + ranges | `ui_must_use_change_set` |
| C08 | **`Debug` como wire** | formato instável | `Debug` é diagnóstico; wire é `to_json` | `debug_is_not_wire` |

## 9. Regras de ouro derivadas

1. **Toda posição é validada no uso, não na construção.** `Pos` é um `u32`; `resolve`/slice
   devolvem `Option`/`Result`. Não confie em posição de outro doc.
2. **Toda borda de run/bloco é um teste.** Split/join/mark nas bordas é onde os bugs moram.
3. **Toda conversão de unidade é explícita.** Bytes, UTF-16, chars e graphemes não se misturam.
4. **Toda mudança estrutural é atômica.** Se falhou, o estado original está intacto.
5. **Todo wire é canônico.** Se dois docs produzem bytes diferentes para o mesmo conteúdo, é bug.
6. **Todo input hostil é um `Result`.** Nenhum panic em parse/apply.
7. **Toda performance é medida.** “Parece rápido” não é evidência.
8. **Toda exceção de lint é local e justificada.** Sem `#[allow]` de workspace.
