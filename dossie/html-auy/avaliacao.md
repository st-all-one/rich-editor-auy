# Avaliação — `html-auy`

Duas perguntas diferentes, duas colunas de nota. "Como editor" pergunta o que o usuário experiencia
ao escrever texto rico. "Como renderer/serializer" pergunta o que o nosso core headless pode aprender.

- Versão avaliada: `0.1.1`, commit `c70213e` (2026-11-07), licença MIT OR Apache-2.0.
- O que foi executado: `cargo check`/`cargo test -p html-auy` (189 testes no total, incluindo 3
  doctests: 186 + 3), `cargo test --features audit`
  (207), `validate_spec.py`, `gen_spec.py --check`, `check_links.py`, 18 testes de ferramenta, dois
  exemplos (um que roda, um que não compila). Transcrições em `exemplos/README.md`.
- O que **não** foi executado: clippy, fmt, doc, `cargo deny`, `cargo audit`, Miri, fuzz, MSRV 1.92,
  benchmark. Marcado como "não verificado" onde aparece.

## 1. Rubrica (0–5)

| Dimensão | Como editor | Como renderer/serializer | Evidência | Comentário |
|---|---:|---:|---|---|
| **Usabilidade** | **0** | **4** | Ausência: 0 ocorrências de `contenteditable`, `struct Selection`, `fn undo`, `collab` em `crates/`. Integração: `exemplos/src/main.rs` rodou; erro vem de `cargo check` em ~2 s | Não há superfície de escrita: não existe documento, cursor, undo, clipboard, IME. Como biblioteca, o fluxo é curto e o erro é acionável — mas exige decorar a sintaxe `attr=(expr)` e as portas únicas (`Form::open`, `Page`) |
| **API** | **0** | **4** | `lib.rs:44-85` (superfície), `markup.rs:11` (tipo opaco), `__private.rs:1-7` ("não é API"), `docs/10-ergonomia.md` | API pequena, tipada e documentada; `#[must_use]` em quase tudo. Perde ponto por: primitivo recebe filhos como **último argumento** (`codegen.rs:107-125`), erro duplicado no mesmo token, e nenhum caminho para markup externo confiável |
| **Neutralidade de estilo** | **0** | **5** | Nenhum `<style>`/CSS emitido; `class` é token (`value/class.rs`); `style=` só como escape hatch auditado (`audit.rs:478-485`); tema é do consumidor (`docs/26:8-30`) | É o ponto mais forte: o crate não tem opinião visual nenhuma. A única classe que nomeia é `sr-only`, e mesmo essa o consumidor precisa definir (`value/text.rs:40-45`) |
| **Modelo de dados** | **0** | **4** | `spec/elements.toml` (111 elementos, 71 invariantes, 13 regras de `audit`), `table.rs:13-33` (`Scope`/`Only` como enums), `audit.rs:21-35` | Não existe modelo de documento (nó/posição/range): o "modelo" é o vocabulário + os valores tipados. Para nós é o padrão a imitar, não o conteúdo |
| **Extensibilidade** | **0** | **3** | 7 traits de contrato (`msg.rs:10`, `value/class.rs:7`, `value/image.rs:82`, `value/url.rs:292`, `value/jsonld.rs:7`, `value/mod.rs:34`, `markup.rs:52`); elemento novo exige TOML + `gen_spec.py` | Trait no ponto certo (locale, tokens, storage, JSON), mas **sem plugin**: não há como registrar elemento/atributo em runtime, e `attrs(…)` arbitrário é proibido por decisão (`docs/02:154-170`) |
| **Performance** | **0** | **3** | Uma alocação por bloco (`codegen.rs:20-28`, `writer.rs:21-25`); `render_to` reusa buffer (`markup.rs:52-70`); metas em `docs/09:15-22` | Desenho de alocação é correto e simples. **Não medido**: não há `criterion` no workspace; "~5–15 ms" do `audit` é declaração de doc, não observação |
| **Acessibilidade** | **0** | **5** | `Img::new` exige `alt`+dims (`media.rs:41-60`); `Video::new` exige captions (`media.rs:438`); `Field` deriva aria (`field.rs:205-241`); `Page` emite skip link/`<main>`/live region (`shell.rs:745-775`); 13 regras de `audit` (`audit.rs:21-35`) | A escada tipo → primitivo → macro → `audit` → gate do consumidor é o melhor artefato do projeto, e a fronteira ("não promete contraste") é declarada (`docs/04:224-235`) |
| **Documentação** | **0** | **5** | 27 capítulos em `docs/`, wiki de 18 páginas em `docs/usage/`, `SKILL.md`, `llms.txt`, `llms-full.txt`, `AGENTS.md` com "regras duras", `CHANGELOG.md` | Documentação de decisão (com números e "o que não promete"), não de marketing. Verifiquei que as contagens citadas batem com o código |
| **Manutenção** | **0** | **4** | `rust-toolchain.toml` (pin 1.99) × `Cargo.toml:21` (MSRV 1.92); lints `deny` de workspace; `#[expect(…, reason)]`; gerador + teste de paridade; fuzz do parser | Rigor de manutenção muito acima da média para 0.1.x. Perde ponto por maturidade real: 0.1.0 e 0.1.1 publicados **no mesmo dia** (`CHANGELOG.md:7,20`), clone raso com um commit, sem histórico de uso em produção que eu possa verificar |
| **Adequação ao nosso caso** | **0** | **4** | Ver §3 | Como editor: nada — o projeto não tem superfície de edição, então a coluna é 0 por definição, não por demérito. Como renderer/serializer: as quatro ideias centrais são aproveitáveis; o vocabulário HTML e o shell HTTP não são |

