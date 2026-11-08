# `html-auy` — vocabulário HTML tipado para Rust

> **Não é um editor.** Não existe documento editável, seleção, undo, IME, `contenteditable`, DOM,
> colaboração, parser de entrada nem CSS. É uma biblioteca que **emite** HTML: recebe dados já
> decididos e devolve bytes determinísticos, validando o vocabulário em tempo de compilação.
> Se você procura um editor de texto rico, este projeto não é o caso de estudo — o que ele oferece
> é o **rigor de validação** que um renderer/serializer pode absorver.

- Origem: `ref/html-auy` (`st-all-one/html-auy`), versão **0.1.1**, commit `c70213e` de 2026-11-07.
- Licença: **MIT OR Apache-2.0** (`ref/html-auy/LICENSE-MIT`, `ref/html-auy/LICENSE-APACHE`).
- Tamanho: ~20,6 k linhas Rust (`wc -l` sobre `crates/**/*.rs`), dos quais 6.382 são duas cópias
  geradas da mesma tabela.
- Estado observado: compila e passa em 189 testes (186 + 3 doctests; +18 com a feature `audit`,
  total 207), em 2026-11-07. Ver `exemplos/README.md` §3 para os comandos e a saída.

## 1. Que problema resolve

Gerar HTML em Rust com template de runtime (`format!`, Askama, Tera) deixa três classes de defeito
abertas: **escape errado**, **atributo/elemento errado** e **saída não determinística**. Nesses
modelos, o escape é decidido no ponto de inserção (fácil de esquecer), o vocabulário é texto livre
(erro só aparece no navegador) e a ordem depende do código (o `ETag` oscila).

O `html-auy` move as três para o compilador:

1. **Escape é tipo.** `Text`, `AttrVal`, `Href`, `Src`, `SrcSet`, `JsonLd` carregam o contexto; `&str`
   não implementa `AttrValue`, `String` não constrói `Markup` (`crates/html-auy/src/markup.rs:11`,
   `value/mod.rs:34-44`, `escape.rs:15-62`).
2. **Vocabulário é tabela.** `spec/elements.toml` declara 111 elementos, 71 invariantes e o
   conteúdo-modelo; o macro `html!` recusa o resto com `compile_error!` e span
   (`spec/elements.toml:1-15`, `crates/html-auy-macros-core/src/validate.rs:120-149`).
3. **Bytes são canônicos.** Atributos em ordem da spec, `class` ordenada, `srcset` por largura,
   booleanos como presença (`codegen.rs:255-370`, `writer.rs:67-72`).

Acrescente a quarta: **acessibilidade por ausência de API**. `Img::new` exige `alt` e dimensões;
`Video::new` exige legendas; `Field` deriva `label`/`id`/`aria-*` (`media.rs:41-60`, `field.rs:205-207`).

## 2. Resultado concreto (executado)

```rust
let view = html! {
    section class=(Class::new("card flat")) {
        h2 { (Text::new("Vendas <trimestre> & cia")) }
        a href=(Href::internal(LUrl::new("/relatorios/2026").expect("rota fixa"))) {
            (Text::new("Ver relatório"))
        }
    }
};
```

sai exatamente como

```html
<section class="card flat"><h2>Vendas &lt;trimestre&gt; &amp; cia</h2><a href="/relatorios/2026">Ver relatório</a></section>
```

E o mesmo bloco montado num shell:

```html
<!DOCTYPE html><html lang="pt-BR"><head><meta charset="utf-8">…<title>Demo html-auy</title>
</head><body><div role="status" aria-atomic="true"></div><main id="main" tabindex="-1"><h1>Demonstração</h1>…</main></body></html>
```

Saída real, não ilustrativa: `exemplos/src/main.rs`, rodado com `cargo run --offline`
(transcrição integral em `exemplos/README.md`).

## 3. Uso mínimo

```toml
[dependencies]
html-auy = "0.1"

# opcional: passe de documento inteiro (html5ever) sobre a saída
# html-auy = { version = "0.1", features = ["audit"] }
```

```rust
use html_auy::{html, Href, LUrl, Text};

let link = html! {
    a href=(Href::internal(LUrl::new("/news").expect("rota constante"))) {
        (Text::new("Notícias"))
    }
};
assert_eq!(link.as_str(), r#"<a href="/news">Notícias</a>"#);
```

Requisitos: Rust **1.92+** (MSRV declarada em `Cargo.toml:21`), edição 2024, `no_std + alloc`.
Nesta máquina: `cargo`/`rustc` 1.99.0; a MSRV **não foi verificada** (toolchain 1.92 não instalado).

Regras de superfície que o usuário sente no primeiro dia:

- texto literal no `html!` **não compila** — use `Msg::X` (i18n) ou `(Text::new(&dado))`;
- atributo tipado **não aceita literal** — `href=(Href::…)`, `class=(Class::…)`;
- opcional é `attr?=(Option<T>)`; booleano é presença (`disabled=(true)`) e `disabled="false"` é erro;
- `<form>` não existe no macro: a única porta é `Form::open(&action, Method::Post(csrf), filhos)`;
- `<html>`, `<head>`, `<body>`, `<title>`, `<main>` e `<meta>` não existem no macro: são do `Page`.

## 4. O que dá para copiar para um renderer/serializer

