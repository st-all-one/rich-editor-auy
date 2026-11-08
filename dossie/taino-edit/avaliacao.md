# taino-edit — avaliação

> Rubrica com nota 0–5 e evidência por dimensão, cobertura da matriz de
> usabilidade do `_GUIA.md` e respostas diretas às perguntas do dossiê.
> Notas sem `caminho:linha` ou execução registrada são inválidas; todas as
> abaixo têm.

## 1. Método e o que foi executado

Comandos rodados no checkout real (nada em `ref/` foi alterado; artefatos em
`/tmp/taino-target`):

```sh
cargo metadata --offline --no-deps                 # ok
cargo test -p taino-edit-core -p taino-edit-extensions   # 212 passaram
cargo test -p taino-edit-dom --lib                       #   6 passaram
cargo run -p headless-core -q                            # saída colada no README
cd dossie/taino-edit/exemplos && cargo run -q             # saída colada em exemplos/README.md
```

Os "212" são **208 `#[test]` + 4 doc-tests**; `--all-features` também dá 212.
O README do projeto anuncia 214 (`README.md:39`), número que não reproduzi.
Reexecutei tudo na revisão (mesmo resultado) e recontei a soma por
`test result: ok. N passed`.

Benchmark próprio em `/tmp/taino-bench` (release, melhor de 3, histórico 100):
27–39 µs/tecla com 100 blocos → **565–862 µs** com 20 000 (a reexecução da
revisão está na tabela de `README.md` §5; os extremos variam entre execuções).

**Não executado** e por quê:

- testes de browser (`wasm_bindgen_test`): exigem Chrome-for-Testing 148 +
  `chromedriver` + runner `wasm-bindgen-cli` patchado
  (`ref/taino-edit/scripts/wasm-test.sh:1-25`); o runner disponível é 0.2.128 e
  o `Cargo.lock` fixa 0.2.121. São 78 + 10 + 4 + 4 = 96 casos declarados,
  contados por `grep -c '#\[wasm_bindgen_test\]'`.
- exemplos Leptos/Dioxus: exigem `trunk`/`dx` e toolchain wasm com as versões
  exatas de `leptos 0.8.3`/`dioxus 0.7`.
- `cargo clippy`/`cargo doc` do workspace inteiro: proibido pelo contrato do
  dossiê (monorepo inteiro); CI do projeto roda isso
  (`ref/taino-edit/.github/workflows/ci.yml:24-45`).

## 2. Matriz de usabilidade (13 itens)

Legenda: **E** = verificado por execução; **C** = verificado no código;
**T** = verificado em teste (lido, não executado quando foi browser); **D** =
verificado em documentação oficial do repo; **X** = não verificado.

| # | Item | Status | Evidência |
| --- | --- | --- | --- |
| 1 | Instalação e pré-requisitos | **E + D** | `cargo test`/`cargo run` funcionaram com Rust 1.99 local; MSRV 1.80 declarado (`Cargo.toml:16`); features em `README.md:163-176`; `rust-toolchain.toml` fixa stable + wasm32 |
| 2 | Inicialização mínima | **E (headless) + D (Leptos)** | `exemplos/src/main.rs` roda; snippet Leptos em `README.md:178-206`; **atenção**: mínima de fato exige `keymap` |
| 3 | Edição básica (texto, parágrafo, negrito/itálico/link) | **E + T** | exemplo insere texto e aplica `Mod-b`; `crates/taino-edit-extensions/tests/marks.rs` e `.../tests/blocks.rs` (não há esses arquivos em `core/tests/`); link exige prompt do host (`extensions/src/link.rs:1-12`) |
| 4 | Seleção, teclado, undo/redo, clipboard, IME | **C + T (browser não executado)** | `codigo/selection.rs`; `keymap.rs:218-242` (8 bindings base); `codigo/state.rs:219-258` (undo/redo); `codigo/view.rs:546-610` (paste); `tests/composition.rs:73-113`, `tests/clipboard.rs` (6 casos) |
| 5 | Schema e restrições de conteúdo | **E + T** | exemplo imprime `true` para parágrafo contendo bloco; `content.rs:1-110`; `tests/model.rs`, `tests/schema_macro.rs`. **Lacuna**: expressão de marks por nó não é aplicada (`schema.rs:29-40`) |
| 6 | Custom node/mark/plugin | **C + E** | `Extension`/`Plugin`/`ViewPlugin` (`extensions/src/lib.rs:71-86`, `plugin.rs:71-130`, `view.rs:20-60`); exemplo compõe 5 extensões; `tests/plugin.rs` (7 casos) |
| 7 | Persistência e formato | **E** | exemplo imprime JSON, HTML e Markdown; `json.rs:96-128` valida contra schema; `html.rs:606-615`; `markdown.rs:1-20` |
| 8 | Renderização e não interferência no CSS | **C** | nenhum `.css` no repo (`find`); `to_dom`/`DomSpec` em `schema.rs:29-45`; classes fixas e `style` inline (`extensions/src/table.rs:87`, `align.rs:50-60`) |
| 9 | Acessibilidade | **C** | só `contenteditable` + `tabindex` (`view.rs:103-106`); grep não acha `aria-*`/`role` nos crates; ROADMAP declara WCAG fora de escopo (`ROADMAP.md:305-315`) |
| 10 | Colaboração | **D** | `COLLAB_DESIGN.md` integral; nenhum código; feature `collab` vazia (`core/Cargo.toml:22-24`) |
| 11 | Performance e limites | **E** | benchmark próprio (tabela em `README.md` §5); risco documentado em `ROADMAP.md:323` |
| 12 | Erros, migração e diagnóstico | **C + D** | `SchemaError`/`DocError` com variantes específicas (`error.rs:1-100`); `StepError`/`ReplaceError`; CHANGELOG com breaking changes por versão (`CHANGELOG.md:9-40`); `keymap.rs` faz `panic!` em modificador desconhecido (`keymap.rs:124`) |
| 13 | Licença e maturidade | **E + D** | `MIT OR Apache-2.0` (`Cargo.toml:17`), arquivos de licença presentes; v0.7.0 com **11 releases** no CHANGELOG (0.1.0→0.7.0); git shallow de **1 commit**, último 2026-09-05 |