Nota "como editor" 0 é veredito, não penalidade: o projeto não tenta ser editor e diz isso na primeira
linha (`README.md:14`). A coluna existe porque a rubrica do guia a exige, e porque um leitor pode
confundir "gera HTML" com "edita HTML".

## 2. Cobertura da matriz de usabilidade

Status: **C** = verificado no código · **T** = verificado em teste · **E** = verificado em exemplo
executado · **D** = verificado em doc · **A** = verificado por ausência (busca) · **NV** = não
verificado.

| Item | Status | Evidência |
|---|---|---|
| Instalação e pré-requisitos | **E + D** | `cargo add html-auy` documentado (`README.md:37-44`); build do exemplo funcionou com o crate por caminho; MSRV 1.92 declarada (`Cargo.toml:21`) — **a MSRV em si não foi testada** (só 1.99 instalado) |
| Inicialização mínima | **E** | `html!` de 1 elemento + `Page::new` rodaram; saída real em `exemplos/README.md` §1 |
| Edição básica (texto, parágrafo, negrito/itálico/link) | **A + C** | Não há edição: o vocabulário tem `p`/`strong`/`em`/`a`/`ul`/`ol`/`blockquote`/`pre`/`code` (`spec/elements.toml:331,382,388,534,492,499,432,406,400`), mas só como **saída**; nenhuma API cria ou altera nó |
| Seleção, teclado, undo/redo, clipboard, IME | **A** | 0 ocorrências de `contenteditable`, `struct Selection`, `fn undo`, `struct Editor` em `crates/`; não há listener, DOM ou teclado em lugar nenhum |
| Schema e restrições de conteúdo | **C + T + D** | `spec/elements.toml` é a fonte; 71 invariantes com escopo (`:33-110`); validado por `validate_spec.py` (0 erros) e `spec_parity.rs` (14 testes); exemplo que falha demonstra `div` dentro de `p` recusado |
| Custom node/mark/plugin | **C** | Não no sentido de editor. Extensão existe como trait: `Render` (markup próprio), `Msg` (i18n), `ClassToken` (tokens), `Variant`/`MediaPath` (mídia), `JsonGraph` (JSON-LD). Elemento novo **não** é registrável sem editar o TOML e regerar |
| Persistência e formato de serialização | **E + C** | A "persistência" é `Markup` → `String` (`markup.rs:30-32`); não há formato intermediário, nem JSON, nem parser HTML→modelo. O `audit` parseia com `html5ever` mas devolve só `Finding` (`audit.rs:76-104`) |
| Renderização e não interferência no CSS | **C + T** | `tests/determinism.rs` e `tests/golden.rs` (18 snapshots) provam bytes; nenhum `<style>` é emitido; `style=` sinalizado por `audit`; `class` só como token ordenado |
| Acessibilidade (ARIA, foco, leitor de tela, editável) | **C + T + D** | `tests/page.rs` (24 casos) e `tests/field.rs` (9) cobrem skip link, `<main tabindex="-1">`, `label`/`aria-describedby`; `tests/ui/aria_hidden_focusable.stderr` fixa a recusa; `docs/05`, `docs/acessibility.md`, `docs/23` documentam a fronteira. "Conteúdo editável" **não se aplica** |
| Colaboração | **A** | 0 ocorrências de `collab`, CRDT, OT, websocket em `crates/`. O crate não tem estado, então não há o que sincronizar |
| Performance e limites conhecidos | **C + D, NV em medição** | Desenho de alocação verificado (`codegen.rs:20-28`, `:372-416`); metas e limites declarados (`docs/09`); **nenhuma medição própria** — não rodei benchmark nem instrumentei `audit` |
| Erros, migração e diagnóstico | **E + T + D** | 6 diagnósticos reais capturados em `exemplos/falha-compilacao`; 76 contratos `trybuild`; `docs/10` (ergonomia), `docs/15` (migração de Blade), `docs/17` (FAQ), `docs/11` (estabilidade) |
| Licença e maturidade | **C + D** | `LICENSE-MIT` + `LICENSE-APACHE` na raiz e nos três crates; `Cargo.toml:22`; `CHANGELOG.md` com 0.1.0/0.1.1 no mesmo dia; `deny.toml` com allowlist de licenças |

