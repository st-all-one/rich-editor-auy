# akribie — avaliação

> Rubrica 0–5 do [`_RUBRICA.md`](../_RUBRICA.md) e matriz de 13 itens do [`_GUIA.md`](../_GUIA.md),
> aplicadas ao commit `7275fedfe81f2d4ac1df4e92a580c05b90a81049` (v0.1.3, 2026-11-03).
> Legenda de status: **E** = verificado por execução · **C** = verificado no código ·
> **T** = verificado em teste (lido e executado na suíte) · **D** = verificado em doc do repo ·
> **X** = não verificado · **—** = não aplicável (explicado).
>
> Toda nota e todo item têm `caminho:linha` ou saída de comando. Nada aqui foi inferido de README.

---

## 1. Método e o que foi executado

Tudo em `ref/akribiechart`, sem alterar `ref/`. Artefatos em `/tmp/dossie-akribie-target`.

```sh
cargo metadata --offline --no-deps                    # ok, 4 pacotes + xtask excluído
cargo test --workspace --locked                       # 230 passaram, 0 falhas, 0 doc-tests
cargo build -p akribie-cli                            # ok → target/debug/akr
cargo build -p akribie-core --no-default-features      # FALHA: E0433 (ver §1.2)
cd exemplos/harness && cargo run --offline --quiet     # ok, saída colada em exemplos/README.md
```

E, contra o binário real, com `fixtures/retry.akrc` e arquivos gerados:

| comando | resultado |
|---|---|
| `akr check retry.akrc` | silencioso, `exit 0` |
| `akr check broken.akrc` | 2 erros + 1 aviso com linha/coluna/`hint`, `exit 8` |
| `akr check broken.akrc --json` | envelope com `success:false` + `error.code:"schema"` |
| `akr render retry.akrc` (2×) | SVG idêntico, sha256 `0f98809a…` |
| `akr render --format scene` / `excalidraw` (2×) | idênticos, `ca83980b…` / `a02bf55e…` |
| `akr render --format png --scale 2` (2×) | `retry.png` idênticos, `99a56460…`, 1672×344 RGBA |
| `akr fmt messy2.akrc` | forma canônica; `fmt(fmt(x)) == fmt(x)` (md5 igual) |
| `akr fmt --check` (não canônico / canônico) | `exit 1` / `exit 0` |
| `akr diff retry.akrc novo.akrc` | `- edge retry->api "backoff"`, `~ node retry shape: cloud -> rounded`, `exit 1` |
| `akr schema` | JSON com `akribie_version`, `diagnostics[]`, `contrast` |
| `akr prime` / `--long` | 2 742 / 6 110 bytes |
| `akr check --batch lote.jsonl` | 4 linhas, `exit 0` mesmo com 2 em erro |
| `akr render --batch --out-dir saida` (sem `mkdir`) | `exit 3` |
| `akr import retry.excalidraw` | `.akrc` com `layout=free` e todos `at=`/`size=` |
| `akr-mcp` com 4 mensagens JSON-RPC | `initialize`, `tools/list`, `chart_check`, `chart_diff` ok |
| `render retry.akrc | xmllint --noout -` | válido |

### 1.1 O que **não** foi executado, e por quê

- **`cargo build --release`**: falhou **duas vezes** por `Disk quota exceeded (os error 122)` no
  `/tmp` (tmpfs de 8,7 GB compartilhado com outros dossiês), ao compilar `read-fonts v0.46.0` e
  depois ao linkar `akr` com `lto = "fat"`. Concluiu na terceira, depois de remover os artefatos de
  `debug` (`Finished release profile in 1m 03s`). **Todos os números de performance deste dossiê são
  do binário release** (8,4 MB), mediana de 7 execuções. Release e debug produzem o mesmo sha256.