**Cobertura: 13/13 itens com evidência; confiança realista ~85%**, sendo 8 com execução direta
(itens 1, 2, 3, 5, 6, 7, 11 e 13) e 5 por código/teste lido/documentação.
Nenhum item ficou como "não verificado"; o que é fraco está marcado como lacuna
dentro do item, não escondido atrás do status.

## 3. Notas por dimensão

Escala 0–5. `5` = referência que copiaríamos sem mudanças; `0` = inutilizável.

| Dimensão | Nota | Evidência | Comentário |
| --- | --- | --- | --- |
| **Usabilidade** (para quem integra) | **3** | 212+6 testes passam; exemplo headless em ~120 linhas funciona (`exemplos/src/main.rs`, 129; o do repo tem 121); porém `<TainoEditor state=… />` sem `keymap` tem Enter/Backspace/Delete mortos (`crates/taino-edit-leptos/src/lib.rs:336-339`); sem stored marks (`commands.rs:99-101`) | O núcleo é fácil; o "hello world" do README é uma armadilha. Com keymap, a experiência é decente. |
| **API** | **4** | `Command` probe/dispatch (`commands.rs:24-41`); `Transaction` com `no_history`/`join_history`/`set_selection` (`state.rs:272-320`); schema builder + `schema!` macro (`schema_macro.rs:1-60`); `Transform` exige `&Schema` em cada chamada (`transform.rs:63-100`) | Coerente e idiomática. Perde um ponto por passar `&schema` a cada método e por `EditorState::apply` engolir erros de inversão (`state.rs:192-198`). |
| **Neutralidade de estilo** | **3** | zero CSS no repo; mas `class="taino-editor"` fixo (`leptos/src/lib.rs:189`, `dioxus/src/lib.rs:228`), `taino-cell-selected` (`table-view/src/lib.rs:33`), e `style` inline no **documento** (`table.rs:87`, `align.rs:50-60`) | Não força folha de estilo — ótimo. Mas emite estilo inline no conteúdo serializado, o que suja o dado e impede tema por CSS. |
| **Modelo de dados** | **5** | `Arc<NodeInner>` imutável (`codigo/node.rs:102`); `Fragment` com `size` cacheado (`fragment.rs:16-45`); posições ProseMirror (`pos.rs`); schema com DFA (`content.rs`); JSON canônico com attrs ordenados (`attrs.rs:1-16`) | O melhor do projeto. É essencialmente o modelo que queremos, com uma ressalva (contagem de chars O(n)). |
| **Extensibilidade** | **4** | três eixos explícitos (`Extension`/`Plugin`/`ViewPlugin`); `add_chained` para colisão de teclas (`keymap.rs:179-215`); 14 extensões embutidas, 13 ligadas no demo (`examples/basic-leptos/src/main.rs:98-112`); `TableView` prova o eixo DOM (`table-view/src/lib.rs`) | Arquitetura de extensão muito boa. Falta registro dinâmico de plugins no adapter e restrição de marks. |
| **Performance** | **2** | 27–39 µs/tecla com 100 blocos → 565–862 µs com 20 000 (medido e reexecutado); histórico clonado por transação (`state.rs:190`); `node_size` O(n) (`node.rs:191`) | Aceitável para documentos típicos, insuficiente para >10k blocos. Nota 2 porque o custo é estrutural, não um detalhe. |
| **Acessibilidade** | **1** | `tabindex="0"` e nada mais (`view.rs:102-106`); nenhum `aria-*`; toolbar sem `aria-pressed` (`examples/basic-leptos/src/main.rs:300-340`); WCAG fora de escopo (`ROADMAP.md:310`) | Baseline de foco. Um leitor de tela não tem como saber que é um editor multilinha. |
| **Documentação** | **5** | `DESIGN_NOTES.md` (decisões datadas), `ROADMAP.md` com fases e riscos, `COLLAB_DESIGN.md` com 5 questões abertas (Q1–Q5), rustdoc com exemplos que compilam (4 doc-tests passaram), `CHANGELOG` completo | Excepcional para um projeto solo. O README é o elo fraco (números desatualizados). |
| **Manutenção** | **4** | CI fmt/clippy/test/doc + job SSR (`ci.yml:1-80`); `#![deny(unsafe_code)]` em 5 dos 7 crates — core, extensions, dioxus, table-view e umbrella; `dom` é a fronteira FFI documentada e `leptos` não declara o lint (não usa `unsafe`) (`DESIGN_NOTES.md:127`); `cargo-deny` (`deny.toml`); 96 testes de browser; porém projeto solo e wiring duplicado Leptos/Dioxus (496 vs 571 linhas) | Sinais de disciplina fortes. Risco de bus factor. |
| **Adequação ao nosso caso** | **3** | core sem DOM roda em qualquer lugar (`examples/headless-core`); modelo + steps + schema são exatamente o que precisamos; mas renderer é só `contenteditable`, a11y é fraca e há `style` inline | Reaproveitamento **conceitual** altíssimo; reaproveitamento **de código** limitado pelo renderer e pelo estilo embutido. |

