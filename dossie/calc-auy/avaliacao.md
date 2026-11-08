# Avaliação — `calc-auy`

Duas perguntas diferentes, duas colunas. **Como editor** pergunta o que o
usuário experiencia ao escrever texto rico. **Como referência de arquitetura**
pergunta o que o nosso core headless pode aprender com o desenho.

- Versão avaliada: `0.1.0`, commit `08f5a3f` (2026-10-04), licença MPL-2.0.
- Executado: `cargo metadata --offline --no-deps`; `cargo test --workspace
  --locked --offline` (**322 testes, 0 falhas**, 41 alvos); `cargo test -p
  calc-auy-core --lib` (167); `cargo test -p calc-auy-api --doc` (1, após
  liberar disco); ~30 invocações do binário com saída e exit code observados;
  harness Rust de 76 linhas (`exemplos/src/main.rs`) compilado e executado.
  Transcrições em `exemplos/README.md`.
- **Não** executado: clippy, fmt, doc, `cargo deny`, `cargo audit`, bench,
  fuzz, Miri, build wasm32, feature `instrument`. Marcado onde aparece.

## 1. Rubrica (0–5)

`—` = **não aplicável**, com explicação na coluna de comentário. Não é nota
baixa; é ausência da dimensão. Um projeto que não tem superfície de edição não
pode ser avaliado em "neutralidade de estilo de editor".