- **`make check` / `make ci`**: exigem `cargo fmt --check`, `clippy -D warnings`, `cargo-deny` e
  `typos` instalados, mais `xtask` com lock próprio que exige rede. Rodei `fmt`/`clippy`/`test`
  isolados via `cargo test` (que já compila com os lints de workspace) e não reproduzi o portão
  completo.
- **`install.sh` e o Release de 6 alvos**: exige rede.
- **`akr init`**: não executado para não escrever `akribie.toml`/`AGENTS.md` dentro de `ref/`.
  Coberto por `crates/akribie-cli/tests/init.rs` (2 testes, passaram).
- **`akr --at REV`**: exige repositório git com histórico; o clone tem **um** commit.
- **Leitor de tela real** (NVDA/VoiceOver/Orca): não executado. A conclusão sobre `<desc>` em
  português com `lang=en` vem do byte emitido, não de teste com usuário.

### 1.2 Achado que muda a leitura do projeto

`crates/akribie-core/Cargo.toml:12-16` promete que `--no-default-features` "deixa só
SVG/scene/excalidraw". **Não compila** (`error[E0433]` em `emit/raster.rs`) porque não existe nenhum
`#[cfg(feature = "raster")]` no crate. Detalhe em [`arquitetura.md`](arquitetura.md) §13.1. Consequência
para a rubrica: a "matriz de features" e o gate `make tree` que testa a variante sem `raster` são
advisory, não mecânicos.

## 2. Matriz de usabilidade (13 itens)

