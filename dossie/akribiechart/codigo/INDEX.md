# `codigo/` — origem, licença e por que cada arquivo está aqui

**Origem de todas as cópias**: `ref/akribiechart`, commit
`7275fedfe81f2d4ac1df4e92a580c05b90a81049` (2026-11-03), versão de workspace `0.1.3`.

**Licença do projeto**: **MIT OR Apache-2.0** (`ref/akribiechart/Cargo.toml:19`; texto integral em
`ref/akribiechart/LICENSE`). Isso permite copiar arquivos completos com atribuição — foi o que fiz.
Os dois arquivos `.ttf` de `assets/` são **OFL-1.1** e não são código; **não** foram copiados.

**Estado das cópias**: **verbatim**. Nenhum byte foi alterado, reordenado ou comentado. Por isso os
`sha256` abaixo servem como prova de identidade: quem quiser conferir roda
`sha256sum ref/akribiechart/<origem>` e compara. A atribuição e o aviso de licença ficam **aqui** e
em [`NOTICE.md`](NOTICE.md), não no topo dos arquivos — justamente para preservar o verbatim.

Os arquivos-fonte **não** têm cabeçalho SPDX nem comentário de licença
(`grep -rl "SPDX" ref/akribiechart/crates/` → vazio); todos abrem com um doc-comment `//!` que
descreve o módulo. Esse cabeçalho foi preservado como está.

---

## Índice

| cópia | origem | linhas | sha256 | por que foi copiado |
|---|---|---|---|---|
| [`pipeline.rs`](pipeline.rs) | `crates/akribie-core/src/pipeline.rs` | 214 | `67adde1d05a975df75579a9a7cbce6436d3a969428b541c80d27a9b8083ba597` | **O coração da composição**: mostra `check` como seis chamadas puras, cada regra de validação como função independente que devolve `Diagnostic`, e a ordenação determinística no fim. É o modelo que queremos para "validar sem renderizar". Também traz `batch_lines`, o parser JSONL sem I/O. |
| [`diag.rs`](diag.rs) | `crates/akribie-core/src/diag.rs` | 172 | `8ede7c5e7e459673591e19bdf1ec4e749a69b9829ef5f1ddd413cfa119a6a0d3` | **Erro como dado.** 16 códigos estáveis numa tabela `const ALL: [Code; 16]` que alimenta `check`, `akr schema` e `akr prime`. `Diagnostic` com `span`/`message`/`hint`, `render()` textual e `to_json()`, e `sort()` por `(linha, coluna, código)`. É a forma exata do nosso `Diagnostic`. |
| [`fmt.rs`](fmt.rs) | `crates/akribie-core/src/fmt.rs` | 170 | `83dc354d5a410ceb532ee1ad707e086ea63939b3989c81990b70bb2ec46cd81d` | **Forma canônica única e idempotente.** As oito regras estão enumeradas no doc-comment (`:3-13`) e implementadas em ~150 linhas: ordem de classes, uma linha em branco entre classes, opções em ordem alfabética, números canônicos. É o análogo direto do nosso serializador canônico. |
| [`a11y.rs`](a11y.rs) | `crates/akribie-core/src/a11y.rs` | 189 | `815bd93886d77a04b7059f5c54e637fb1ba56b799f564ea5b38a11e54b20e17a` | **Acessibilidade derivada do modelo.** `describe()` monta contagem + rótulos + relações a partir do `Chart` (nunca da geometria) e `check_contrast()` emite `W007` percorrendo nós, grupos e arestas. Mostra como transformar a11y em diagnóstico testável — e onde o idioma fixo vira defeito (`:30-41`). |
| [`emit-svg.rs`](emit-svg.rs) | `crates/akribie-core/src/emit/svg.rs` | 245 | `c32eff6e1f2806c644976af01aeda582c939117efdb9ab0f40d9065588e64fcb` | **Emissão determinística e acessível.** Ordem fixa de seções, `<title>`/`<desc>` como primeiros filhos, `aria-label` por figura, todo número por `num::format`, todo texto por `escape`. Inclui a nota de que a alternativa de escrita foi medida e empatou (`:10-13`). |
| [`diff-mod.rs`](diff-mod.rs) | `crates/akribie-core/src/diff/mod.rs` | 256 | `56e2f74b73893ffe07fc94d292cb1576f636787032e10903294a8d7bd2844fa5` | **Diff semântico.** Identidade explícita por tipo de elemento (nó por `id`, aresta por `(from,to,ordinal)`, nota por âncora + posição), saída agrupada e ordenada, e a estrutura `Change { kind, element, field, before, after }` com `line()` para texto. É o desenho do nosso change set. |

Total: **1 246 linhas** de código Rust real, verbatim.

### O que ficou de fora, e por quê

O guia pede 4–6 arquivos substantivos, não o projeto inteiro. Ficaram fora, com a citação no
[`../arquitetura.md`](../arquitetura.md):

- `lex.rs` (292 linhas) e `parse/` — o tokenizador é específico de uma gramática de uma linha por
  afirmação; o *princípio* (tokens com `Span`, recuperação por linha) está descrito em §4.1–4.2.
- `model/mod.rs` e `model/enums.rs` — o modelo de quatro camadas está mapeado em §5 com as linhas.
- `layout/` (15 arquivos) — o algoritmo é dagre/portes geométricos; nada disso transfere para um
  editor de texto.
- `emit/excalidraw`, `emit/scene_json`, `emit/raster` — formatos e codecs específicos.
- `commands/`, `mcp/` — adaptadores; descritos em §9–§10 com as linhas dos pontos que importam.
- `theme/mod.rs` — a paleta fechada é o exemplo de "o que não copiar" e está citada em §7.

## Como verificar as cópias

```sh
cd ref/akribiechart
sha256sum crates/akribie-core/src/{pipeline,diag,fmt,a11y}.rs \
          crates/akribie-core/src/emit/svg.rs \
          crates/akribie-core/src/diff/mod.rs
```

Os seis hashes devem bater com a tabela acima. Foi assim que este índice foi produzido — não por
confiança.
