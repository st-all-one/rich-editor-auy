# Changelog

Todas as mudanças relevantes deste projeto são registradas aqui. O formato segue Keep a Changelog
e o projeto usa Versionamento Semântico.

## [Não publicado]

### Adicionado

- Colagem de fatia aberta: `Doc::insert_slice`/`replace_slice` fundem `openStart`/`openEnd` no bloco
  hospedeiro, normalizando marcas na junção; bordas não representáveis falham com
  `open_slice_boundary`/`open_slice_kind`.
- Relatório de importação ecoa `source`/`destination` (auditoria permanece no host).
- Adaptador DOM: `Mod+Home`/`Mod+End` movem o caret para o início/fim do documento; `Home`/`End`/
  `PageUp`/`PageDown` sem modificador continuam nativos, com sincronização por `selectionchange`.
- Importação estruturada com `options.idPolicy` (`preserve`/`regenerate`/`reject`), `idSeed`,
  `idMap`, `requestId`, `acceptedBlocks`, fingerprint de schema verificado e limites
  `maxBytes`/`maxBlocks`/`maxDepth`/`maxNodes`/`maxRunsPerBlock`. `regenerate` reescreve âncoras
  e todo `RefTarget.node` que apontava para elas, em duas passadas; `options.addToHistory` mantém a
  ingestão fora do undo; `maxOps`/`maxTableCells` fecham os limites de lote; `dryRun` valida
  documento, fragmento e ops sem mutar o editor.
- Demo ao vivo em `examples/demo`: página TypeScript com WASM stateful, persistência PHP e
  validação/canonicalização pelo CLI Rust; seed cobre schema completo, mídia, tabela, `raw_code`,
  refs, clipboard, IME e painéis `DocView`/`A11y`/change set. `./examples/demo/run.sh`.
- `validate_a11y` reconhece `table_header` aninhado em `table_row` e não emite `A006` falso para
  tabela com cabeçalho.
- `NodeId::derived` para âncoras determinísticas (BLAKE3, domain separation
  `rich-editor-auy/node-id/v1`).
- `openStart`/`openEnd` no wire de `Slice`: `Doc::slice` marca bordas parciais, o decoder valida
  `0|1` e a álgebra de passos recusa fatias abertas com `open_slice`.
- `ChangeSet` por bloco no core: `blocks` cobre só a janela afetada, `insertedBlocks`/
  `removedBlocks` identificam blocos criados e removidos, e `full` fica restrito a `ReplaceDoc`.
- `Applied` no WASM/TS não carrega mais o documento inteiro: `apply`, `undo`, `redo`, colagem,
  composição e `setSelection` devolvem `selection`, `version`, `changes` e `history`; o documento é
  lido sob demanda por `docJSON`/`stateJSON`.
- `Transaction` no WASM/TS com operações compostas, `origin`, grupo, `addToHistory` e versão base:
  `applyTransaction` consome a transação uma vez e rejeita `stale_batch`/`consumed_transaction`.
  Lotes também aceitam `origin`, `group` e `addToHistory`.
- Comandos de seleção no core/WASM/TS: `insertText`, `deleteBackward`, `deleteForward`,
  `deleteSelection`, `splitBlock`, `insertHardBreak`, `setBlockType`, `toggleMark`, `setNodeAttr`,
  `setInlineAttr`, `setDocAttr`, `setNodeId`, `canUndo`/`canRedo`, `validate`/`validateA11y`.
- `Transaction::set_block_type` no core (substituição estrutural reversível, com validação de
  conteúdo, marcas e atributos) e novos steps de lote `set_block_type`/`replace_slice`.
- Conversões de posição no WASM: `posToUtf16`, `utf16ToPos`, `snapToScalar`, `blockAt`,
  `blockRange`; limites de histórico configuráveis em `fromJSON`/`fromEnvelope`.
- `make check` inclui `package-wasm` (`cargo package --list` offline); publicação do artefato usa
  `package-wasm-publish` depois do core. `build_wasm.sh` exige `wasm-opt` quando
  `AUY_REQUIRE_WASM_OPT=1` e falha se a otimização falhar.
- Envelope durável versão 2 com `migrate_v1_to_full`; envelopes versão 1 continuam legíveis e são
  migrados, e o fingerprint do schema completo é exigido na versão 2.
- `Schema::basic` fica atrás da feature `migration` (fixtures de migração, testkit e benchmark), e
  não faz mais parte da API padrão.
- Clipboard de verdade no core: `Doc::slice` (runs + marcas dentro de um bloco, blocos completos com
  bordas parciais entre blocos, âncoras `id` removidas), `Doc::insert_slice`, `Doc::replace_slice`,
  `Doc::delete_range`, `Doc::same_text_block` e `Doc::caret_after_delete`; wire canônico
  `Slice::to_json_*`/`wire::from_slice_json`.
