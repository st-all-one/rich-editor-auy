# Arquitetura — `html-auy`

> Escopo deste documento: como o projeto pensa, onde cada garantia mora e onde ela **para**.
> Toda afirmação relevante aponta `caminho:linha` do `ref/html-auy` ou um comando observado
> (ver `exemplos/README.md` para as execuções).

## 1. A tese: HTML é artefato, não projeção

A biblioteca trata HTML como **saída determinística de dados já decididos**, não como espelho de
uma árvore reativa. A frase é do próprio README (`README.md:14`) e é executável: não há VDOM,
hidratação, template de runtime, JS embutido, I/O, relógio nem leitura de ambiente.

Consequências de desenho que aparecem no código:

- `#![no_std]` + `extern crate alloc` (`crates/html-auy/src/lib.rs:33-36`) e `#![forbid(unsafe_code)]`
  (`:34`). O crate público **não tem dependência normal externa**; `audit` é a única feature opcional
  e puxa `html5ever`/`markup5ever_rcdom` só quando ligada (`crates/html-auy/Cargo.toml:23-32`;
  o módulo é gateado em `lib.rs:44-45`).
- Não há `build.rs`: a tabela do vocabulário é **gerada e versionada**
  (`spec/elements.toml:3-6`, `tools/gen_spec.py:1-20`).
- O crate não formata texto, não escolhe locale e não lê `std::env` — os três são proibidos por lint
  (`clippy.toml`, seção `disallowed-methods`).

Isso é o oposto de um editor: um editor **precisa** de estado mutável, seleção, IME e DOM. O dossiê
avalia o projeto nesta chave, e é por isso que metade da matriz de usabilidade é "não aplicável".

## 2. Três crates, um pipeline de compilação

| Crate | Papel | Evidência |
|---|---|---|
| `html-auy` | API pública: valores, `Markup`, shell, componentes, `audit` | `crates/html-auy/src/lib.rs:44-85` |
| `html-auy-macros` | `proc-macro` mínima (`11` linhas) | `crates/html-auy-macros/src/lib.rs` |
| `html-auy-macros-core` | parser + validação + codegen, **sem** `proc_macro` | `crates/html-auy-macros-core/src/lib.rs:1-40` |

A separação não é estética. `html-auy-macros-core` é uma biblioteca comum, então pode ser testada e
**fuzzada** sem plugin de compilador: `fuzz/fuzz_targets/parser.rs:1-8` alimenta
`html_auy_macros_core::expand` com tokens arbitrários. O comentário do `lib.rs:12-18` explica por que
`syn` nunca entra: o fluxo é HTML, não Rust; `proc-macro2` + `quote` bastam.

Fluxo de uma expansão (`html-auy-macros-core/src/lib.rs:33-40`):

```
tokens → parser::parse  → Block (AST)
       → validate::check → Vec<Diag>
       → codegen::expand → TokenStream de escrita
```

Qualquer `Err`/`Vec<Diag>` não vazio vai para `diagnostic::emit`, que produz `compile_error!`. Não há
expansão parcial: ou valida tudo, ou nada é emitido (`parser.rs:87-96`).

## 3. Modelo tipado

### 3.1 `Markup` — o bypass é inexprimível, não proibido

```rust
pub struct Markup(pub(crate) String);     // markup.rs:11
```

O campo é `pub(crate)` e **não existe** `From<String>`, `From<&str>` nem `Display`. Os únicos
produtores são [`Render`] (código do crate) e a expansão do macro (`markup.rs:8-11`). Um `String` de
banco/usuário não vira HTML: passar por um valor de contexto é obrigatório.

`Render` (`markup.rs:52-70`) é o contrato de escrita sem template:

```rust
pub trait Render {
    const SIZE_HINT: usize = 0;
    fn render_to(&self, out: &mut String);
    fn size_hint(&self) -> usize { Self::SIZE_HINT }
    fn render(&self) -> Markup { /* uma alocação */ }
}
```

`render_to(&mut String)` existe para o shell reusar um buffer único na página inteira
(`docs/09-desempenho-e-cache.md:1-12`).