| Absorver | Por que | Onde ver |
|---|---|---|
| Escape escolhido pelo **tipo do valor**, não pelo sítio | elimina a classe de bug "esqueci de escapar" e impossibilita escape duplo | `escape.rs:15-62`, `value/mod.rs:34-44` |
| `Markup` com construtor privado | o bypass deixa de ser proibido e passa a ser inexprimível | `markup.rs:8-11` |
| Validação **data-driven** a partir de uma tabela gerada | regra nova é linha de TOML, não `if` novo; paridade cobrada por teste | `spec/elements.toml`, `tests/spec_parity.rs:1-8` |
| Ordem canônica de atributos + presença booleana | bytes estáveis ⇒ `ETag`/diff/cache confiáveis | `codegen.rs:331-370`, `writer.rs:67-86` |
| `Writer` com `attr_*` que omitem vazio | o caso `class=""`/`sizes=""` deixa de ser possível | `writer.rs:88-110` |
| Atributos sintetizados (`rel="noopener noreferrer"`, `aria-hidden` em SVG decorativo) | semântica exigida não depende do autor lembrar | `codegen.rs:276-289` (SVG), `codegen.rs:296-312` (rel) |
| `Diag` com `regra` + `ajuda` e emissão em lote | erro de vocabulário vira mensagem acionável | `diagnostic.rs:5-60` |
| Traits de contrato (`Msg`, `ClassToken`, `Variant`, `MediaPath`) | o core não conhece o vocabulário do consumidor | `docs/02:113-140` |
| Gates: `gen_spec --check`, `validate_spec`, `trybuild`, `proptest` de escape, fuzz do parser | cada garantia tem o teste que a segura | `docs/12:1-70`, `fuzz/fuzz_targets/parser.rs` |

## 5. O que **não** se aplica ao nosso core headless

| Não trazer | Por quê |
|---|---|
| O vocabulário fechado de 111 elementos | nosso core tem schema próprio (nós/marks); uma allowlist HTML de 38 KB duplicaria o schema |
| `Page`/`ShellData`/`Assets`/`OpenGraph`/hints | é montagem de documento HTTP. Num core headless o serializer emite fragmento; o head pertence ao host |
| `Form`/`Field`/`CSRF` no tipo | validação de formulário é do produto; no nosso caso o equivalente é schema de nó |
| `no_std` como dogma | se o core precisar de `serde`/`regex`, `no_std` vira custo sem retorno |
| `audit` com `html5ever` | útil como *gate de teste*, não como dependência de runtime do core |
| A regra "proc-macro só emite erro" | consequência de Rust stable; em TS/PHP a validação pode ser em runtime com diagnóstico rico |
| `Img` exigindo `width`/`height` **em pixels** | dimensão intrínseca é decisão de mídia; no nosso editor o nó de imagem já carrega `attrs` |
| Byte-canônico global | para HTML de editor, preservar a ordem escolhida pode importar mais que `ETag` estável |

O ponto mais transferível **não** é o vocabulário: é a **separação entre tipo, validação de bloco,
validação de documento e gate de navegador**, cada uma com dono declarado (`docs/04:5-70`).

## 6. Prós e contras observados

**Prós**

- O erro chega no `cargo check`, com span, regra e ajuda — 76 contratos fixados por `trybuild`.
- Impossível emitir `on*`, `<style>`, `srcdoc`, `accesskey`, `javascript:` (tipo + allowlist).
- Saída determinística e legível; `Field`/`Img` produzem o `aria-*` correto sem o autor saber ARIA.
- Zero `unsafe`, zero dependência normal **externa** no crate público (a única normal é o
  `html-auy-macros`, do próprio workspace; `html5ever` só com `audit`), `cargo check` de um exemplo
  em ~2 s.
- Documentação de decisão fora do comum (27 capítulos + wiki + `SKILL.md`), com números que conferem
  com o código (111/71/13).

**Contras**

- **Rigidez é o produto**: para HTML vindo de fora (CMS, markdown de usuário) não há caminho — e é
  intencional.
- Custo de mudança alto: elemento/atributo novo exige TOML + gerador + testes + docs.
- `defaults` implícitos (`type="button"`, `aria-hidden` em SVG) surpreendem quem lê o fonte e não vê o
  atributo.
- Mensagens podem duplicar erro no mesmo token (`alt` em `span` gera 2), e o `unreachable!()` da
  expansão gera warnings que não são do usuário (observado em `exemplos/falha-compilacao`).
- Maturidade baixa: **0.1.0 e 0.1.1 publicados no mesmo dia** (`CHANGELOG.md:7,20`), um único commit
  no clone raso. API pública ainda pode mudar (política `0.x` sobe minor — `AGENTS.md`).
- Sem benchmark versionado; metas de performance são declaração (`docs/09:15-22`).

## 7. Mapa do dossiê

| Arquivo | Conteúdo |
|---|---|
| `arquitetura.md` | modelo tipado, spec gerada, macro, validação em 4 níveis, escape, determinismo, a11y, renderer, extensibilidade, limites, tradeoffs |
| `avaliacao.md` | notas 0–5 com evidência, matriz de usabilidade, valor para renderer × valor como editor |
| `codigo/INDEX.md` + 6 arquivos | `escape.rs`, `markup.rs`, `writer.rs`, `parser.rs`, `validate_excerpt.rs` (trecho), `codegen.rs` |
| `exemplos/` | exemplo que roda (com saída real) + exemplo que **não compila** (diagnósticos reais) |

## 8. Veredito em uma linha

Como **editor**: 0/5 — não é editor e não tenta ser. Como **fonte de rigor para o nosso
renderer/serializer**: 4/5 — o desenho "escape por tipo + vocabulário em tabela + validação em camadas
com dono declarado" é diretamente aproveitável; o vocabulário HTML completo e o shell HTTP não são.
Detalhamento e notas por dimensão em `avaliacao.md`.