- Fachada `Editor` para clipboard e IME: `slice`, `insert_slice`, `replace_slice`, `delete_range`,
  `apply_composition`, `composition_id`, `is_composing`, `stored_marks` e `schema`.
- WASM: `copyRange`, `pasteSlice`, `replaceSlice`, `deleteRange`, `cutRange`, `beginComposition`,
  `updateComposition`, `commitComposition`, `cancelComposition` e `isComposing`; tipagens no pacote
  TS `@rich-editor-auy/core`.
- Adaptador DOM completo: mapeamento posição↔ponto DOM em bytes UTF-8 (`positions.ts`), seleção
  nativa ↔ range canônico (`selection.ts`), intents de teclado/colagem/IME (`input.ts`), live region
  educada (`announce.ts`), modo `semantics: "neutral"` e `editor`/`onIntent` em `MountOptions`.
- Testes do adaptador com `jsdom` e `axe-core` (inclui controle negativo que exige uma violação
  real), e teste de IME como um único grupo de undo.
- Fuzz: alvos em `Schema::full`, novos alvos `round_trip` (bytes canônicos estáveis + slice wire) e
  `fragment` (fragmento/slice canônicos) e corpus de sementes em `fuzz/corpus/`; `make fuzz` e o
  job de CI rodam 60 s por alvo.
- TS `Editor`: `selection()`, `block(index)`, `view(range)`, aliases `slice`/`insertSlice` e
  `apply(Transaction)`; `@rich-editor-auy/schema` exporta `SCHEMA_FINGERPRINT`, com teste contra o
  WASM para impedir drift.
- PHP 8.2+: `ProcessRunner` injetável (`Process\SystemProcessRunner` é a única implementação de
  sistema) e `Validator` recebe o port; `ProcessResult` carrega `exitCode`/`stdout`/`stderr`, sem
  arquivos temporários, shell ou `sys_get_temp_dir`.
- PHP 8.2+: `ValidationReport` com `issues`, `diagnostics` (`error:`/`usage:`) e `unrecognized`
  cru — nenhuma linha do CLI vira erro de validação inventado; construtor privado, parse único.
- PHP 8.2+: exceções tipadas `InvalidDocument`, `IncompatibleEnvelope` e `TransportError`.
- Gate do PHP em Docker (`make php` → `packages/php/Makefile`): imagens fixadas por digest, matriz
  PHP 8.2/8.3/8.4/8.5 (glibc) + 8.2 (musl), `composer validate --strict`, Mago (lint + format),
  PHPStan level max com strict/deprecation/PHPUnit/dead code, PHPUnit 11.5 estrito, cobertura de
  statements 100% e smoke de consumidor.
- `Schema::doc_filler` e `Slice::size` como API pública documentada.

- Schema completo (`Schema::full`) com tabela, figura/`image_block`, `raw_code` textual, metadados,
  referências, atributos tipados (`Enum`/`List`/`Map`) e projeções a11y (`CodeBlock`, `code_language`).
- `ProblemDetails` RFC 9457 para tokens desconhecidos/não representáveis, com `code`, `instance` e
  sugestão de conversão (gráfico/fórmula → imagem; page break → erro; markup bruto → `raw_code`).
- `Doc::table_info`/`Doc::cell` e validação de grade de tabela (I17–I25), incluindo spans, headers
  e seleção `Selection::Table` com comandos de tabela.
- `Editor` stateful com versão, seleção, histórico, undo/redo, `ReplaceDoc`, fragmentos e comandos
  de tabela; WASM expõe a classe `Editor`, ingestão em lote e erros `ProblemDetails`.
- `DocView` render-oriented no WASM/TS: nomes, ranges, runs, atoms, atributos, filhos e versão,
  separado da árvore a11y; CLI e WASM usam `Schema::full()`.
- Mídia completa no schema (`mime`, `size_bytes`, dimensões/intrínsecas em vídeo/anexo), listas
  ordenadas projetam `ordered`, e URLs `data:`/`javascript:` são rejeitadas na decodificação.
- Steps `ReplaceDoc`, `SetInlineAttr` e `SetMarkAttr`, com inversão exata; composição/histórico
  agrupado, `HistoryInfo`, clear e limites.
- Benchmark A/B `bench/examples/stateful.rs`: ~69× por edição sem JSON e ~18,7× no round-trip completo.
- Workspace Rust `rich-editor-auy` com core `no_std`, macros, bindings WASM e testkit.
- Modelo canônico de blocos, schema builder, expressões de conteúdo, posições e wire JSON.
- Fingerprint BLAKE3-256 e envelope durável.
- Árvore semântica de acessibilidade, validação com códigos estáveis e projeção JSON canônica.
- Steps estruturais: join de blocos, inversão de split e slices de fragmento.
- Índice ordenado de início de blocos (`Doc::block_at`) para consultas locais.
- API WASM para split, delete, mark, join e árvore de acessibilidade.
- Adaptador DOM semântico sem `class`/`style`, com renderização de headings, listas, links, mídia e
  tabelas, e restauração do host em `destroy()`.