| Dimensão | Como editor | Como referência de arquitetura | Evidência | Comentário |
|---|---:|---:|---|---|
| **Usabilidade de integração** | — | **4** | `exemplos/src/main.rs` compilou e rodou (6,7 s); `1 + 2` → `3.00`; `--params` desconhecido devolve 2 (`tests/cli.rs:198-204`); exit codes observados (`exemplos/README.md` §3.3) | Init de 3 linhas (`CalcAuy::new` + `parse` + `commit`) e erro claro. Perde o 5 por não haver crate publicado (`Cargo.toml:25` `publish = false`) e por exigir um `CustodyVerifier` mesmo quando não há custódia |
| **API** | — | **4** | `crates/calc-auy-api/src/lib.rs:13-35` (fachada), `crates/calc-auy-builder/src/calculation.rs:27-49` (typestate), `into_rational.rs:7-14` (trait selado), `#[must_use]` em toda a superfície | Pequena e difícil de usar errado (o compilador recusa `f64` e estado inválido). Perde ponto por `pub use calc_auy_core::*` (re-export glob de 30 itens), por `add`/`mul` consumirem `self` e por `mod_` como nome público |
| **Neutralidade de estilo** | — | **3** | `html.rs:39` (`CssMode::Inline` é o padrão) e `:163` (injeta `katex.min.css`); o asset tem **140 680 bytes** (`render/assets/katex.min.css`); `CssMode::External` existe (`html.rs:170`) | Não há tema global nem `style` inline por elemento, mas o modo HTML injeta 140 KB de CSS dentro do fragmento e fixa a classe `calc-auy-result`. O consumidor pode desligar (`External`), mas o padrão interfere |
| **Modelo de dados** | — | **5** | `ast/flat.rs:12-45` (4 variantes), `:74-90` (arena + metadados + profundidades), `ast/types.rs:12-45` (`NodeId` `NonZeroU32`), `canonical/mod.rs:231-284`, `ast/wire.rs:22-33` | Modelo canônico, normalizado (racional sempre irredutível), versionado (`trace_version`) e validável por construção. É o padrão que queremos para o `Doc`: valor, não árvore mutável de renderização |
| **Extensibilidade** | — | **3** | `integrity/custody.rs:36-52` (`CustodyVerifier`), `processor.rs:58-66` (`OutputProcessor`), `crates/calc-auy-i18n/src/lib.rs:163-187` (realizers); adicionar operação toca **30 arquivos** não-teste com `OperationType` (162 ocorrências no total) | Extensão de *comportamento* existe e é limpa (custódia, saída, idioma). Extensão de *vocabulário* não: nova operação exige editar enum, parser, canonical, wire, código de saída e 3 schemas. Não é plugin |
| **Performance** | — | **3** | Desenho: arena (`ast/flat.rs:78-81`), avaliação iterativa (`eval.rs:91-198`), memoização DAG (`eval.rs:74-90`), SoA + tiling (`batch/soa.rs:1-8`, `:33-45`), `rayon` indexado (`parallel.rs:1-25`), `OnceLock` na saída (`output.rs:74-95`). **Medição: não executada** | Desenho correto e sem cópia oculta óbvia, mas **3** pela ausência de benchmark próprio; os "~6,9×" são de `bench/README.md:40-60` + `plan/evidence/bench/compare.md`, evidência do projeto, não desta revisão |
| **Acessibilidade** | — | **3** | `html.rs:138-181` (`aria-label` + escape), `crates/calc-auy-output/src/render.rs:20-45` (descrição verbal do realizer), saída real em `exemplos/README.md` §3.8; SVG **sem** `title`/`desc`/`role` (`svg.rs:88-99`) | Metade do caminho: o HTML tem um rótulo verbal genuinamente útil (não um `alt` genérico), mas SVG/PNG não têm nada. Não há UI, então teclado/foco/leitor não se aplicam — o que existe é semântica de saída |
| **Documentação** | — | **4** | 16 ADRs curtos e decisórios (`docs/adr/`), livro de 8 capítulos, README, CHANGELOG detalhado, `docs/book/src/cli.md:14-34` (tabela de params) | Documenta decisão e consequência, não marketing. Perde ponto por 2 drifts reais: o livro mostra `丸め：nbr5891` e o código imprime `NBR-5891` (`docs/book/src/i18n.md:12-14` vs saída observada); e `schema` anuncia 7 modos omitindo `json`/`html`/`svg` (`commands.rs:316-331`) |
| **Manutenção** | — | **4** | 322 testes; CI com 8 jobs (`.github/workflows/ci.yml:11-95`); lints `deny` de workspace (`Cargo.toml:100-160`); 3 alvos de fuzz; `rust-toolchain.toml:2` | Rigor de processo alto. Perde ponto por: CI usa `dtolnay/rust-toolchain@stable` (`.github/workflows/ci.yml:19-61`) e **nunca testa a MSRV 1.98** que o repositório fixa; clone shallow de 1 commit, sem tags nem release; `TODO` no `SystemClock` |
| **Adequação ao `rich-editor-auy`** | **0** | **4** | Ver §3 | Como editor: 0 por definição — não há documento, cursor, undo, seleção, IME, DOM. Como referência: os pilares (estado em tipos, bytes canônicos, limites na desserialização, erro com `type` estável) são exatamente os nossos |

A coluna "como editor" é `—` em 9 de 10 dimensões porque o projeto **não tem
superfície de edição**: avaliá-lo em "neutralidade de estilo de editor" seria
medir o vazio. A única exceção é a adequação ao nosso caso, onde "como editor"
é um veredito — e o veredito é 0, declarado pelo próprio projeto na primeira
linha do README (`README.md:1-6`: "calculadora", não editor).

## 2. Cobertura da matriz de usabilidade

Status: **C** código · **T** teste · **E** exemplo executado · **D** doc ·
**A** ausência verificada por busca · **NV** não verificado.