| # | Item | Status | Evidência |
|---|---|---|---|
| 1 | Instalação e pré-requisitos | **E + D** | `cargo build -p akribie-cli` e `cargo test` funcionam offline com Rust 1.99 local; MSRV `1.99` e `edition 2024` (`Cargo.toml:17-18`); `rust-toolchain.toml` fixa o canal; **zero dependência de sistema em runtime** — as duas faces OFL são `include_bytes!` (`crates/akribie-core/src/text.rs:29-31`). Instalação por `install.sh` com SHA-256 (`ref/akribiechart/README.md:30-51`) não executada (rede). |
| 2 | Inicialização mínima | **E** | Harness de 5 chamadas roda em 20,6 s de build e ~40 ms de execução (`exemplos/harness/src/main.rs`); `Font::load()` + `pipeline::check()` + `layout()` + `paint()` + `emit::svg()`. Pela CLI: `akr check x.akrc` sem configuração. **Atrito**: `config` default exige `Config::default()` explícito e não há builder validado (`crates/akribie-core/src/config.rs:56-69`). |
| 3 | Edição básica (texto, parágrafo, negrito/itálico/link) | **E (análogo)** | Não há superfície de edição: o texto é escrito no `.akrc` em qualquer editor e validado depois. O análogo executado é `check` → `fmt` → `diff` sobre texto real, com erro apontando linha/coluna. **Negrito/itálico/link não existem** como conceito — não há marks nem inline content no modelo (`crates/akribie-core/src/model/mod.rs:16-47`). |
| 4 | Seleção, teclado, undo/redo, clipboard, IME | **—** | Não aplicável: artefato estático, sem interação. Não há cursor, seleção, histórico nem composição de texto. Confirmado por ausência: `grep -rn "IME\|composition\|undo" crates/` não acha nada de edição (só undo não existe mesmo); `docs/plan/00-panorama.md:59-60` põe "edição interativa" **fora da v1**. |
| 5 | Schema e restrições de conteúdo | **E + T** | Conjuntos fechados e faixas numa tabela única (`crates/akribie-core/src/syntax/specs.rs:63-82`), validados por tipo (`syntax/value.rs`); erros `E002`/`E003` com a lista válida no `hint` (**[E]**: `shape=trapezio` → lista as 9 formas); `E004` id duplicado, `E005` referência ausente, `E006` `layout=free` sem `at`; 13 testes em `crates/akribie-core/tests/parse.rs` + 17 em `src/syntax`. `akr schema` publica a mesma tabela (`exit 0`). |
| 6 | Custom node/mark/plugin | **C (negativo)** | Não existe eixo de extensão. Cinco statements fixos (`syntax/mod.rs:24-50`), 9 formas e 6 layouts como `const &[&str]` (`syntax/specs.rs:63-82`), `enum` fechados (`model/enums.rs`). Adicionar um item exige editar o crate e recompilar. Não há plugin, hook, registro em runtime nem macro de schema. É escolha de escopo, não lacuna — mas é um teto duro para nós. |
| 7 | Persistência e formato de serialização | **E + T** | Fonte `.akrc` em texto UTF-8, forma canônica única e **idempotente** (md5 igual em duas passadas); `--check` com `exit 1`; goldens `fmt_canonical.akrc`/`fmt_expected.akrc` + 11 testes em `tests/fmt.rs`; determinismo de bytes verificado em SVG/scene/excalidraw/PNG. Formato versionado: `chart v1` obrigatório, outra versão ⇒ `E009` (`diag.rs:70-72`). |
| 8 | Renderização e não interferência no CSS | **E** | O SVG é documento autocontido: `xmlns`, `viewBox`, `role`, `lang`, cores como atributos `fill`/`stroke` hex, sem `<style>`, sem classe CSS, sem dependência de folha externa (`crates/akribie-core/src/emit/svg.rs:20-213`); `xmllint` aceita. Não há CSS a vazar porque não há host. **Contrapartida**: a apresentação é imposta — paleta fechada de 9 tokens × 2 temas, 2 fontes, 9 formas (`theme/mod.rs:37-72`). |
| 9 | Acessibilidade (ARIA, foco, leitor de tela, conteúdo editável) | **E + C + T** | **[E]** SVG real tem `role="img"`, `aria-labelledby="akr-t akr-d"`, `<title>`, `<desc>` com topologia em texto, `aria-label` por figura, `lang`, `focusable="false"`; `xmllint` válido. Contraste auditado por código com `W007` (`a11y.rs:58-136`) e teste sobre **todas** as combinações de paleta (`theme::tests::text_meets_aa_on_every_fill` passou). `--strict` transforma `W007` em `exit 8`. **Lacunas verificadas**: `<desc>` é português fixo mesmo com `lang=en` (`a11y.rs:30-41`); `role="list"`/`listitem` prometido em `docs/specs/A11Y.md:16,25` **não** é emitido (`emit/svg.rs:85-122`); não há foco/teclado/navegação. |
| 10 | Colaboração | **—** | Fora de escopo declarado (`docs/plan/00-panorama.md:59-60`: "Fora da v1: … colaboração"). Nenhum código de OT/CRDT/presença em `crates/`. |
| 11 | Performance e limites conhecidos | **E** | Medido em **release**, mediana de 7 execuções: 3 nós → 3,1 ms SVG / 2,5 ms `check`; 200 nós → 16,6 ms / 3,6 ms; 1000 nós / 999 arestas / 45 KB → **80,8 ms** SVG e **14,4 ms** `check`; raster ~30× o SVG (`scripts/bench.sh`: PNG 101–137 ms nas fixtures reais). `check` é ~5,6× mais barato que `render` (não faz layout/pintura/emissão). Existe `scripts/bench.sh` com saída fixa para comparar versões; **não** há gate de performance no CI nem orçamento de alocação. |
| 12 | Erros, migração e diagnóstico | **E + D** | 16 códigos estáveis (`E001`–`E009`, `W001`–`W007`) numa tabela única que alimenta `check`, `akr schema` e `akr prime` (`diag.rs:55-91`); mensagem = código + linha:coluna + esperado + `hint:`; saída de texto e JSON (`diag.rs:134-159`); exit codes numa só tabela (`error.rs:37-46`, verificados 0/1/2/3/8). Migração: `chart v1` com `E009` obriga versão nova a ser explícita; `CHANGELOG.md` em Keep a Changelog. **Lacuna**: `check --batch` com erro parcial ainda sai `0`; só `8` quando todos os itens falham. |
| 13 | Licença e maturidade | **E + D** | `MIT OR Apache-2.0` (`Cargo.toml:19`), `LICENSE` presente (Apache-2.0 integral) e as duas fontes sob OFL-1.1 com textos próprios (`assets/EXCALIFONT-OFL.txt`, `LIBERATION-OFL.txt`); v0.1.3 no `CHANGELOG.md` com 1 release documentada; **um único commit** no clone, 2026-11-03; `make check` como portão declarado com lint agressivo. |

