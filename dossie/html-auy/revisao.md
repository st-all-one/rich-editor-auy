# Revisão — `html-auy`

Revisão independente da rodada extra. Fonte: `ref/html-auy` (commit `c70213e`, 2026-11-07) e o
dossiê em `dossie/html-auy/`. Tudo abaixo foi **observado** nesta revisão, salvo o que está marcado
como não executado.

Nota de posse: esta revisão foi feita depois de o explorador original ser encerrado. Os arquivos
foram relidos no estado atual (mtimes de 22:26–22:31) antes das correções.

## 1. Veredito

O dossiê está correto na tese central: **`html-auy` não é editor**. A ausência foi buscada, não
inferida: 0 ocorrências de `contenteditable`, `struct Selection`, `fn undo`, `fn redo`, `collab`,
`struct Editor` e `dirty` em `crates/`; `lib.rs` não tem módulo de documento/edição. O valor real é
`Markup` opaco + escape por tipo + vocabulário em tabela + validação em camadas com dono declarado,
e é isso que o dossiê entrega. A distinção "editor 0 / renderer 4–5" está mantida e agora
consistente entre as colunas.

Confiança final desta revisão: **88%**. O que impede 90%+ é o mesmo que o dossiê já declara:
nenhuma medição de performance, MSRV não testada, `audit` não exercitado sobre uma página real,
fuzz/Miri/clippy não rodados.

## 2. Reexecuções (todas baratas e feitas)

| Comando | Resultado observado |
|---|---|
| `cargo run --offline` em `exemplos/` | compila e roda em **2,09 s**; saída **byte-idêntica** à registrada, incluindo `577 bytes` de determinismo e `class="card flat"` ordenado |
| `cargo check --offline` em `exemplos/falha-compilacao/` | **6 erros** exatamente como transcritos + **2 warnings** (`unused import: Text`, `unreachable statement`); ~5,0 s a frio, 0,06 s a quente |
| `cargo test -p html-auy --offline` | **24 alvos**; 186 testes + 3 doctests = **189 ok**, 0 failed |
| `cargo test -p html-auy --features audit --offline` | **207 ok** (+18) |
| `python3 tools/validate_spec.py` | 0 erros, 0 avisos |
| `python3 tools/gen_spec.py --check` | 2 tabelas em dia |
| `python3 tools/check_links.py` | 396 links, 0 quebrados |
| `python3 -m unittest discover -s tools/tests` | 18 testes OK |

Os diretórios `target/` gerados por essas execuções (nos dois exemplos e em `ref/html-auy`) foram
removidos ao final; `ref/html-auy` voltou ao estado original (`git status` limpo) e o dossiê não
contém artefato de build.

## 3. Verificações de números e de integridade

| Afirmação do dossiê | Verificação |
|---|---|
| 111 elementos | `grep -c '^\[element\.'` = **111** |
| 71 invariantes | 59 `block` + 4 `type` + 8 `output` = **71** (e as três listas de `rules.rs` somam 59: 20 + 15 + 24) |
| 13 regras de `audit` | `RULES: [&str; 13]` em `audit.rs:21-35` |
| `only = "shell"` 9 / `"component"` 1 | confirmado (`spec/elements.toml:719` é `form`) |
| 76 contratos `trybuild` | 76 `.rs` + 76 `.stderr` em `tests/ui/` |
| ~20,6 k linhas Rust; 6.382 geradas | `20646`; `3191 × 2 = 6382` |
| Cópias de `codigo/` | `diff -q` idêntico para `escape.rs`, `markup.rs`, `writer.rs`, `parser.rs`, `codegen.rs`; `validate_excerpt.rs` = linhas 1–200 + 430–560 + 2 linhas de marcação |
| MSRV/edição/resolver | `Cargo.toml:21` (1.92), `:20` (2024), `:15` (resolver 3) |
| Licença | MIT OR Apache-2.0 em `Cargo.toml:22` e nos três crates |

## 4. Achados e correções feitas

### 4.1 Erros de conteúdo (corrigidos)

1. **Invariantes de saída erradas** (`arquitetura.md` §9): a lista dos 8 itens `output` incluía
   `main_unique`, que é escopo **`type`**, e omitia `no_content_derived_identifiers`, que é
   `output`. Corrigido; acrescentada a nota de que `main_unique` é garantido pelo construtor.
   Evidência: `spec/elements.toml` (`main_unique = "type"`, `no_content_derived_identifiers =
   "output"`) e `docs/04-validacao.md:64-90`.
2. **Citação de linhas de elementos** (`avaliacao.md` §2): os 9 elementos citados apontavam para 7
   linhas, e `p`/`a` não estavam entre elas. Corrigido para
   `331(p),382(strong),388(em),534(a),492(ul),499(ol),432(blockquote),406(pre),400(code)`.
3. **Caminho do crate nos exemplos**: `../../../../ref/...` (4 níveis) não resolve a partir de
   `exemplos/`; o `Cargo.toml` usa 3. Corrigido no texto.
4. **Contagem de testes**: "25 binários; 189 testes + 3 doctests" somava 192. O correto é 24 alvos,
   186 testes + 3 doctests = 189 (com `audit`, 207). Corrigido em `exemplos/README.md`, `README.md`
   e `avaliacao.md`.
5. **Tempo do exemplo que falha**: o valor registrado (1,87 s) não se reproduziu; reobservado
   ~5,0 s a frio e 0,06 s a quente. Atualizado.