| Item | Status | Evidência |
|---|---|---|
| Instalação e pré-requisitos | **E + D** | Dependência por caminho funcionou (`exemplos/Cargo.toml`); `cargo metadata` OK; **não há instalação por crates.io** (`publish = false`, `Cargo.toml:25`); a toolchain 1.98 fixada em `rust-toolchain.toml:2` estava sem `cargo` no ambiente e o build usou 1.99 (`exemplos/README.md` §0) — **MSRV 1.98 não verificada** |
| Inicialização mínima | **E** | CLI `calc-auy "1 + 2"` → `3.00`; harness de API compilou e imprimiu expressão, decimal, monetário, LaTeX, assinatura |
| Edição básica (texto, parágrafo, negrito/itálico/link) | **A + C** | Não existe edição. Busca por `contenteditable`, `Selection`, `clipboard` e por `undo`/`IME` como **palavra inteira** (`grep -riwE`) em `crates/` → 0 ocorrências (sem fronteira, `grep -i undo` casa `segundo`/`fundo`, e `-i ime` casa `time`/`prime`). O análogo é entrada de expressão: `parse("10000 * (1 + 14.75%) ^ 3")` (`parser.rs:350`) |
| Seleção, teclado, undo/redo, clipboard, IME | **A** | Idem: nenhum módulo de interação. O único "teclado" é o `repl` interativo (`commands.rs`, subcomando `repl`) |
| Schema e restrições de conteúdo | **C + T** | AST com 4 variantes e campos fechados por variante (`ast/wire.rs:22-33`; campo desconhecido → erro em `:342`); 5 limites impostos na construção e na leitura (`limits.rs`, `ast/builder.rs:126-165`); 3 schemas versionados em `schema/`; corpus de 32 vetores (`api/tests/corpus.rs`) |
| Custom node/mark/plugin | **C** | Não há plugin. Extensão por trait: `CustodyVerifier` (`custody.rs:36-52`), `OutputProcessor` (`processor.rs:58-66`), `VerbalRealizer` (9 idiomas). Vocabulário novo = editar 30 arquivos não-teste com `OperationType` |
| Persistência e formato de serialização | **E + C + T** | `hibernate`/`hydrate` executados no harness (rascunho JSON → reidratado → `1.33`); wire recursivo com `kind` primeiro (`ast/wire.rs:93-158`); 4 codecs sobre um `WireNode` único (`crates/calc-auy-processors/src/wire.rs:10-66`); 32 vetores de conformidade |
| Renderização e não interferência no CSS | **E + C** | HTML, SVG e JSON executados; HTML auto-contido injeta CSS por padrão e oferece `CssMode::External` (`html.rs:39`, `:163-170`); nenhuma regra global é emitida; SVG é documento isolado (`svg.rs:88-99`) |
| Acessibilidade (ARIA, foco, leitor de tela, editável) | **E + C** | `aria-label` real com descrição verbal em pt-BR (saída em `exemplos/README.md` §3.8; o teste em `crates/calc-auy-output/tests/render.rs:13-23` só verifica a presença do atributo e do `<style>`, não o texto); 9 realizers (`crates/calc-auy-i18n/src/lib.rs:73-93`); SVG/PNG sem rótulo (`svg.rs:88-99`). "Conteúdo editável" **não se aplica** |
| Colaboração | **A** | Busca por CRDT/OT/websocket em `crates/` → 0. Existe log encadeado tipo Merkle (`integrity/merkle.rs:1-20`) para detectar reordenação/remoção de commits — é prova de integridade de auditoria, **não** colaboração |
| Performance e limites conhecidos | **C + D; NV em medição** | Desenho de arena, avaliação iterativa, memoização DAG, SoA/tiling e `rayon` indexado (âncoras na §1). Limites com números explícitos (`limits.rs:34-45`). **Nenhuma medição minha**: bench do projeto em `bench/README.md` e `plan/evidence/bench/`, não reproduzido |
| Erros, migração e diagnóstico | **E + T + C** | Exit codes 2/3/4/5/6/7 observados (`exemplos/README.md` §3.3); `ProblemDetails` com `instance` reprodutível via `MockClock`; `diag` com ids estáveis e custo zero (`diag/mod.rs:1-10`); migração v1→v2 via `SignaturePayloadV2::from_v1` (`integrity/payload.rs:110-125`, teste `:243`). **Não** existe `migrate(vN→vN+1)` genérico que o ADR-013 sugere (`docs/adr/ADR-013-schema-versionado.md:16-18`) |
| Licença e maturidade | **C + D** | MPL-2.0 (`Cargo.toml:24` + `LICENSE`); versão 0.1.0; clone shallow de 1 commit sem tags; CI 8 jobs; `CHANGELOG.md` com entradas extensas e honestas |