### 3.2 `Writer` — o alvo da expansão

`crates/html-auy/src/__private.rs:1-7` diz literalmente "**não é API**": o macro compila para
chamadas a `Writer::open/close_open/attr*/child/msg/close/finish`. Cada método escreve o fragmento
**já canônico**:

- `attr_flag` (`:67-72`) escreve só o nome quando `true` — presença, nunca `="false"`;
- `attr_static` (`:75-86`) trata valor vazio como presença (`hidden`);
- `attr_class` (`:88-92`) omite `class` vazio;
- `msg` (`:159-161`) chama `escape::text` — texto de mensagem nunca entra cru.

Como `Writer` é `#[doc(hidden)]` e o nome do método pode mudar sem aviso, a fronteira macro↔biblioteca
é privada e versionada em par.

### 3.3 Valores: um tipo por contexto de escrita

O escape **não** é escolhido no ponto de inserção; é escolhido na construção do valor
(`docs/02-tipos-e-contextos.md:25-46`). Amostra verificada no código:

| Contexto | Tipo | Garantia | Evidência |
|---|---|---|---|
| texto de conteúdo | `Text<'a>` | escapa 5 + NUL na escrita | `msg.rs:15-40` |
| valor de atributo | `AttrVal<'a>` | escapa 5 + `\r\n\t` | `value/text.rs:12-28` |
| URL de link | `Href` | só `https?`, `mailto`, `tel`, `LUrl`, `Fragment` | `value/url.rs`, `docs/08:23-45` |
| rota interna | `LUrl<'a>` | começa com `/`, sem `//`, sem `:`, sem `..` | `value/url.rs:11-24` |
| recurso | `Src` | interno validado ou `http(s)` absoluto | `value/url.rs` |
| responsivo | `SrcSet`+`Sizes` | ordena por largura, deduplica, emite `Nw` | `value/image.rs:103-124` |
| texto alternativo | `Alt` | `new("")` ≡ `decorative()`; não há default | `value/image.rs:10-33` |
| dimensão | `WidthPx`/`HeightPx` | `0` satura para `1` | `value/image.rs:45-79` |
| JSON-LD | `JsonLd` | `< > & '` → `\uXXXX` | `escape.rs:51-62` |
| classe | `Class`/`ClassList` | ordenada e deduplicada | `value/class.rs:14-74` |
| identificador | `Id`/`FieldName` | normalizados para `[A-Za-z0-9_-]` | `value/text.rs:89-140` |

O trait `AttrValue` (`value/mod.rs:34-44`) é o que torna o tipo obrigatório no macro: `&str` e
`String` **não** o implementam. É por isso que o exemplo que falha produz
`the trait bound '&str: AttrValue' is not satisfied` em vez de um aviso
(`crates/html-auy/tests/ui/attr_value_free_str.stderr:1-15`).

## 4. A spec é a fonte da verdade (validação data-driven)

`spec/elements.toml` (38,5 KB) declara, por elemento: `category`, `void`, `required`, `allowed`,
`accessible_name`, `children`, `required_children`, `first_child`, `forbidden_children`,
`invariants`, `roles`, `only`, `order` (`spec/elements.toml:112-142` documenta cada campo).

Duas tabelas Rust **idênticas** são geradas a partir dele (`tools/gen_spec.py:1-20`) e versionadas:

- `crates/html-auy/src/spec/table.rs`
- `crates/html-auy-macros-core/src/table.rs`

O cabeçalho do gerado diz `GERADO por tools/gen_spec.py — NÃO EDITE`
(`crates/html-auy-macros-core/src/table.rs:1-5`). Sem `build.rs` de propósito: esquecer de rodar o
gerador deixa o CI vermelho (teste de paridade), nunca o produto errado
(`spec/elements.toml:5-6`, `crates/html-auy/tests/spec_parity.rs:1-8`).

Números verificados por contagem no arquivo:

| Grandeza | Valor | Comando/fonte |
|---|---|---|
| elementos | **111** | `grep -c '^\[element\.' spec/elements.toml` |
| invariantes | **71** (59 `block`, 4 `type`, 8 `output`) | `sed -n '/^\[invariants\]/,/^\[globals\]/p' … \| grep -c '= "block"'` |
| regras de `audit` | **13** | `audit.rs:21-35` |
| `only = "shell"` | 9 | `grep -c 'only = "shell"' spec/elements.toml` |
| `only = "component"` | 1 (`form`) | `spec/elements.toml:719` |

O escopo da invariante é a informação que quase toda spec de vocabulário não tem
(`docs/04-validacao.md:64-90`; `table.rs:13-22`): `block` = a macro recusa; `type` = o construtor não
compila sem o dado; `output` = só o HTML final responde. (`spec/elements.toml:33-40` ainda descreve os
escopos como `block`/`page` — nome antigo que o código e o `docs/04` já não usam; deriva de comentário
no upstream, não do comportamento.) `table::Scope` é enum (`table.rs:13-22`), não string: o escopo
entra na API pública (`pub use value::…`, `spec` é módulo público — `lib.rs:59`).

## 5. O macro `html!`

### 5.1 Gramática observada no parser

- elemento: `ident` do vocabulário + atributos + corpo `{ … }`, ou `;` para void
  (`parser.rs:163-231`);
- atributo: `nome=(expr)`, `nome?=(Option<T>)`, `nome="literal"` (`parser.rs:233-286`);
- texto: **literal não é permitido**; `Msg::X`, `(Text::new(..))`, `(markup)` (`parser.rs:469-486`);
- diretivas: `@if cond { }`, `@else if`, `@else`, `for padrao in iteravel { }`
  (`parser.rs:358-467`);
- primitivo: `Form::open(a, b) { filhos }` — os filhos viram o **último argumento** da chamada
  (`codegen.rs:107-125`), e passam pela mesma validação (`validate.rs:105-107`).

Duas salvaguardas estruturais:

- `MAX_DEPTH = 64` (`parser.rs:12`): aninhamento patológico vira `Diag`, não estouro de pilha
  (`parser.rs:99-110`);
- garantia de progresso: se um caminho não consumir token, o cursor avança
  (`parser.rs:140-146`), o que impede laço infinito no parser.

### 5.2 Codegen: ordem canônica e atributos sintetizados

`plan_attrs` (`codegen.rs:255-297`) faz três coisas antes de emitir:

1. coleta os atributos do fonte;
2. **sintetiza** semântica obrigatória: `target="_blank"` sem `rel` ganha
   `rel="noopener noreferrer"` (`codegen.rs:296-312`); `svg` sem nome ganha
   `aria-hidden="true" focusable="false"` (`codegen.rs:276-289`, com a decisão em `:314-330`);
3. ordena por `effective_order` = `order` do spec (quando existe) → `required` → `allowed` →
   globals → `aria-*`, e nome desconhecido vai para o fim (`codegen.rs:331-370`).

A ordem resultante **não depende do fonte**: o mesmo bloco escrito em ordem diferente gera os mesmos
bytes. Isso é testado em `tests/determinism.rs` (`attribute_order_is_canonical`), e o exemplo
executado mostra `class="card flat"` a partir de `Class::new("card flat")`.

Detalhe que só aparece no código: SVG usa `close_open_self` (`/>`), porque no HTML `/>` só é honrado
em SVG/MathML e sem ele um segundo `<circle>` vira filho do primeiro (`codegen.rs:149-153`,
`writer.rs:34-42`).

## 6. Validação em quatro níveis

`docs/04-validacao.md:5-16` fixa a escada:

| Nível | Mecanismo | Falha em |
|---|---|---|
| **N1** tipo | ausência de API / parâmetro obrigatório | `cargo check` |
| **N2** macro | validação no passe da proc-macro | `cargo check` |
| **N4** saída | `html5ever` sobre a página, `feature = "audit"` | CI do consumidor |
| **N5** gate | `axe`, `sniffCSS`, teclado, e2e | CI do consumidor |

