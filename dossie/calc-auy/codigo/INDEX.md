# Código copiado — `ref/calc-auy`

Origem: `ref/calc-auy` (workspace `calc-auy-*`), commit
`08f5a3f0a907cf503f8b2afbdae0a19f6618f79f` (2026-10-04), versão `0.1.0`
(`Cargo.toml:21`). Licença: **MPL-2.0** (`Cargo.toml:24`, `LICENSE`).

## Licença e escopo das cópias

MPL-2.0 é *copyleft por arquivo*: copiar arquivos é permitido, e **qualquer
arquivo modificado** derivado deles continua sob MPL-2.0. O projeto **não**
põe cabeçalho de licença por arquivo (nenhum dos seis originais tem SPDX ou
texto MPL; a licença está no `LICENSE` da raiz). Para não criar obrigação
nova nem ambiguidade:

- os seis arquivos abaixo são cópias **verbatim**, sem uma única alteração de
  conteúdo (cabeçalhos `//!` originais intactos);
- **nada** deste diretório é compilado ou linkado pelo `rich-editor-auy`;
- se algum dia um trecho for reaproveitado, o arquivo resultante deve
  continuar sob MPL-2.0 e declarar isso no topo — motivo pelo qual
  `ideias.md` recomenda **reimplementar o conceito**, não copiar o código.

## Arquivos

| Arquivo aqui | Origem | Linhas | Por que foi copiado |
|---|---|---:|---|
| `parser.rs` | `crates/calc-auy-parser/src/parser.rs` | 448 | O parser de precedência que emite **direto na arena**: mostra como `binding_power` derivado de `OperationType::precedence` (`:37-45`), achatamento de cadeias associativas (`:147-192`) e o `%` pós-fixado (`:110-125`) mantêm a AST pequena sem tabela de precedência duplicada. |
| `eval.rs` | `crates/calc-auy-core/src/engine/eval.rs` | 700 | O coração da avaliação: máquina de estados iterativa com pilha explícita (`Action`, `Evaluator`, `step`), memoização de DAG construída só quando há subárvore compartilhada (`build_dag`) e `apply_operation` para cada operação. É o modelo de "sem recursão, sem surpresa de pilha" que um walker de documento também precisa. |
| `canonical.rs` | `crates/calc-auy-core/src/canonical/mod.rs` | 398 | O escritor canônico à mão: JSON compacto, chaves em ordem de bytes, `Rational` como `{"d","n"}` em string, e o **modo** `Trace` × `Value` que decide se metadados entram no hash. É a resposta pronta para "como dois hashes do mesmo conteúdo não divergem". |
| `custody.rs` | `crates/calc-auy-core/src/integrity/custody.rs` | 235 | O tipo `VerifiedAst` com campo privado e o trait `CustodyVerifier`: o padrão *parse, don't validate* em que o motor só aceita o tipo que passou pela verificação. Inclui a checagem estrutural de `cross_context_add`. |
| `wire.rs` | `crates/calc-auy-core/src/ast/wire.rs` | 636 | Projeção recursiva da arena para JSON **e** desserialização com `DeserializeSeed` que impõe `Limits` durante a leitura, além de `deny_unknown_fields` por variante. É o que impede um payload hostil de virar AST antes de ser medido. |
| `html.rs` | `crates/calc-auy-render/src/html.rs` | 225 | O único caminho realmente acessível do projeto: `aria-label` com a descrição verbal, escape de atributo, sanitização de classe CSS e CSS embutido de um asset fixo. Serve de contraste com o SVG, que **não** tem rótulo (ver `arquitetura.md` §9). |

## O que estes seis arquivos provam juntos

1. **O tipo é a fronteira.** `VerifiedAst` (`custody.rs:13-20`) e
   `Committed` (`engine/commit.rs:13-22`) têm campos privados; nenhuma função
   pública aceita a versão "crua". O compilador impede o caminho inseguro,
   não a revisão de código.
2. **A ordem é código, não configuração.** `canonical.rs` escreve cada chave
   na posição alfabética à mão (`{"kind":…,"metadata":…,"op_type":…}`), e
   `wire.rs` lista os campos aceitos por variante (`:22-33`). Renomear um
   campo quebra a assinatura — por isso `trace_version` existe.
3. **Limites são aplicados onde o dado nasce.** `wire.rs` mede durante a
   desserialização; `eval.rs` consulta `Limits` em `apply_operation`; o parser
   usa o `AstBuilder`, que valida ao inserir. Não há um "validate()" final que
   alguém pode esquecer de chamar.
4. **Determinismo é verificável, não aspiracional.** O mesmo `Canonical`
   escrito à mão alimenta `blake3::keyed_hash`; a única entrada variável
   (sal) é explícita, e o relógio/aleatoriedade ficam fora do payload.

## O que **não** está copiado

- `crates/calc-auy-core/src/numeric/rational.rs` (1105 linhas): o racional
  híbrido é interessante, mas é domínio de calculadora, não de editor. Os
  invariantes estão resumidos em `arquitetura.md` §3.1 com âncoras.
- `crates/calc-auy-core/src/error.rs` e `crates/calc-auy/src/error.rs`: o
  modelo RFC 7807 e a tabela de exit codes são citados linha a linha em
  `arquitetura.md` §7; copiar 316 linhas de `thiserror` não acrescentaria.
- `crates/calc-auy-output/src/{output,options,decimal,monetary,json}.rs` e
  `crates/calc-auy-render/src/svg.rs`: descritos com âncoras; a saída real de
  cada um está em `../exemplos/README.md`.
- `crates/calc-auy-wasm/src/lib.rs` (491 linhas) e os processadores binários
  (`WireNode`, Protobuf/CBOR/MessagePack): resumidos em `arquitetura.md` §10.