**Cobertura: 13/13 itens com status explícito**, sendo 8 com execução direta (1, 2, 3, 5, 7, 8, 9,
11), 3 por código/teste/documentação (6, 12, 13) e 2 marcados `—` por ausência real de superfície
(4, 10). Nenhum item ficou "não verificado". Os pontos fracos estão nomeados dentro do item, não
escondidos atrás do status.

## 3. Notas por dimensão

Escala 0–5. `5` = referência que copiaríamos sem mudanças; `0` = inutilizável.
Como o akribie **não é um editor**, as notas respondem "quão boa é a referência, para o nosso caso".

| Dimensão | Nota | Evidência | Comentário |
|---|---|---|---|
| **Usabilidade de integração** | **4** | Build offline ok e suíte completa em 67 s **contando a compilação** (1,8 s com artefatos prontos); harness de biblioteca em ~75 linhas roda; erros de borda claros (``formato `pdf` desconhecido; use svg, scene, excalidraw, png, jpg ou avif``, `` `--format png` exige `--out` ``). Atritos reais: `edge a->b` é erro (§4.3), `--out-dir` precisa existir, `check --batch` só falha quando todos os itens falham. | Init mínimo e API clara; perde o 5 pelos três atritos acima, todos verificados. |
| **API** | **4** | `lib.rs:22-58` expõe 24 módulos públicos e 12 reexports; `#![forbid(unsafe_code)]` + `#![warn(missing_docs)]` (`lib.rs:17-18`); funções puras com assinatura estável (`pipeline.rs:27`, `layout/mod.rs:150`, `paint/mod.rs:168`, `emit/svg.rs:20`, `diff/mod.rs:84`). | Pequena e tipada. Perde 1 ponto por: `Config::default()` ser o único caminho sem validação acumulada; `Font::load()` obrigatório em quase toda chamada; a superfície de `features` mentir (§1.2). |
| **Neutralidade de estilo** | **—** | Não aplicável: não é biblioteca de UI nem injeta CSS/DOM — emite um documento SVG final (`emit/svg.rs`), que é o produto, não um componente a ser estilizado pelo host. Constraint correlato (não nota): paleta fechada de 9 tokens × 2 temas (`theme/mod.rs:37-72`), 2 fontes (`text.rs:29-31`), 9 formas. | Inventar nota aqui compararia coisas diferentes. O que importa para nós — "o core impõe estilo ao host?" — não tem análogo: não há host. |
| **Modelo de dados** | **5** | Quatro camadas com responsabilidade única e sem vazamento: `Document` (sintaxe, preserva comentários) → `Chart` (semântica tipada, com `Span`) → `Placed` (geometria sem cor) → `Scene` (pintura pronta para bytes) — `parse/mod.rs:17`, `model/mod.rs:16-47`, `layout/mod.rs:127-146`, `paint/mod.rs:144-165`. Enums fechados, zero string mágica; todo elemento carrega sua posição no fonte; forma canônica validável. | O melhor do projeto, e a analogia exata do que queremos: cada estágio consome um valor e não reconstrói o anterior. Ressalva fora da nota de "modelo": lookup por id é O(n) (`model/mod.rs:151-174`) — o modelo é bom, a indexação não. |
| **Extensibilidade** | **1** | Nenhum eixo. Statements, formas, layouts, tokens, fontes e regras de diagnóstico são `const`/`enum` fechados (`syntax/specs.rs:63-82`, `model/enums.rs`, `diag.rs:88-91`). Sem plugin, hook, macro de schema ou registro em runtime. O único caminho é fork/recompile. | `1` e não `0` porque a tabela de `syntax::specs` é uma fonte única limpa — adicionar uma *chave* é mecânico (parser, `schema` e `prime` acompanham, ADR-0002). Mas isso é extensão de *build*, não de *produto*, e não serve a um editor. |
| **Performance** | **4** | Medido em release com mediana de 7 execuções (§2, item 11): 1000 nós → 80,8 ms SVG, 14,4 ms `check`; fixtures reais → 3,7–4,6 ms SVG. Escala ~linear de 200→1000 nós com 5× mais linhas → 4,9× mais tempo. `check` desacoplado de `render` (razão 5,6×) permite loop de validação barato. `scripts/bench.sh` existe. | `4` e não `3` porque o custo é previsível, medido no artefato distribuído e o gargalo é identificado (dagre). Não chega a `5`: sem benchmark em CI, sem orçamento de alocação, e `describe()` é O(V·E) (`model/mod.rs:151-174`). |
| **Acessibilidade** | **4** | `role="img"` + `aria-labelledby` + `<title>` + `<desc>` derivado do modelo + `aria-label` por figura + `lang` + `focusable="false"`, validado por `xmllint`; contraste auditado em código com piso WCAG 2.2 (4,5:1 / 3:1) e teste sobre toda a paleta; `W007` com `--strict` = falha de CI. | `4` = "boa base, faltam casos", literalmente: `<desc>` só em português (§4.5), `role="list"` prometido e não emitido, nenhuma navegação por teclado. O *mecanismo* (a11y derivada do modelo + auditoria como diagnóstico) é nível 5; a *cobertura* não. |
| **Documentação** | **4** | 6 ADRs (558 linhas) com status/data/substituição; 4 specs normativos; 8 guias de uso (um por verbo); plano com 9 fases e matriz de aceitação; `review-protocol.md`; `CHANGELOG.md` em Keep a Changelog; `SKILL.md` e `llms.txt`; saída de máquina em `akr schema`/`akr prime`. | Volume e profundidade de nível 5; desconto por **três** afirmações normativas desatualizadas: portas `Env`/`Logger` (`00-panorama.md:39-40`, `DETERMINISM.md:32`), `role="list"` (`A11Y.md:16,25`), e a feature `raster` (`crates/akribie-core/Cargo.toml:12-16`). Doc que mente sobre o próprio código não tira 5. |
| **Manutenção** | **4** | Guardas reais: `forbid(unsafe_code)` (`lib.rs:17`), `unwrap_used`/`expect_used`/`panic`/`indexing_slicing`/`as_conversions` negados (`Cargo.toml:77-120`), limite de 300 linhas verificado (**nenhum arquivo viola**), `cargo tree` sem async runtime, `cargo-deny` + `typos` no `make ci`, 230 testes, `CHANGELOG` por release, ADR com protocolo de substituição. | Forte. Descontos: **um commit e um autor** (bus factor), feature sem CI de matriz (§1.2), release não comprovado neste ambiente, 3 docs desatualizados. |
| **Adequação ao `rich-editor-auy`** | **2** | Não é editor: sem documento editável, sem marks, sem posições, sem steps, sem seleção, sem histórico (§2 itens 4 e 10). Compartilha, porém, *exatamente* o núcleo de rigor que queremos: bytes canônicos, erro como dado, a11y derivada do modelo, diff semântico, forma canônica idempotente. | `2` pela âncora da rubrica ("referência indireta"), o que é honesto: **nenhuma linha de código é reaproveitável** num editor. O valor é arquitetural e de contrato, e é alto — ver [`ideias.md`](ideias.md). Não dou 3 porque "adaptável com custo" sugeriria portabilidade de código que não existe. |