Média simples: **3,4**. Se ponderarmos modelo/API/extensibilidade (o que mais
importa para nós) e ignorarmos a11y, sobe para ~4.

## 4. Respostas diretas

### 4.1 O que podemos reaproveitar conceitualmente

1. **Modelo tipado `Arc<Node>` + `Fragment` com size cacheado + `ResolvedPos`.**
   É o mesmo desenho do ProseMirror com ownership idiomático em Rust
   (`codigo/node.rs`, `pos.rs`). Copiar a *forma*, não o código, e corrigir o
   `node_size` O(n) cacheando o comprimento do texto.
2. **Schema declarativo + autômato de conteúdo compilado (NFA→DFA).**
   `content.rs:1-110` mostra que dá para ter validação real sem dependência e
   com erro de schema em tempo de build do schema, não em runtime de edição.
3. **Steps invertíveis + `Mapping` com mirror/recover.** A separação
   `apply/invert/map/to_json` (`codigo/step.rs:35-55`) é o contrato certo para
   undo/redo e para uma futura camada de colaboração.
4. **Comando como função `probe`/`dispatch` + `chain`.** Permite toolbar e keymap
   usarem a mesma superfície e compor fallbacks sem meio-aplicar
   (`commands.rs:24-41`).
5. **Keymap com `add_chained` em vez de sobrescrever.** Resolve de forma elegante
   a colisão `Tab` (tabela vs. lista) sem um sistema de prioridades
   (`keymap.rs:179-215`).
6. **Contrato "o serializador SSR emite exatamente o markup que o renderer
   monta", fixado por teste.** (`view_html.rs:1-45`.) Excelente ideia para
   renderização no servidor sem flash/divergência.
7. **Três eixos de extensão** (schema+keymap / estado observador / interação
   DOM). Ver `arquitetura.md` §6.
8. **Overlay para decorations inline**, em vez de dividir nós de texto
   (`decoration.rs:1-20`). Isso preserva a leitura do DOM — vale para busca,
   comentários e cursores remotos.
9. **`<br data-taino-trailing-break>` em bloco vazio** para o caret entrar
   (`view.rs:685-724`).
10. **IME como "suspender leitura, commitar no fim"**, com flag consultada na
    leitura (`view.rs:465-467`; `tests/composition.rs`).

### 4.2 O que NÃO devemos copiar

1. **Enter/Backspace/Delete mortos sem keymap.** Não sequestrar tecla
   estrutural sem ter um comando aplicável; deixar o browser agir e reconciliar
   é melhor do que um editor que não quebra linha
   (`leptos/src/lib.rs:336-339`).