**Cobertura: 12 de 13 itens com evidência direta** deste dossiê (execução,
teste, leitura de código ou busca de ausência). O 13º (performance) tem
**desenho verificado e medição não executada** — parcial, não coberto. Itens
"A" contam: num projeto que não é editor, "não existe" é o achado e está
localizado por busca. Com 12/13 plenos e 1 parcial, a matriz passa do patamar
de **≥ 85%** do guia.

## 3. Copiar / adaptar / evitar (resposta direta)

| Pergunta | Resposta |
|---|---|
| **Copiar?** | Nada de código: MPL-2.0 por arquivo, domínio de calculadora, AST aritmética. Copiar arquivos criaria obrigação de licença para ganho zero. |
| **Adaptar?** | Sim, **cinco conceitos**: (1) estados como tipos (`Draft→Committed→Verified` → `EditorState→Applied`/`Verified`); (2) escritor canônico com modo `Trace`/`Value` → `Doc` no hash, `doc_id`/timestamps fora; (3) limites impostos no `DeserializeSeed` → abrir documento de terceiro com orçamento; (4) `ProblemDetails` + exit codes estáveis → adaptador CLI/MCP testável por fora; (5) `Clock`/`MockClock` como porta → determinismo testável sem dormir. |
| **Evitar?** | (1) `publish = false`/toolchain pin sem CI de MSRV; (2) introspecção escrita à mão (`commands.rs:316-331`); (3) anotação de arredondamento embutida em todo formato textual; (4) recusar campo desconhecido como única política de wire (bom para assinatura, ruim para ler documento antigo); (5) relógio de produção fixo com `TODO`; (6) extensão de vocabulário que exige editar 30 arquivos. |

Detalhamento priorizado em `ideias.md`.

## 4. Riscos e limites desta avaliação

| Risco | Situação |
|---|---|
| Performance | **Nenhum número medido por mim.** O desenho é verificável; os "~6,9×/~7,8×" são do projeto (`bench/README.md`, `plan/evidence/bench/`). |
| MSRV 1.98 | Não testada: a toolchain local não tinha `cargo`; tudo rodou em 1.99. |
| WASM | `calc-auy-wasm` tem 10 testes de unidade passando, mas **não** compilei para `wasm32-unknown-unknown` (target ausente). |
| Fuzz/Miri | Alvos lidos (`fuzz/fuzz_targets/{parse,canonicalize,deserialize}.rs`), não executados. |
| Diag/instrument | Feature não compilada; código lido (`diag/mod.rs:98-160`). |
| Benchmark do projeto | Confio no método descrito (`bench/README.md:20-40`), não no resultado. |
| Clone | Shallow, 1 commit, sem tags → maturidade histórica não avaliável. |
| `repl` | Subcomando interativo não exercitado (exigiria PTY). |
| 4 codecs binários | Testes passam (`processors/tests/codecs.rs`, 8); não gerei bytes manualmente para comparar com o schema CDDL. |
| "Não é editor" | Verificado por ausência (buscas citadas) e pela lista de módulos de `calc-auy-core` (`lib.rs:8-18`) — não há documento, seleção, histórico ou view. |

## 5. Confiança

**87%** nas afirmações deste dossiê, após a revisão independente
([`revisao.md`](revisao.md)). Alto: pipeline, modelo de dados,
canonicalização, assinatura, custódia, limites, exit codes, formatos, a11y do
HTML, contagem de testes (322 reexecutada), e o comportamento do binário (~30
invocações com saída colada). Baixo: performance medida, MSRV, WASM em alvo
real, fuzz/Miri e bytes dos codecs binários — todos marcados como não
verificados em vez de inferidos.

O que impede 95%: nenhuma medição própria de performance e o build em 1.99 em
vez da MSRV declarada.