**Cobertura**: **12 de 13** itens com evidência direta deste dossiê (código, teste, exemplo ou busca
de ausência); o 13º (performance) tem desenho verificado e **medição não executada** — está marcado
como parcial, não como coberto. Itens "A" contam como cobertura porque a ausência foi buscada e
citada (`grep` com 0 ocorrências): num projeto que não é editor, "não existe" é o achado, não a
lacuna. Com 12/13 plenos e 1 parcial, a matriz passa do patamar de **≥ 85%** do guia, com a ressalva
explícita de que performance é a única dimensão sem número medido.

## 3. Valor para o nosso renderer/serializer × valor como editor

### 3.1 O que vale para o `rich-editor-auy` (copiar)

1. **Escape por tipo, não por sítio.** Nosso serializer precisa emitir HTML a partir de nós/marks; o
   equivalente é um `Text`/`Attr`/`Url`/`Json` que carrega o contexto e um tipo de saída
   (`Markup`) sem construtor público. Copiar o desenho de `escape.rs` (3 tabelas) e o `write_with`
   de uma passada é barato e elimina a classe de bug mais comum de serialização.
2. **Vocabulário como dado gerado.** Trocar `if`/`match` de validação por uma tabela (schema de
   nós/marks) + gerador + **teste de paridade** é o que impede a validação de apodrecer. O par
   `gen_spec.py --check` + `spec_parity.rs` é diretamente replicável (em TS/PHP, um `check` no CI).
3. **Validação em camadas com dono declarado.** `block`/`type`/`output` (`table.rs:13-22`) é uma
   resposta pronta para o nosso problema de "quem valida o quê": o schema valida o nó, o documento
   valida ids/estrutura, o navegador valida o resto. Sem escopo declarado, toda regra vira promessa
   vaga.
4. **Ordem canônica e presença booleana.** Para diff/colab, ordem estável e `disabled` sem `="false"`
   reduzem ruído. Aqui vale **adaptar**: no editor, preservar a ordem escolhida pelo usuário pode
   importar mais que `ETag`.
5. **Atributos sintetizados pela semântica** (`rel="noopener noreferrer"` em `target="_blank"`,
   `aria-hidden` em SVG decorativo). No nosso caso: `rel` em link externo, `lang` em bloco citado,
   `alt` herdado. É a diferença entre "o autor lembra" e "o serializer garante".