2. **Leitura do DOM baseada em um único text run divergente.** Não escala para
   estrutura, IME exótico, autocorrect, ditado (`view.rs:455,464-519`).
3. **Clonar o histórico inteiro e inverter todos os steps a cada transação**
   (`state.rs:190-198`). Preferir estruturas persistentes/`Arc`-compartilhadas e
   inversão preguiçosa.
4. **`style` inline no documento** (`text-align`, `width`). Isso acopla dado a
   apresentação e impede tema por CSS; usar atributos semânticos + classes.
5. **Sem stored marks** (`commands.rs:99-101`). Hoje `Ctrl+B` sem seleção não faz
   nada; é um buraco de UX que usuários de TipTap esperam que exista.
6. **Wiring de eventos duplicado** entre Leptos e Dioxus (496 e 571 linhas quase
   idênticas). Extrair uma camada `web-sys` comum.
7. **Classe de container fixa e sem prop** (`class="taino-editor"`).
8. **Marks declaradas por nó e não aplicadas** (`schema.rs:29-40`) — não criar
   configuração que mente.
9. **`panic!` em modificador de keymap desconhecido** (`keymap.rs:124`):
   erro de configuração deve ser `Result`, não crash no editor do usuário.

### 4.3 O quanto é realmente headless e zero CSS

**Headless: sim no modelo, não no renderer.**

- `taino-edit-core` tem 4 dependências e nenhuma é web (`core/Cargo.toml:14-18`);
  `examples/headless-core` edita, aplica mark, desfaz e serializa sem DOM — eu
  rodei. Os 212 testes de core/extensões rodam em host.
- Mas **não há abstração de renderer**: para mostrar algo na tela é obrigatório
  `taino-edit-dom` + `web-sys` + `contenteditable` (`dom/Cargo.toml:14-30`).
  Um renderer nativo (`blitz`/`freya`) é explicitamente futurologia
  (`DESIGN_NOTES.md:47-50`; `ROADMAP.md:305-315`). "Headless" aqui significa
  *core sem DOM*, não *editor renderizável em qualquer backend*.

**Zero CSS: parcial.**

- Não existe nenhum `.css`/`.scss` no repositório (verificado com `find`), e o
  exemplo define todo o visual em `<style>` próprio
  (`examples/basic-leptos/index.html:7-55`). Nesse sentido, sim: nada de estilo
  imposto por folha.
- Porém o editor **impõe semântica de estilo pelo DOM**: `class="taino-editor"`
  fixa no container, `taino-cell-selected` nas células, e — pior —
  `style="text-align: …"` e `style="width: …px"` **dentro do documento**
  serializado (`extensions/src/align.rs:50-60`, `extensions/src/table.rs:87`).
  O estilo viaja com o conteúdo, então "zero CSS" vale para o *sheet*, não para
  o *dado*.
- Também não há como trocar a classe do container: o adapter não expõe prop para
  isso (`leptos/src/lib.rs:189`).

Resumo honesto: **8/10 headless** (modelo sim, renderer não) e **7/10 zero CSS**
(nenhuma folha, mas estilo inline e classes fixas).

## 5. Confiança e lacunas

- **Confiança nas afirmações de core**: alta (92%). Foram executadas (212 testes,
  exemplo próprio) e o código está copiado em `codigo/`. A revisão independente
  reexecutou os testes (212 + 6, também com `--all-features`), o
  `headless-core`, o exemplo do dossiê e o benchmark, e reconfirmou por `diff -q`
  que as seis cópias são idênticas às origens.
- **Confiança na experiência de browser**: média (78%). Toda a análise de IME,
  seleção, clipboard e patch vem de código + testes lidos, **não executados**
  (sem chromedriver/Chrome-for-Testing no ambiente). Se alguma conclusão mudar,
  é provavelmente aí.
- **Não verificado**: comportamento real em Safari/Firefox; fluidez do IME CJK;
  colagem de HTML real da web; arrasto nativo; se `input` dispara em `cut` no
  browser (não há handler de `cut`, então recorte depende disso).
- **Não verificado**: números de serialização (5 000 parágrafos → 844 KB de JSON
  em ~11 ms) — medidos uma vez, não reexecutados na revisão. Os benchmarks de
  tecla foram reexecutados, com variação registrada em `README.md` §5.
- **Números de host corrigidos na revisão**: LOC por crate (o dossiê
  subestimava core/extensions e superestimava dom), releases do CHANGELOG
  (11, não 7), commits do clone (1, não 4), questões abertas de colaboração
  (5, não 8), contagem de extensões (14 `impl`, 13 no demo).