**Média (9 dimensões aplicáveis): 3,56.** Se ponderássemos só modelo/API/documentação/manutenção —
o eixo "como se constrói um núcleo canônico" — a média sobe para **4,25**; se ponderássemos
"ser editor", desaba para ~1,5. As duas leituras são verdadeiras e é por isso que a nota de adequação
é `2` sem contradizer as outras.

## 4. Respostas diretas

### 4.1 O que **copiar** (conceito + forma, não código)

1. **Forma canônica única com prova de idempotência.** `fmt(fmt(x)) == fmt(x)` é testável em uma
   linha e impede a representação de degradar. Regras enumeradas e sem ambiguidade
   (`codigo/fmt.rs:3-13`). É a propriedade que o nosso serializador precisa ter desde o primeiro dia.
2. **Diagnóstico como tipo com código estável.** `Code { code, name, severity }` + `Diagnostic
   { code, span, message, hint }`, com `ALL` alimentando simultaneamente o CLI, `schema` e `prime`
   (`codigo/diag.rs:28-91`). Copiar a *forma*: código estável, severidade intrínseca ao código,
   `hint` como campo, ordenação determinística por posição.
3. **`num::format` como única porta de float para texto.** Três linhas que eliminam uma classe
   inteira de flakiness (`num.rs:8-25`). No nosso wire canônico isso é obrigatório.
