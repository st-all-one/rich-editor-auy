# Revisão independente — `dossie/calc-auy/`

Revisor: agente independente do autor do dossiê. Alvo: `ref/calc-auy` no commit
`08f5a3f0a907cf503f8b2afbdae0a19f6618f79f` (2026-10-04), versão 0.1.0,
MPL-2.0, clone shallow. Consumidor: engine Rust `rich-editor-auy`.

Regra desta revisão: **não confiar** em nenhuma afirmação do dossiê sem
reexecução, teste ou leitura da origem. Nada em `ref/` foi alterado; o que
mudou está limitado a `dossie/calc-auy/`.

## 0. Método e comandos executados

Toolchain: `rustup` tinha `1.98` instalada **sem `cargo`** (confirmado:
`RUSTUP_TOOLCHAIN=1.98-x86_64-unknown-linux-gnu cargo --version` → erro
"`cargo` binary ... not applicable"; `rustc` 1.98.1 existe). Todos os comandos
usaram `RUSTUP_TOOLCHAIN=1.99-x86_64-unknown-linux-gnu`. `CARGO_TARGET_DIR`
apontou para `/tmp/dossie-calc-review` (link para `/var/tmp`, porque `/tmp` é
tmpfs de 8,7 GB e estava com ~3 GB livres — o dossiê já tinha registrado
`Disk quota exceeded` ali).

Executado nesta revisão:

| Comando | Resultado |
|---|---|
| `cargo test --workspace --locked --offline` | **322 passaram, 0 falharam**, **41 linhas `test result:`** (30 alvos + 11 doctests), `real 6.93s` em cache |
| `cargo test -p calc-auy-core --lib --offline` | **167 passaram**, 2,47–2,51 s |
| `cargo test -p calc-auy-api --doc` | 1 passou |
| `cargo test -p calc-auy-render --doc` | 4 passaram (ver §3, falha transitória) |
| `cargo metadata --offline --no-deps` | **13 pacotes**, todos `MPL-2.0`, `0.1.0`, `publish` vazio (false) |
| `grep -rhoE '^[[:space:]]*#\[test\]' --include=*.rs crates xtask \| wc -l` | **311** (bate com 322 − 11 doctests) |
| `cargo run --offline --locked` em `dossie/calc-auy/exemplos` | compilou e imprimiu o harness inteiro; assinatura `7067199…c831` |
| ~25 invocações do binário | `3.00`; monetário `R$`+NBSP+`15.109,78`; `sign` `82b1836d…724d` (2×); `commit`/`verify` `true`/0 e adulterado `false`/6; exit 3/4/5/6/7/2; `schema`, `man`, `man --completions bash`, `hydrate`, lote NDJSON (4 linhas, exit 0), `error_format=json` determinístico |
| `diff` dos 6 arquivos de `codigo/` contra a origem | **verbatim**, linha a linha: 448, 700, 398, 235, 636 e 225 linhas |
| Script de links relativos em `dossie/calc-auy/**/*.md` | **0 quebrados** |
| `ET.parse` do SVG emitido | documento XML válido, sem `title`/`desc`/`role`/`aria-label` |
| `od -c` das saídas monetárias (CLI, lote, harness) | `R $ 302 240 1 5 . 1 0 9 , 7 8` — separador **U+00A0** |
| Buscas de ausência (`contenteditable`, `Selection`, `clipboard`, `undo`/`IME` com `-w`) | 0 |

Contagens de apoio conferidas na origem: 16 ADRs em `docs/adr/`, 8 entradas em
`docs/book/src/SUMMARY.md`, `clippy.toml` com 178 linhas, 3 alvos de fuzz, 8
jobs no CI (fmt, clippy, test, docs, wasm, deny, audit, miri
`continue-on-error`), 32 vetores em 5 seções do corpus (9/14/4/3/2), 5 limites
(`1_000_000`, `500`, `1000`, `100`, `16_384`).

## 1. O que foi corrigido no dossiê

Correções factuais (sem reescrever estilo):