O nível intermediário (N3, lint advisory) **foi removido** (`docs/04:5,109-131`): em Rust stable uma
proc-macro só emite erro, não aviso, então "recomendação" virou teste que falha ou default do gerador.

### 6.1 O que a macro vê (N2)

`validate.rs:120-149` é o despacho por elemento, em pré-ordem:

```
check_parent → check_required → check_allowed → check_duplicates
→ check_attr_value (por atributo) → check_children → check_forbidden_descendants
→ recursão → rules::check (regras que precisam de contexto)
```

O que é dado puro vem da tabela (allowlist, obrigatórios, conteúdo-modelo, enums, booleanos); o que
precisa de contexto vive em `rules.rs:1-6` (24 invariantes codificadas, listadas em
`rules.rs:70-98`). O teste em `rules.rs:605-680` classifica toda invariante em `DATA_DRIVEN`,
`BY_CONSTRUCTION` ou `CODED` e falha se alguma ficar sem bucket — nenhuma invariante pode ficar sem
dono.

Fechamento de escopo do bloco (`validate.rs:151-199`):

- `id` repetido no bloco → erro;
- `label for` / `aria-describedby` / `aria-labelledby` apontando para id inexistente → erro;
- controle sem nome acessível → erro, com exceções de `type` (`hidden`, `submit`, …).

### 6.2 Proibições duras

`FORBIDDEN_ATTRS` (`validate.rs:16-29`) fecha: `srcdoc`, `formaction`, `ping`, `slot`, `part`,
`exportparts`, `shadowrootmode`, `is`, `xlink:href`, `accesskey`, `draggable`, `novalidate`; e
`is_forbidden` (`:548`) acrescenta todo `on*`. `style=` **não** está na lista: existe como escape
hatch auditado (`docs/08:46-70`), e o `audit` sinaliza (`audit.rs:478-485`).

### 6.3 Conteúdo-modelo

`check_parent` (`validate.rs:256-340`) combina três mecanismos:

- nome exato em `forbidden_children` do pai (vale só para filho direto);
- categoria permitida (`child_allowed`, `:405-422`) com `flow` absorvendo
  `sectioning/heading/form/phrasing/interactive` (`validate.rs:33`);
- `<a>` transparente: herda o modelo do ancestral não transparente mais próximo (`:272-286`).

E `check_forbidden_descendants` (`:337-369`) aplica categoria proibida em **qualquer** profundidade:
`<a><div><button></a>` não passa.

## 7. Escape por contexto

`crates/html-auy/src/escape.rs` é o **único** lugar que decide entidades (`:1-2`). Três tabelas:

| Função | Contexto | Escapa |
|---|---|---|
| `text` (`:15-25`) | conteúdo de elemento | `& < > " '` e NUL → `&#xFFFD;` |
| `attr` (`:31-45`) | valor entre aspas duplas | os 5 + `\r \n \t` → `&#13; &#10; &#9;` |
| `json_script` (`:51-62`) | corpo de `<script type="application/ld+json">` | `< > & '` → `\u003c …`, mais U+2028/U+2029 |

Implementação: uma passada com `char_indices`, copiando trechos seguros e substituindo só o perigoso
(`:64-79`). Não há buffer intermediário, e não há como esquecer um contexto: a função é escolhida pelo
tipo do valor.

Por que `/` não é escapado: em texto/atributo a barra não fecha nada; a única sequência perigosa
(`</script>`) é fechada pelo escape do `JsonLd` (`docs/02:77-80`). Escapar `/` em todo texto custaria
bytes e não protegeria nada.

Prova executável: `tests/escape_props.rs` usa `proptest` sobre `.*` e afirma que a saída não contém
`<`, `>`, `"`, `'`, NUL, e que todo `&` inicia uma entidade conhecida (`escape_props.rs:1-58`); o
teste de `json_script` faz **round-trip** por `serde_json` e verifica ausência de `</script` e de
U+2028 (`escape_props.rs:60-80`).

## 8. Determinismo

O byte é contrato de produto, não estética. Os mecanismos:

| Mecanismo | Onde |
|---|---|
| atributos em ordem canônica (spec → required → allowed → globals → aria) | `codegen.rs:331-370` |
| `class` literal canonicalizada no codegen | `codegen.rs:186-198` |
| `Class` ordena/deduplica em runtime | `value/class.rs:20-27` |
| `SrcSet` ordena por largura e deduplica | `value/image.rs:105-118` |
| `HashMap`/`HashSet` proibidos por lint | `clippy.toml` (`disallowed-types`) |
| `SystemTime::now`/`Instant::now` proibidos | `clippy.toml` (`disallowed-methods`) |
| capacidade do buffer estimada no fonte | `codegen.rs:372-416` |
| sem indentação, sem whitespace do fonte | `tests/determinism.rs:24-42` |

O objetivo declarado é `ETag` estável (`docs/09:14-30`) e a proibição de "knobs que mudam byte"
(trim, auto-escape, delimitador). O teste `determinism.rs` renderiza o mesmo bloco escrito de três
formas e compara bytes.

## 9. Acessibilidade por construção

A ordem de garantia é explícita (`AGENTS.md`, seção "Acessibilidade";
`docs/04-validacao.md:5-16`):

**tipo/ausência de API → primitivo → macro → `audit` → gate do consumidor.**

Exemplos que são **tipo**, não regra:

- `Img::new(src, alt, width, height)` (`media.rs:41-60`) não tem default: sem `Alt` e sem as duas
  dimensões não compila. `Alt::decorative()` é explícito (`value/image.rs:10-33`).
- `Video::new(src, captions)` (`media.rs:438`) exige `Track::captions`; `Track::descriptions` cobre
  audiodescrição; `Transcript` cobre alternativa textual.
- `Field::text(name, label: M: Msg, state)` (`field.rs:205-207`) deriva `label for`, `id`, `*`,
  `aria-describedby`, `aria-invalid` e `required` do `FieldState` — o consumidor não escreve id à mão.
- `Page` emite skip link, `<main id="main" tabindex="-1">`, `<h1>`, região viva persistente
  `role="status" aria-atomic="true"` e o slot `help` (`shell.rs:745-775`).

Exemplos que são **macro** (N2): `no_aria_hidden_on_focusable` (`rules.rs:115-134`),
`no_aria_label_over_visible_text` (`rules.rs:136-175`), `blank_target_implies_noopener`
(`codegen.rs:296-312`), `select_has_selectable_option`, `caption_required`, `li_only_in_list`.

Exemplos que são **saída** (N4, 8 invariantes): `single_h1`, `no_content_derived_identifiers`,
`ids_unique`, `heading_levels_no_skips`, `landmark_names_distinct`, `landmark_names_required`,
`aria_current_unique_per_nav`, `popover_target_resolves`
(`spec/elements.toml:65-95`, `docs/04:64-90`). `main_unique` **não** está aqui: é escopo `type`,
garantido pelo construtor do `Page`. O `audit` implementa as 13 regras sobre `html5ever`
(`audit.rs:21-35`) e devolve `Finding` em ordem determinística (pré-ordem + ordem fixa de `RULES`).

O que **não** promete (`docs/04:224-235`): contraste, alvo de toque, foco visível, ordem de
tabulação, layout. Esses continuam gates de `axe`/`sniffCSS`/e2e — o crate reduz o custo de chegar
lá, não substitui.

## 10. Renderer / serializer: `Page`, `View`, `Assets`, `Partial`

- `View` = conteúdo do `<main>` + modais + assets (`view.rs:11-18`); `merge` soma modais e deduplica
  assets mantendo o `main` de `self` (`view.rs:54-60`).
- `ShellData` reúne **dados**, não HTML: título, heading, `Lang`, description, canonical, manifest,
  `Assets`, `RootAttributes`, `Regions`, skip, help, noscript, alternates, OpenGraph, hints, JSON-LD
  (`shell.rs:462-485`). `Page::new` monta o documento numa sequência fixa (`shell.rs:671-796`), com
  capacidade pré-calculada — **uma** alocação para a página.