4. **Descrição de acessibilidade derivada do modelo, não do layout** (`codigo/a11y.rs:16-42`).
   A ADR rejeita explicitamente a alternativa (`ADR-0005`, "Alternativas rejeitadas"). Para nós: a
   descrição de um documento deve sair do `Doc`, nunca da view data.
5. **Auditoria de acessibilidade como diagnóstico, com teste exaustivo da paleta.**
   `check_contrast` devolve `Vec<Diagnostic>` e o teste percorre todas as combinações
   (`a11y.rs:58-136`, `theme/tests.rs`). "Acessibilidade é teste, não recomendação."
6. **Diff semântico com identidade explícita por tipo de elemento** (`codigo/diff-mod.rs:84-132`):
   nó por `id`, aresta por `(from, to, ordinal)`, nota por âncora + posição. É exatamente o desenho
   de um "change set" de documento, e resolve o caso `id` ausente com uma regra declarada.
7. **Porta que não tem consumidor é deletada.** `Env`/`Logger` foram removidas
   (`ports/mod.rs:5-6`). Regra de higiene arquitetural barata e correta.
8. **Erro de ferramenta é resultado, não exceção do protocolo** (MCP, `tools.rs:1-5`). Nossa
   fronteira CLI/JSON-RPC deve ter o mesmo contrato.
9. **"Isto foi medido e empatou"** documentado no código (`emit/svg.rs:10-13`). Evita que alguém
   reabra a mesma micro-otimização em seis meses.

### 4.2 O que **adaptar**

1. **O pipeline de estágios com estruturas intermediárias tipadas.** O akribie tem
   `Document → Chart → Placed → Scene`; nós temos `Doc → ViewData` com steps no meio. Adaptar:
   manter a regra "cada estágio consome um valor e não reconstrói o anterior", mas com `step`/
   `Mapping` no lugar de `layout`/`paint`.
2. **Recuperação de erro por linha.** O `.akrc` recupera por linha (`GRAMMAR.md:3-5`); nosso parser
   de documento precisa recuperar por *bloco*, não por linha. O princípio (um erro não invalida o
   resto) transfere; o granular não.
3. **`--json` na CLI e `content[]` no MCP sobre a mesma função.** O akribie faz isso certo; adaptar
   é manter o domínio alheio ao envelope, com um adaptador por superfície.