6. **`Diag` com `regra` + `ajuda`, emitido em lote.** Serve tanto para erro de schema no
   carregamento quanto para o relatório de um `audit` de documento.
7. **Processo**: `#[expect(reason)]` em vez de `#[allow]`, `HashMap` proibido quando a ordem importa,
   fuzz do parser, `trybuild`/snapshot por mensagem. Nada disso exige Rust.

### 3.2 O que vale como editor

**Nada de comportamento.** Não há o que copiar de edição: sem modelo de documento, sem transação,
sem seleção, sem IME, sem undo, sem DOM, sem colab. O único resíduo aproveitável é a **semântica de
saída** (o que um `<strong>`/`<a>`/`<ul>` deve conter, e o que é erro estrutural) — que já é o nosso
schema, não uma feature de editor.

### 3.3 O que evitar

1. **Trazer o vocabulário HTML de 111 elementos para dentro do core.** Nosso schema é de nós/marks;
   duplicar a allowlist HTML cria duas fontes de verdade.
2. **Fechar o core com `no_std`/zero deps por dogma.** O projeto pode porque não faz I/O nem parseia
   nada; um editor headless provavelmente precisa de `serde`/`regex`.
3. **Colocar `Form`/CSRF/`Page`/`Assets` no serializer.** É camada de aplicação HTTP.
4. **Proibir markup externo sem oferecer porta.** O `html-auy` decide que HTML de fora nunca entra
   (`markup.rs:8-11`). Num editor, colar HTML é caso de uso real: precisamos de um caminho explícito
   (sanitizado, com política) em vez de ausência de API.
5. **`defaults` invisíveis.** `type="button"` e `aria-hidden` sintetizados são bons para a11y e ruins
   para depurar: o autor procura no fonte um atributo que só existe na saída. Se copiarmos, o
   inspetor do editor precisa mostrar o atributo sintetizado.
6. **Custo de mudança de 6 passos** (TOML → gerador → testes → 3 docs → contagens). Para o nosso
   schema, que vai mudar muito no início, isso é atrito; a tabela deve ser gerada, mas o caminho de
   edição precisa ser um comando.

## 4. Riscos e limites desta avaliação

| Risco | Situação |
|---|---|
| Maturidade | 0.1.x com dois releases no mesmo dia; API pública pode mudar (`AGENTS.md`, "Em `0.x`, sobe o minor"). Não há evidência de uso em produção no clone. |
| Performance | Nenhum número medido por mim. `docs/09` declara metas e "~5–15 ms" para o `audit`; não reproduzi. |
| `audit` | Os 18 testes da feature passaram, mas não instrumentei o `audit` sobre uma página do exemplo. As 13 regras foram lidas, não exercitadas manualmente. |
| MSRV | 1.92 declarada; só 1.99 está instalada — não verificada. |
| Fuzz/Miri | Alvos lidos (`fuzz/fuzz_targets/parser.rs`, `tests/miri_smoke.rs`), **não executados** (exigem nightly/componentes). |
| Escala | O maior bloco `html!` que testemunhei tem ~10 nós. O custo de `cargo check` em blocos de 200 linhas é afirmação de doc (`docs/09:88-100`), não observação. |
| "Não é editor" | Verificado por busca e por leitura integral de `lib.rs` (85 linhas) e da lista de módulos: `audit`, `escape`, `field`, `form`, `hint`, `markup`, `media`, `msg`, `numeric`, `response`, `shell`, `spec`, `value`, `view`. Nenhum módulo de documento/edição. |

## 5. Confiança

**88%** nas afirmações deste dossiê (após a rodada de revisão, que reexecutou os testes, os dois
exemplos, as três ferramentas de spec e `gen_spec --check`). O que está alto: arquitetura,
vocabulário, escape, a11y, determinismo, ergonomia do erro, contagens de teste e bytes de saída
(lidos no código e reexecutados). O que está baixo: performance medida, MSRV, comportamento do
`audit` em páginas reais, e o comportamento em blocos grandes — os quatro estão marcados como não
verificados em vez de inferidos.