- `Partial`/`Swap`/`MainSwap` modelam troca de região sem emitir `hx-*`: "o crate não faz I/O nem
  escolhe transporte" (`response.rs:1-7`). É o equivalente tipado de swap, para o consumidor
  serializar no seu mecanismo.
- `Assets` deduplica preservando a ordem de declaração (`shell.rs:134-178`), e `Asset::style` só aceita
  caminho interno validado — rastreamento de terceiro não entra por acidente (`docs/08:105-125`).

### CSS: neutralidade real

O crate **não emite CSS nem `<style>`**. `class` é token (`Class`/`Tone`), e o único caminho de estilo
inline é `style=`, permitido mas auditado (`docs/08:46-70`, `audit.rs:478-485`). O tema (tokens, CSS
compilado, hash no nome) é do consumidor (`docs/26-d29-consumidor.md:10-24`). A única classe que o
crate nomeia é `sr-only` (`value/text.rs:40-45`), e mesmo essa exige que o consumidor a defina no CSS.

## 11. Extensibilidade

Não existe plugin system; existem **traits de contrato** onde o vocabulário do consumidor entra:

| Trait | O que o consumidor fornece | Evidência |
|---|---|---|
| `Msg` | catálogo de mensagens resolvido no locale | `msg.rs:10-14` |
| `ClassToken` | enum de tokens do design system | `value/class.rs:7-11` |
| `Variant` | variantes de imagem (`src` + largura) | `value/image.rs:82-90` |
| `MediaPath` | storage de mídia (caminho já validado) | `value/url.rs:292` |
| `JsonGraph` | grafo JSON-LD serializável | `value/jsonld.rs:7` |
| `AttrValue` | valor que assume o próprio escape | `value/mod.rs:34-44` |
| `Render` | primitivo de markup próprio | `markup.rs:52-70` |

O padrão é sempre o mesmo: **o crate não conhece o vocabulário do consumidor, só o contrato**
(`docs/02:113-140`). Um componente novo (`Comp::new(…) { … }`) é aceito pelo macro como "primitivo" e
seus filhos passam por validação normal (`validate.rs:105-107`).

Limite explícito: não há como adicionar **elemento** nem **atributo** novo sem editar
`spec/elements.toml` e regenerar; e não há `attrs(…)` arbitrário — "validação que aceita qualquer
coisa não valida" (`docs/02:154-170`).

## 12. Erros e diagnóstico

`Diag` tem três partes: mensagem (uma linha), `rule` (a invariante) e `hint` (o que fazer)
(`diagnostic.rs:5-40`), renderizadas como `= regra:` / `= ajuda:` (`:50-60`). `emit` gera **todos** os
erros de uma vez e fecha o bloco com `unreachable!()` para que a expansão ainda tenha tipo `Markup` e
não gere cascata (`diagnostic.rs:73-84`).

Contratos fixados por `trybuild`: **76** arquivos `.rs` + 76 `.stderr` em
`crates/html-auy/tests/ui/`, comparados por `cases.compile_fail("tests/ui/*.rs")`
(`crates/html-auy/tests/ui.rs:1-9`). Exemplos reais de mensagem:

```
error: atributo proibido `onclick` em `div`
         = regra: spec:element.div
         = ajuda: estilo é token e comportamento é asset declarado
```

Custo observado (e não documentado pelo projeto): o `unreachable!()` produz
`warning: unreachable statement` nas linhas seguintes e o `use` do bloco que falhou aparece como
"unused" (`exemplos/README.md` §2, item 4). Não impede a leitura, mas polui a saída.

## 13. Performance

Modelo (`docs/09:1-12` + código):

- uma alocação por bloco: `Writer::with_capacity(estimate(block))`
  (`codegen.rs:20-28`, `:372-416`); o `estimate` reserva o **pior caso** de escape de atributo
  literal (`text.len() * 6`, `codegen.rs:390-392`);
- `render_to(&mut String)` no shell reaproveita um buffer para a página inteira
  (`shell.rs:671-680`);
- `Markup::into_string` devolve o `String` sem cópia (`markup.rs:29-33`);
- `audit` custa um parse `html5ever` por página: `docs/04:216-222` declara **~5–15 ms**, rodado no CI
  do consumidor.