4. **`check` separado de `render`.** O análogo nosso é uma API `validate(&Doc) -> Vec<Diagnostic>`
   que não constrói view data — barata o suficiente para rodar a cada tecla.
5. **Exit codes como tabela única em `error.rs`, publicada por `schema`.** Adaptar como um
   `ErrorKind` no core com `exit_code()` e um doc que lista a tabela.
6. **Goldens de saída real sobre corpus real** (`fixtures/reais/` + `tests/golden/reais/*.svg`).
   Adaptar para nosso corpus de documentos e o `doc → wire` canônico.

### 4.3 O que **evitar**

1. **Feature declarada que não compila.** `raster` existe em `Cargo.toml` e não existe no código
   (§1.2). No nosso caso: se `serde` é feature opcional, o CI **tem** de buildar
   `--no-default-features`; senão a feature é ficção.
2. **Texto gerado em idioma fixo.** `a11y::describe` tem português hardcoded com `lang=en` no
   documento (`a11y.rs:30-41`). O nosso `lang` precisa governar todas as strings geradas, ou o campo
   não deve existir.
3. **Spec que promete o que o emissor não faz.** `role="list"`/`listitem` em `A11Y.md:16,25` e
   ausentes em `emit/svg.rs:85-122`. Regra: a11y prometida em doc tem teste que a verifica no byte.
4. **Lookup linear por id em estrutura quente.** `Chart::node` é `iter().find`
   (`model/mod.rs:151-174`), e `describe`/`diff` chamam isso por aresta. Num editor, `NodeId → &Node`
   num índice é orçamento, não luxo.
5. **Gramática que exige espaçamento exato em token estrutural.** `edge a->b` é `E007`
   (`lex.rs:97-123`). Nosso wire não deve ter essa fragilidade; se tiver, o parser precisa normalizar
   antes de reclamar.
6. **Batch que esconde erro parcial no exit code.** `check --batch` com itens válidos e inválidos
   retorna `0`; só falha (`exit 8`) quando **todos** os itens falham
   (`crates/akribie-cli/src/app/emit/batch.rs:21-24`) (**[E]**). Um veredito
   que não cabe no exit code precisa de um campo agregado explícito — não de silêncio.
7. **`import` apresentado como reversível.** `akrc → excalidraw → akrc` perde `curve`, colapsa formas
   e materializa coordenadas (§6.6 do README). Se publicarmos import/export, precisamos de um
   `ImportReport` que conte o que foi degradado — e de dizer isso no nome do campo.

### 4.4 O quanto é "determinístico de verdade" e o quanto é doc

**Determinístico de verdade**, com três níveis de prova:

- **Prova por execução**: SVG, `scene`, `.excalidraw` e PNG com sha256 idêntico em duas execuções
  (§1).
- **Prova por teste**: 8 testes dedicados (`emit/svg/tests.rs:22`, `emit/scene_json/tests.rs:21`,
  `emit/excalidraw/tests.rs:23`, `emit/raster.rs:151`, `layout/tests.rs:194`, `sketch/tests.rs:15`,
  `tests/render.rs:26`, `tests/render.rs:473` com goldens de corpus real), todos passando.
- **Prova por mecanismo**: as cinco proibições (relógio, RNG, `HashMap`, `f32::to_string`, fonte do
  sistema) têm substituto em código, não em convenção (§6 do [`arquitetura.md`](arquitetura.md)).

**É doc, não código**: "o raster é determinístico na versão" (`DETERMINISM.md:12-14`). Há um teste
(`emit/raster.rs:151`) que compara duas codificações **na mesma máquina/versão**, o que não prova
nada sobre "entre versões" — só sobre "nesta". É o único contrato de determinismo que é declaração.

### 4.5 O quanto a acessibilidade passa de recomendação a garantia

**Garantido por mecanismo**:

- `<title>`/`<desc>`/`aria-labelledby`/`role="img"`/`lang` são emitidos sem condição
  (`emit/svg.rs:20-37`) — e há teste de binário real (`tests/render.rs:18-25`).