1. **Separador monetário não era fiel.** `README.md` (§1 e tabela §3) e
   `exemplos/README.md` (§3, §3.5, §4) mostravam `R$ 15.109,78` com espaço
   ASCII. A saída real usa **U+00A0** entre `R$` e o número
   (`crates/calc-auy-output/src/monetary.rs:86`;
   `crates/calc-auy/tests/cli.rs:223` espera `"R$\u{00a0}15.109,78"`). Como o
   arquivo não preserva o byte de forma legível, os blocos mantêm o espaço
   comum e foi acrescentada uma **nota de fidelidade de bytes** em
   `README.md` §1 e `exemplos/README.md` §3, com o `od -c` observado.
2. **Referências de seção inexistentes** em `arquitetura.md` §10.1.1:
   `exemplos/README.md` §2.6-§2.8 e §2.5 → **§3.8** e **§3** (o arquivo de
   exemplos só tem §2.1-§2.4; HTML/SVG e verbal estão em §3).
3. **Atribuição errada do `redact`** em `arquitetura.md` §7.3: o dossiê dizia
   `diag/mod.rs:15-17` e que seria "usado no Mermaid para PII". O correto:
   `pub mod fingerprint; pub mod redact;` em `diag/mod.rs:11-12`, e o
   `redact` de `diag` trata **segredos de log** (`diag/redact.rs:1-4`) — não é
   usado pelo Mermaid, que tem `Privacy::Redacted`
   (`crates/calc-auy-output/src/mermaid.rs:17-31`; saída do livro em
   `docs/book/src/renderizacao.md:55-58`, não 60-66).
4. **ADR-014 atribuído ao encadeamento Merkle** em `ideias.md` R1. O
   ADR-014 é o do **relógio** (`docs/adr/ADR-014-clock.md`); nenhum ADR cobre
   `integrity/merkle.rs` (busca por "Merkle" em `docs/` → 0). Referência
   removida e substituída pela observação explícita.
5. **Faixa de linhas da allowlist do wire**: `ast/wire.rs:22-29` →
   **`22-33`** (as constantes `LITERAL_FIELDS`/`OPERATION_FIELDS`/
   `GROUP_FIELDS`/`CONTROL_FIELDS` vão de 22 a 34) em `arquitetura.md`,
   `avaliacao.md` (2×), `ideias.md` e `codigo/INDEX.md`.
6. **Faixa do `FormatMode`**: `config.rs:34-56` → **`32-54`** (enum começa em
   32 e fecha em 54) em `arquitetura.md` §10.1.1 e `ideias.md` §5.