Metas declaradas e **não medidas aqui**: view < 300 nós em < 20 µs e < 3 alocações; página completa
< 1 ms (`docs/09:15-22`). Não há `criterion` no workspace; a medição é do consumidor
(`docs/09:107-112`). O dossiê não reproduz benchmark.

## 14. Rigor de engenharia (o que dá para copiar de processo)

| Item | Valor observado | Fonte |
|---|---|---|
| toolchain pin | `1.99` + clippy + rustfmt | `rust-toolchain.toml:1-4` |
| MSRV (contrato) | `1.92`, edition 2024, `resolver = "3"` | `Cargo.toml:15,19-21` |
| lints de workspace | `unsafe_code = forbid`; clippy `all/correctness/suspicious/style/complexity/perf = deny`; `pedantic`/`nursery = warn` | `Cargo.toml:44-78` |
| `clippy.toml` | `HashMap`/`HashSet` proibidos, `now()`/`env::var` proibidos, `disallowed-names`, limiares | `clippy.toml` |
| `deny.toml` | licenças permissivas, `yanked = deny`, `tokio/reqwest/hyper/axum` banidos | `deny.toml:16-45` |
| fuzz | `expand(tokens)` nunca pode entrar em pânico | `fuzz/fuzz_targets/parser.rs:1-18` |
| Miri | `tests/miri_smoke.rs` existe; job de CI | `docs/22:166-190` |
| ferramentas de spec | `validate_spec.py`, `gen_spec.py --check`, `check_links.py`, 18 testes | executados, saída em `exemplos/README.md` §3 |

O ponto de processo mais transferível é a **política de exceção** (`docs/22:192-207`): todo `#[allow]`
exige `reason` (`allow_attributes_without_reason = deny`), e `#[expect]` vira erro quando deixa de ser
necessário — supressão não fica zumbi.

## 15. Limites explícitos (o que o projeto não valida)

1. **Não vê a página inteira no compilador.** Uma proc-macro não vê outra; 8 invariantes ficam em
   `audit` (`docs/04:64-90`).
2. **Não valida semântica humana**: se o `alt` descreve a imagem, se o heading faz sentido
   (`docs/04:224-235`).
3. **Não sanitiza HTML vindo de banco.** Não há parser HTML→modelo: `Markup` impede que texto cru
   vire markup, mas um HTML já pronto de outra origem não passa por aqui (`README.md:84`).
4. **Não decide CSS, contraste, foco, layout, HTTP, locale.**
5. **Não é editor**: não há documento, seleção, undo, IME, `contenteditable` nem colaboração.
   Verificado por busca: zero ocorrências de `contenteditable`, `struct Selection`, `fn undo`,
   `collab` em `crates/`.
6. **Não tem parser de entrada**. O único parse de HTML no projeto é o `audit` (via `html5ever`) e ele
   só produz `Finding`, não árvore editável.

## 16. Tradeoffs, em uma frase cada

| Ganho | Preço |
|---|---|
| Bypass de escape inexprimível | impossível injetar HTML já confiável (ex.: snippet de CMS, SVG externo) sem escrever `Render` |
| Validação em compilação | `cargo check` mais lento em blocos grandes; erro em bloco grande recompila unidade grande (`docs/09:88-100`) |
| Tabela gerada e versionada | mudança de vocabulário exige rodar Python e commitar 3 mil linhas geradas; sem `build.rs` |
| Ordem canônica e presença booleana | o autor não controla a ordem dos atributos no fonte; `disabled="false"` é impossível — inclusive quando se *queria* `false` (tem de ser `Option<bool>`) |
| A11y por ausência de API | `Img` sempre exige `alt` e dimensões: ícone decorativo de 1 px também precisa declarar as duas |
| Sem runtime/VDOM | atualizar um nó exige reconstruir o bloco e trocar a região (`Partial`); não há diff |
| `no_std`, zero deps | sem `serde`, sem `regex`, sem `url` — toda validação de URL e de data é escrita à mão |