6. **"Zero dependência normal"** (`README.md` §6): o crate tem uma dependência normal — o
   `html-auy-macros` do próprio workspace. Corrigido para "zero normal **externa**", com
   `html5ever`/`markup5ever_rcdom` só sob `audit` (`crates/html-auy/Cargo.toml:23-32`).
7. **Ponteiros de linha deslocados** (verificados no fonte e corrigidos):
   - `is_forbidden`: `:549` → `:547`;
   - `check_forbidden_descendants`: `:343-370` → `:337-369`;
   - `audit` de `style=` : `audit.rs:482-486` → `:478-485` (em `arquitetura.md` e `avaliacao.md`);
   - `pub mod spec`: `lib.rs:63` → `:59`;
   - `resolver`: `Cargo.toml:16` → `:15`;
   - dependência opcional: era citada como `lib.rs:41-42` (só o gate do módulo); agora aponta para
     `crates/html-auy/Cargo.toml:23-32` + `lib.rs:44-45`;
   - auto-fechamento SVG (`close_open_self`): `codegen.rs:163-171` → `:149-153` (o trecho citado era
     o fechamento de elemento comum);
   - síntese de atributos: `codegen.rs:301-329`/`:315-329` → `:276-289` (SVG) e `:296-312` (rel);
   - dimensões de imagem: `value/image.rs:52-58` → `:45-79` (cobria só `WidthPx`; `HeightPx::new`
     está em `:70-72`).

### 4.2 Colunas editor × renderer

- `Adequação ao nosso caso` tinha **1 na coluna "como editor"** enquanto todas as outras linhas da
  mesma coluna eram 0 e o próprio texto dizia "como editor: nada". Corrigido para **0**, com nota de
  que é 0 por definição (não há superfície de edição), não por demérito. A coluna renderer segue
  4/5 na mesma linha.
- A nota "como editor 0 é veredito, não penalidade" permanece e está coerente com o guia.
- A matriz de usabilidade agora diz **12 de 13 com evidência direta + 1 parcial (performance)**, em
  vez de "13 de 13", que contradizia a própria linha de performance ("NV em medição"). 12/13 ainda
  supera a meta de 85%.

### 4.3 Deriva no upstream (registrada, não corrigida — não é nossa)

- `spec/elements.toml:33-40` ainda documenta os escopos como `block`/`page`, mas o código usa
  `block`/`type`/`output` (`table::Scope`) e `docs/04` já usa os nomes novos. O dossiê citava esse
  trecho como se ele descrevesse os três escopos; agora cita `docs/04:64-90` + `table.rs:13-22` e
  anota a deriva.
- `docs/09` descreve classes "em ordem estável (`Class` ordena por `Token` derivado)", enquanto a
  implementação ordena por `&'static str` (`value/class.rs`). Não afeta o dossiê (que cita
  `sort_unstable`/`dedup` no código), mas vale saber que a doc do projeto é mais frouxa que o código.

### 4.4 O que o explorador não viu (e ficou registrado)

- O `audit` tem **13** regras, mas a spec declara **8** invariantes de escopo `output`: 5 regras do
  `audit` (`aria_refs_resolve`, `no_content_derived_identifiers` já é spec, `no_inline_script`,
  `no_style_attribute`, `no_event_handler`) existem sem invariante `output` correspondente. Ou seja,
  "13 regras" e "8 invariantes de saída" não são o mesmo conjunto — o dossiê já não os iguala, mas a
  relação merecia uma linha.
- `Markup::from_render` é um produtor público adicional (além da macro), que aceita qualquer
  `impl Render`; o dossiê diz "os únicos produtores são `Render` e a expansão do macro", o que está
  correto, mas `from_render` é a porta que um consumidor usaria para um componente próprio.
- O default `type="button"` é sintetizado, mas não há teste de `trybuild` específico para ele;
  o comportamento foi lido no codegen, não exercitado.

## 5. Não verificado (declarado, não inferido)

- `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo doc`, `cargo deny`, `cargo audit`,
  `cargo machete`.
- Miri e fuzz: alvos lidos (`tests/miri_smoke.rs`, `fuzz/fuzz_targets/parser.rs`), não executados
  (exigem nightly/componentes).
- MSRV 1.92: só 1.99 instalado.
- Performance: nenhum benchmark; as metas de `docs/09` e o "~5–15 ms" do `audit` continuam
  declaração do projeto.
- `audit` sobre uma página real: os 18 testes da feature passaram, mas nenhuma página do exemplo foi
  submetida ao `audit`.
- `cargo check` em bloco `html!` grande (200+ linhas): só há o bloco pequeno do exemplo.

## 6. Confiança por eixo

| Eixo | Confiança | Base |
|---|---|---|
| "Não é editor" | 99% | busca por 7 símbolos + leitura de `lib.rs` e da lista de módulos |
| Modelo tipado / escape | 97% | código integral copiado e lido; proptests e doctests executados |
| Vocabulário / spec / paridade | 96% | contagens reexecutadas; `gen_spec --check` e `spec_parity` verdes |
| Exemplo e diagnósticos | 98% | reexecutados, saída byte-idêntica |
| A11y | 90% | código + testes de página/field + `audit` lido; sem leitor de tela real |
| Performance | 35% | nenhuma medição |
| MSRV | 30% | só declaração |

Média ponderada pelo que o dossiê afirma: **88%**.
