# Código copiado — `ref/html-auy`

Origem: `ref/html-auy` (repositório `st-all-one/html-auy`, commit `c70213e`, 2026-11-07).
Licença do projeto: **MIT OR Apache-2.0** (`ref/html-auy/LICENSE-MIT`, `ref/html-auy/LICENSE-APACHE`,
declarada em `ref/html-auy/Cargo.toml:22`). Ambos os arquivos de licença existem na raiz e nos três
crates (`crates/html-auy/LICENSE-MIT`, etc.). Como a licença é permissiva, os arquivos completos são
reproduzidos aqui com atribuição, **sem modificação de conteúdo** (exceto onde o índice diz
"trecho"). Nada deste diretório é compilado pelo dossiê; é material de leitura.

| Arquivo aqui | Origem | Linhas | Por que foi copiado |
|---|---|---|---|
| `escape.rs` | `crates/html-auy/src/escape.rs` | 79 | O **único** lugar do crate que decide entidades. Mostra a tese central: o escape é do contexto (`text` × `attr` × `json_script`), não do template. Uma passada, sem alocação intermediária (`write_with`). |
| `markup.rs` | `crates/html-auy/src/markup.rs` | 83 | O tipo que torna o bypass **inexprimível**: `Markup(pub(crate) String)` sem `From<String>`/`Display`. Junto com o trait `Render` (`render_to` + `SIZE_HINT`), é o contrato de composição sem alocação por nó. |
| `writer.rs` | `crates/html-auy/src/__private.rs` | 162 | O alvo de expansão do macro (`Writer`): `open`/`attr*`/`child`/`msg`/`close`. É onde ordem canônica, presença de booleano e omissão de vazio viram bytes. Cabeçalho diz explicitamente "não é API". |
| `parser.rs` | `crates/html-auy-macros-core/src/parser.rs` | 521 | O parser de tokens do `html!`: elemento/atributo/diretiva (`@if`/`for`)/primitivo, com `MAX_DEPTH = 64` e garantia de progresso do cursor. É o que roda no `cargo check` do consumidor. |
| `validate.rs` | `crates/html-auy-macros-core/src/validate.rs` | 1041 | **Trecho**: linhas 1–200 (despacho, escopo de ids/referências, nome acessível) e 430–560 (atributos obrigatórios/permitidos, `role`, booleanos, atributo tipado). O arquivo completo tem 1041 linhas; as omitidas são majoritariamente texto de diagnóstico e conteúdo-modelo. |
| `codegen.rs` | `crates/html-auy-macros-core/src/codegen.rs` | 416 | AST → tokens de escrita: `plan_attrs` (ordenação canônica + atributos sintetizados como `rel="noopener noreferrer"` e `aria-hidden` em SVG decorativo), `emit_element`, estimativa de capacidade do buffer. |

## O que estes seis arquivos provam juntos

1. **O escape vem do tipo** (`escape.rs`), e o tipo de saída não aceita texto de fora (`markup.rs`).
2. **A validação é de dados, não de código**: `spec/elements.toml` → `table.rs` gerado → `validate.rs`
   consulta a tabela. Regra nova não é `if` novo; é linha no TOML.
3. **O byte é o produto**: `codegen.rs::plan_attrs` ordena atributos e sintetiza os que a semântica
   exige; `writer.rs` emite presença em vez de `="false"`.
4. **O parser não pode entrar em pânico**: falha sempre vira `Diag` com span → `compile_error!`
   (`diagnostic.rs:76-84`), e o fuzz alimenta `expand` justamente para isso.

## O que **não** está copiado

- `crates/html-auy/src/spec/table.rs` e `crates/html-auy-macros-core/src/table.rs` (3191 linhas cada,
  gerados por `tools/gen_spec.py` a partir de `spec/elements.toml`). Copiar dois arquivos idênticos de
  3 mil linhas não ensina nada; a fonte normativa é o TOML (cito `spec/elements.toml:<linha>` no
  `arquitetura.md`).
- `shell.rs` (841 linhas) e `field.rs` (578): a API está descrita com âncoras no `arquitetura.md`; o
  exemplo executável mostra a saída real.
- `audit.rs` (506): descrito no `arquitetura.md`, com âncoras.

Ver `../exemplos/README.md` para o que foi **executado** (build, testes do crate, exemplo e exemplo
que não compila).