- Pacotes Deno/JSR com testes; pacote Composer com validação estrita de envelope.
- CLI `diff` e `a11y-tree`.
- Testes de alocação, memória, visitas; benchmarks `criterion`; alvo de fuzz; exemplos TS/PHP/host.
- Gate local `make check`, `make visits`, `make check-ecosystem` e pipeline de CI com wasm/js/php/bench/fuzz.
- Registro de divergências em `DIVERGENCES.md`.

### Alterado

- `schema!` deixou de ser placeholder: proc-macro real para doc, nós, marcas, grupos, conteúdo e
  atributos tipados, com teste de integração e expansão `no_std`.
- Parser JSON decodifica surrogate pairs, reporta offsets de origem nos erros e aplica os limites
  de attrs, marcas, runs e texto no documento final.
- A árvore de acessibilidade carrega `DocPath` e a validação usa o path real do documento.
- Substituições de bloco no caminho quente reutilizam o índice de inícios e revalidam apenas o
  bloco alterado.
- Blocos de topo passaram a ser armazenados em chunks de 128; S2 caiu de ~106 µs para ~3,1 µs.
- Documentos com até 128 blocos usam um único chunk flat; S1 caiu de ~2,05 µs para ~1,73 µs.
- `apply_owned` muta o snapshot quando ele é único; insert de 1 char caiu de 35 para 25 alocações.
- Inversão extrai um intervalo com um único `slice_text`.
- Parser reutiliza uma pilha de paths e escaneia strings sem escape por slice; `from_json` S2 caiu de ~34,8 ms para ~21,2 ms e `check_json` para ~8,9 ms.
- Writer pré-calcula capacidade e copia trechos sem escape de uma vez; `to_json_string` S2 caiu de ~2,97 ms para ~1,12 ms.
- A11y move o `DocPath` para o nó em vez de clonar; `validate_a11y` caiu de ~11,34 ms para ~11,04 ms.
- Decoder aceita attrs de marca no objeto canônico `attrs` (round-trip do writer) e mantém campos planos por compatibilidade.
- Benchmarks `bench/examples/all_types.rs` e `bench/reference/` comparam AUY com ProseMirror e Lexical; relatório em `bench/COMPLEX-COMPARISON.md`.
- `schema!` ganhou split em módulos, gramática de `attrs`/`a11y`, acessores tipados e testes
  `trybuild` de erro de compilação; o subconjunto está documentado no README.
- `make package-wasm` usa `--no-verify` (a crate depende de crates ainda não publicados) e continua
  fora de `make check`; o CI instala `binaryen` e `b3sum` e publica o digest do artefato.
- PHP: `Envelope` tem construtor privado e `Envelope::fromArray` valida formato, versão, fingerprint
  e `doc_id`; nenhum envelope inválido existe. `Document::fromBytes` rejeita topo lista, `doc`
  ausente/vazio e propaga `IncompatibleEnvelope` antes de checar `doc`.
- PHP: o pacote passa a exigir `php ^8.2` + `php-64bit` + `ext-json`, com `composer.lock`
  versionado, `.editorconfig`/`.gitattributes`/`.dockerignore` e política de licença dual
  (Apache-2.0 + MIT) dentro do pacote.
- PHP: `Validator::run()` (execução bruta) saiu da API pública; a execução é do port `ProcessRunner`
  e `Validator` expõe apenas `validate()`.

### Corrigido

- Faixa invertida (`start > end`) é erro em todo consumidor: `Range::is_empty` só é verdadeiro em
  `start == end`; `AddMark`/`RemoveMark`/`SetMarkAttr`/`SetInlineAttr` rejeitam com `OutOfBounds`
  (antes viravam no-op silencioso e `add_mark` ainda guardava a marca); `Selection::range`
  normaliza âncoras antes de expor a faixa. Contrato em `tests/ranges.rs`.
- Inversão de múltiplos steps, `StepMap` de split, preservação de marcas em `Slice::runs`, remoção
  de attrs por `Null`, validação estrutural no wire, allowlist de URLs no DOM e envelope no CLI/TS.
- PHPStan level max: `Document`/`Envelope` declaram shapes de array e o decode rejeita listas.
- PHP: `doc_id` com newline final deixou de ser aceito (`\A[\x21-\x7e]{1,128}\z` em vez de `$`).
- `check_links.py` não trata o corpus local `ref/` (ignorado pelo git) como doc quebrada.

### Removido

- `basicSchema()`/`documentSchema()` do pacote TypeScript: existe um único preset público,
  `fullSchema()`, e todos os caminhos WASM usam `Schema::full`.