7. **Âncora do ADR-013**: `:13-16` → **`:16-18`** (a consequência "Funções de
   migração `migrate(vN -> vN+1)` com testes" está na linha 18).
8. **Teste de a11y superestimado**: `crates/calc-auy-output/tests/render.rs:13-23`
   só afirma a **presença** de `aria-label="` e `<style>`; não compara o texto
   verbal. O texto real é evidência de execução (HTML observado). Redação
   ajustada em `avaliacao.md`.
9. **Busca de ausência irreprodutível**: `grep -i undo` casa `segundo`/`fundo`
   e `-i ime` casa `time`/`prime` (6 e 32 arquivos). A afirmação "0
   ocorrências" agora diz explicitamente que é **palavra inteira**
   (`grep -riwE`) — que de fato dá 0, como o dossiê concluía.
10. **Licença em `codigo/INDEX.md`**: o texto dizia que copiar exige preservar
    "o aviso de licença" e que um derivado deve "manter o cabeçalho MPL".
    Nenhum dos seis originais tem cabeçalho SPDX/MPL (a licença está só no
    `LICENSE` da raiz). Redação corrigida para "continuar sob MPL-2.0 e
    declarar isso no topo", sem inventar cabeçalho na origem.
11. **`arquitetura.md` §10.4 ganhou o que faltava sobre `build.rs`/schema**:
    o `.proto` é compilado em `crates/calc-auy-processors/build.rs:11-22` e
    incluído como código prost em `src/lib.rs:31-32`; `.cddl`/`.schema.json`
    saem das constantes de `src/schema.rs` via `xtask schema`
    (`xtask/src/main.rs:435-447`); o teste
    `crates/calc-auy-processors/tests/schema.rs:4-9` compara **só** esses dois
    arquivos — o `.proto` não tem teste de paridade.

Nada foi alterado em `README.md` do repositório, `_GUIA.md`, `_RUBRICA.md`,
`IDEIAS-akribie-calc.md`, `QUADRO-COMPARATIVO.md`, `_PROVENIENCIA.md` nem
`plan/`.

## 2. O que o dossiê não viu (ou viu de menos)

1. **Duas descrições independentes do schema, sem teste entre elas.** O
   `.proto` (usado pelo `build.rs` → prost) e `schema.rs` (constantes
   JSON/CDDL) são mantidos à mão; só o segundo par é checado. Uma operação
   nova ou um campo renomeado pode divergir entre o codec Protobuf e o
   JSON/CDDL sem o CI acusar. O dossiê fala em "3 schemas versionados" mas não
   distingue fonte e projeção, nem o alcance do teste.
2. **Drift do próprio ADR-013**: ele afirma "Processadores gerados de
   `schema/calc_auy.cddl`", mas os codecs são escritos à mão sobre `WireNode`
   (`crates/calc-auy-processors/src/wire.rs:1-9`) e o código gerado vem do
   `.proto` (prost). É drift do projeto, não do dossiê — mas enfraquece a nota
   de documentação como evidência de "geração".
3. **`xtask schema` não é stub, apesar do help.** `Commands::Schema` está
   documentado como "(stub)" no enum de subcomandos, mas `run_schema`
   escreve os arquivos e compara (`xtask/src/main.rs:435-447`). Quem ler só o
   `--help` conclui o oposto do que o código faz.
4. **`diag::redact` × Mermaid** (corrigido em §1.3): a política de PII do
   Mermaid é própria (`Privacy`/`Sensitivity`), não reusa `diag`.
5. **O teste de `<style>`/`aria-label` não valida o texto** (corrigido em
   §1.8): a a11y do HTML é real, mas a prova de conteúdo é a execução, não o
   teste.
6. **`cargo test` completo é estável, mas o linker não.** Numa segunda
   execução do workspace, os 4 doctests de `calc-auy-render` falharam com
   `linking with 'cc' failed`; a execução isolada do mesmo alvo e uma terceira
   execução completa passaram (322/0, 6,93 s). É a mesma classe de incidente
   que o dossiê documenta em `exemplos/README.md` §2.4 (`Disk quota exceeded`),
   e reforça que a falha é do ambiente, não do projeto. Não reescrevi §2.4;
   fica registrado aqui.

## 3. Matriz de usabilidade (13 itens) — estado após a revisão

| Item | Status na revisão | Prova independente |
|---|---|---|
| Instalação e pré-requisitos | **E + D** | path dep compilou; `cargo metadata` 13 pacotes; `publish=false`; **MSRV 1.98 não testada** (1.98 sem `cargo`, confirmado) |
| Inicialização mínima | **E** | binário `1 + 2` → `3.00`; harness compilou/rodou |
| Edição básica | **A + C** | buscas com fronteira → 0; não há documento/edição |
| Seleção/teclado/undo/clipboard/IME | **A** | 0 módulos; só `repl` |
| Schema e restrições | **C + T** | 4 variantes, allowlist 22-33, erro em `:342`, 5 limites, 3 schemas, 32 vetores |
| Custom node/mark/plugin | **C** | traits de custódia/saída/i18n; vocabulário novo = editar enum + parser + wire + schema |
| Persistência e serialização | **E + C + T** | `hibernate`/`hydrate` → `1.33`; 4 codecs sobre `WireNode`; 32 vetores |
| Renderização / não interferência | **E + C** | HTML injeta 140 680 B de CSS por padrão e oferece `External`; SVG isolado |
| Acessibilidade | **E + C** | `aria-label` verbal em pt-BR observado; SVG sem rótulo; teste só de presença; sem UI → foco/leitor não se aplicam |
| Colaboração | **A** | CRDT/OT/websocket → 0; Merkle é integridade de auditoria |
| Performance e limites | **C + D; NV em medição** | desenho lido (arena, iterativo, DAG, SoA, rayon); **nenhuma medição própria** |
| Erros, migração, diagnóstico | **E + T + C** | exit 2/3/4/5/6/7 observados; `instance` determinístico; `from_v1` existe, `migrate` genérico não |
| Licença e maturidade | **C + D** | MPL-2.0; 0.1.0; 1 commit, 0 tags; CI 8 jobs; `SystemClock` com `TODO` |

**12 de 13 com evidência direta; 1 parcial (performance).** Isso satisfaz o
patamar de ≥85% **da matriz de usabilidade**, não a performance medida.

## 4. Confiança final

**87%.** Justificativa:

- Alto (verificado por execução): contagem de testes (322 = 311 + 11 em 41
  alvos), 167 de core, determinismo da assinatura (CLI e harness), exit codes,
  `ProblemDetails` determinístico, HTML/aria, SVG sem a11y, `schema` com 7
  modos contra 10 reais, `publish=false`, clone shallow, MSRV não executável,
  precedência de configuração, lote NDJSON, `hydrate`, `man`, cópias verbatim
  dos 6 arquivos, 0 links quebrados.
- Baixo (não verificado, e o dossiê diz isso): performance medida, MSRV 1.98,
  build `wasm32`, fuzz/Miri, bytes dos 4 codecs contra o CDDL.
- O número não sobe para 90% porque **nenhum eixo de performance foi medido**
  por mim nem pelo dossiê, e porque a MSRV declarada continua não testada.
  Não cai para 80% porque os erros encontrados foram de precisão (âncoras,
  atribuição, fidelidade de bytes), não de tese: pipeline, tipos de estado,
  canonicalização, limites na desserialização e contrato de erro estão de pé.

## 5. Lacunas remanescentes

1. **Performance**: só desenho; `bench/README.md` e `plan/evidence/bench/` são
   evidência do projeto, não reproduzida.
2. **MSRV 1.98**: não há `cargo` para essa toolchain neste ambiente; o CI usa
   `dtolnay/rust-toolchain@stable` e nunca testa 1.98 (fato confirmado em
   `.github/workflows/ci.yml:19-61`).
3. **WASM**: 10 testes de unidade passam, mas não compilei para
   `wasm32-unknown-unknown` (target ausente).
4. **Fuzz/Miri**: alvos lidos, não executados; Miri é `continue-on-error`.
5. **Codecs binários**: testes passam (8), mas não gerei bytes manualmente
   para confrontar com `schema/calc_auy.cddl`/`.proto`.
6. **`repl`**: subcomando interativo não exercitado (exigiria PTY).
7. **Paridade `.proto` × `schema.rs`**: inexistente no projeto e não
   acrescentei teste (fora do escopo: `ref/` não é editável).
8. **Links**: 0 quebrados em `dossie/calc-auy/` (script sobre todos os `.md`).

## 6. Veredito

O dossiê é **majoritariamente correto e executável**, com evidência de
execução onde importa. As correções desta rodada foram de precisão e de
fidelidade (bytes do NBSP, atribuições de ADR/`diag`, referências de seção,
faixas de linha, licença por arquivo, escopo do teste de schema), não de
conclusão. Mantenho a avaliação de que o projeto **não serve como editor**
(0) e **serve como referência de arquitetura** (4), pelos motivos já
registrados em `avaliacao.md` e `ideias.md`.

## 7. Estado do repositório

`ref/calc-auy` continua limpo: `git status --porcelain` no clone não devolve
nada. No repositório do dossiê, `git status` mostra alterações **pré-existentes
e fora do meu escopo** (`.gitignore`, `README.md` da raiz, `plan/` e o próprio
`dossie/` como untracked). Eu escrevi apenas em `dossie/calc-auy/`; os seis
arquivos de `codigo/` permanecem **byte a byte iguais** à origem (`diff` vazio
contra `ref/calc-auy/crates/…`).