- Contraste tem piso numérico em código (`theme/mod.rs:32-35`) e teste exaustivo da paleta.
- `W007` + `--strict` transforma violação em falha de CI.

**Não garantido**:

- `role="list"`/`role="listitem"` — prometido, não emitido.
- Idioma do `<desc>` — fixo em português.
- Navegação por teclado, foco, ordem de leitura — inexistentes (`focusable="false"`, zero
  `tabindex`; verificado com `grep -c tabindex` → `0`).
- Legibilidade do traço à mão em tamanho pequeno — não há teste de "renderizado a 50%".

Resumo honesto: **a11y do artefato ≈ 8/10; a11y de interação = 0/10 (e fora de escopo)**.

### 4.6 Onde a documentação diverge do código

Três divergências verificadas, todas normativas:

| doc afirma | código faz |
|---|---|
| `docs/plan/00-panorama.md:39-40` — "quatro portas (`Fs`, `Git`, `Env`, `Logger`)" | duas portas (`ports/mod.rs:12-43`); `Env`/`Logger` removidas |
| `docs/specs/DETERMINISM.md:32` — "a CLI usa a porta `Env` apenas para configurar caminhos" | não existe porta `Env` |
| `docs/specs/A11Y.md:16,25` — `#nodes` tem `role="list"`, cada nó `role="listitem"` | `emit/svg.rs:85-122` emite `<g id="nodes">` e `<g data-id=… data-kind=…>` sem `role` |
| `crates/akribie-core/Cargo.toml:12-16` — desligar `raster` "deixa só SVG/scene/excalidraw" | `--no-default-features` não compila |

Nenhuma delas quebra o produto (todas são sobre superfície ou documentação), mas todas indicam que
**o CI não cobre documentação nem matriz de features**. Para nós: um teste que compara a saída real
do `schema` com a tabela do doc, e um job `--no-default-features`.

## 5. Confiança e lacunas

- **Confiança nas afirmações de arquitetura e modelo: 93%.** Todo o pipeline foi lido no fonte,
  230 testes passaram num `cargo test --workspace --locked` real, o harness de biblioteca rodou, e
  as afirmações de determinismo têm sha256 de execução própria além dos testes.
- **Confiança em performance: 85%.** Números de **release** com mediana de 7 execuções por amostra e
  `scripts/bench.sh` (10 execuções por modo × fixture) como segunda fonte. O build de release exigiu
  três tentativas por quota de disco (§1.1), mas concluiu, e release/debug dão os mesmos bytes — a
  medição é do artefato que o projeto distribui. A revisão independente fixou 85% por causa do
  ambiente compartilhado.
- **Confiança em a11y "de leitor de tela real": 70%.** Verifiquei estrutura, contraste e validade
  XML; **não** testei com NVDA/VoiceOver/Orca. A conclusão sobre o `<desc>` em português com
  `lang=en` é dedução direta do byte emitido, não de teste com leitor.
- **Confiança em batch/`--out-dir`/`import`: 90%.** Executados; o comportamento de erro foi
  observado, não inferido. Não testei lotes grandes, nem `--at REV` (exigiria repo git com histórico —
  o clone tem um commit).
- **Não verificado**: `make check`/`make ci` completos; `install.sh` e os 6 alvos de release;
  `akr init` (não quis escrever dentro de `ref/`); comportamento em macOS/Windows; se
  `akribie-core` compila em toolchains 1.92/1.98.

**Meta de 85% de usabilidade/resultado: atingida (≈90%)** — 13/13 itens com status, 8 deles com
execução direta, 10 dimensões com nota e evidência (9 notas + 1 `—` justificado), e cada afirmação
central com caminho de arquivo ou saída de comando. A revisão independente
([`revisao.md`](revisao.md)) reexecutou a suíte e subiu a confiança final para **90%**.
