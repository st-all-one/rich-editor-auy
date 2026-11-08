# Ideias para o `rich-editor-auy`, a partir do `calc-auy`

Cada ideia tem: **hipótese** (o que acreditamos que melhora), **evidência** no
`ref/calc-auy` (`caminho:linha`), **encaixe** no nosso desenho (modelo, wire,
erros, import/export, diff, a11y, CLI, MCP), **custo/risco** e **recomendação**
(`agora` = MVP, `pós-MVP`, `rejeitar`).

Referências do nosso lado: `plan/00_panorama.md` (tese), `plan/04_arquitetura.md`
(§8 wire, §10 erros/limites), `plan/06_api-e-integracao.md` (§6 persistência,
§7 erros, §8 a11y), `plan/09_frontend-e-referencias.md` (§3.4 erros, §8 a11y) —
caminhos `plan/…` são relativos à raiz deste repositório; caminhos `crates/…`
são relativos a `ref/calc-auy/`.

Nada aqui defende copiar código: a licença dos dois é diferente (MPL-2.0 lá) e
o domínio também. O que se transfere é **desenho**.

---

## 1. Hash de conteúdo por subárvore, calculado no writer canônico

**Hipótese.** Se cada nó do documento tem um digest estável do seu conteúdo
(canonicalizado), então identidade, memoização de layout, diff estrutural e
"o que mudou nesta transação" saem de graça e com determinismo — sem comparar
JSON inteiro.

**Evidência no `calc-auy`.**
- `crates/calc-auy-core/src/engine/eval.rs:309-340` — `compute_subtree_hashes`
  percorre a arena em ordem de construção e produz `Vec<Digest>` por nó.
- `eval.rs:296-302` — o motor usa esses hashes como chave de cache;
  `engine/cache.rs:1-40` é um `BTreeMap<Digest, Rational>` limitado.
- `canonical/mod.rs:231-301` — o digest deriva de bytes canônicos, não de
  `Hash` de struct, então é estável entre builds.
- `integrity/merkle.rs:1-20` — o mesmo `Digest` encadeia commits.

**Encaixe no nosso core.**
- **Modelo:** `Block`/texto já tem posições; um `content_hash` por bloco (e,
  opcionalmente, por documento) é metadado derivado, nunca campo do wire.
- **Diff:** `ChangeSet` hoje carrega ranges; com hash por bloco, o adaptador
  pode pular blocos não tocados no re-render (hoje ele compara ranges).
- **Wire:** hash não entra no JSON (é derivado). Entra no `Envelope` se
  quisermos detecção de corrupção de armazenamento.
- **Import/export:** ao importar HTML/Markdown, hash por bloco permite
  reimportar e mostrar "nada mudou" sem reconstruir tudo.
- **MCP:** uma tool `doc_hash(block)` responde "este bloco mudou?" sem trazer
  o bloco.

**Custo/risco.** Baixo a médio: um passe pós-`apply` sobre os blocos sujos.
Risco real: hash por bloco não é hash de documento (ordem e concatenação
importam); documentar que é chave de memória/identidade, **não** assinatura.

**Recomendação: agora** (MVP), restrito a um `u64`/`[u8; 16]` por bloco, sem
criptografia. Digest criptográfico só se houver caso de armazenamento.

---

## 2. Escritor canônico com dois modos (`Trace` × `Value`) e envelope versionado

**Hipótese.** O documento precisa de **duas** serializações com propósitos
diferentes: a canônica (para hash, diff, ETag, dedup) e a durável (com
metadados de publicação). Misturar as duas produz hash instável — todo
`doc_id`, timestamp ou seleção muda o hash sem o conteúdo mudar.

**Evidência no `calc-auy`.**
- `canonical/mod.rs:54-66` — `CanonicalMode::{Trace, Value}`; o teste
  `value_hash_ignores_metadata` (`:369-397`) prova que o metadado entra num
  hash e não no outro.
- `integrity/payload.rs:49-70` — o payload assinado é exatamente
  `{ast, final_result, round_strategy, trace_version}`: nada volátil.
- `integrity/payload.rs:110-140` — `SignaturePayloadV2` **adiciona** um campo
  e sobe `trace_version`; política aditiva (ADR-013).
- `canonical/mod.rs:22-50` — escape mínimo e determinístico, escrito à mão.

**Encaixe no nosso core.**
- **Wire:** `Doc::to_json` = canônico, sem `doc_id`, sem `created_at`, sem
  seleção (já é o plano em `plan/04_arquitetura.md:447-485`). `Envelope` =
  `{format, version, schema_fingerprint, doc_id?, doc}` e **nunca** entra no
  hash do `Doc`.
- **Diff:** dois documentos com o mesmo conteúdo produzem os mesmos bytes em
  qualquer máquina — condição para diff textual confiável.
- **CLI/MCP:** `doc hash` responde o digest canônico; `doc save` grava o
  envelope. São comandos diferentes, como `commit` e `sign` lá.
- **Import/export:** exportar duas vezes o mesmo documento deve dar bytes
  idênticos; é o teste que falta hoje.

**Custo/risco.** Baixo (writer pequeno), alto valor. Risco: cada regra de
canonicalização é uma decisão que, errada, quebra todo hash já emitido — por
isso o modo e a versão precisam existir desde o primeiro commit.

**Recomendação: agora.** Pode ser o primeiro artefato do core depois do
`Doc`.

---

## 3. Estados do pipeline como tipos (`Doc` → `Validated` → `Applied`)

**Hipótese.** O pipeline de edição tem estados em que certas operações são
ilegais (documento não validado não serializa; transação não aplicada não tem
`ChangeSet`). Codificar isso em tipos elimina uma classe de bug sem custo de
runtime.

**Evidência no `calc-auy`.**
- `crates/calc-auy-builder/src/calculation.rs:27-49` — `Calculation<S: State>`
  com `PhantomData`; métodos existem por estado.
- `:267-282` — `commit(self, verifier) -> Calculation<Committed>` chama
  `verify_custody` antes de avaliar.
- `crates/calc-auy-core/src/integrity/custody.rs:13-33` — `VerifiedAst` com
  campo privado, construível só por `verify_custody`.
- `crates/calc-auy-builder/tests/compile_fail.rs` + `tests/ui/draft_has_no_result.rs`
  — a garantia é testada por compilação negativa (`trybuild`).

**Encaixe no nosso core.**
- **Modelo:** `EditorState::apply(&self, tx) -> Result<Applied, ApplyError>`
  (`plan/06_api-e-integracao.md:30-63`) já devolve `Applied`, que é o nosso
  "estado pós-verificação". O análogo do typestate é impedir `Doc` de existir
  sem passar pelo `Schema`: `Doc::from_json` devolve `ValidatedDoc`.
- **Adaptador:** a UI não consegue pedir `change_set()` de uma transação não
  aplicada — erro de compilação, não `panic`.
- **Erros:** o typestate reduz `ApplyError` a erros de *input*, porque estado
  inválido deixa de ser representável. Isso simplifica o mapa de erros do MCP.

**Custo/risco.** Médio: `PhantomData` e construtores privados em Rust são
baratos, mas reexportar o tipo "errado" na fachada cria atrito (o `calc-auy`
resolve com `pub use` explícito, `crates/calc-auy-api/src/lib.rs:14-22`). Risco maior: schema é
dinâmico em runtime; não dá para exigir tipo por schema sem gerar código.
Limitar o typestate aos **estágios** (draft/applied), não ao schema.

**Recomendação: agora**, só para `Doc`/`Applied`/`ChangeSet`. Rejeitar
typestate por tipo de nó.

---

## 4. Um contrato de erro, `type` estável e exit codes por classe

**Hipótese.** Se todo erro tem (a) uma variante tipada, (b) um identificador
estável e (c) um código de saída por classe, então CLI, MCP e HTTP testam o
erro de fora, sem depender de mensagem. Erro "stringly" é o que impede um
adaptador de ser confiável.

**Evidência no `calc-auy`.**
- `crates/calc-auy-core/src/error.rs:34-103` — enum `CalcAuyError` com
  variantes de domínio.
- `error.rs:109-160` — `to_problem_details(&dyn Clock)` com `type` URI
  (`https://calc-auy.dev/errors/...`), `status`, `detail`, `instance`,
  `context`.
- `crates/calc-auy/src/error.rs:29-37` — tabela de exit codes: parse 3,
  limite 4, divisão 5, integridade/custódia 6, irracional 7, uso 2.
- Observado na prática (`exemplos/README.md` §3.3).

**Encaixe no nosso core.**
- **Erros:** `ApplyError` com `kind()` estável (`plan/06:337-354` já pede
  isso). Manter o enum como fonte; derivar `type`/`status`/código de saída
  dele — nunca escrever a URI à mão em dois lugares.
- **CLI:** `mcp`/`cli` mapeiam `kind` → código; o teste de integração afirma
  o código, não o texto.
- **MCP:** códigos de erro JSON-RPC são o transporte; `kind` vai em
  `error.data.kind`, e a mensagem pode mudar.
- **Import/export:** `ImportReport` deve ter `kind` por problema
  (`plan/06:337-354`), não uma lista de strings.

**Custo/risco.** Baixo. Risco de over-engineering: não inventar 20 variantes;
começar com as classes que a UI precisa distinguir (input do usuário vs bug do
core vs limite vs schema).

**Recomendação: agora.** É barato e paga em todo teste de adaptador.

---

## 5. Introspecção de capacidades vinda de uma fonte única

**Hipótese.** Listas de capacidades escritas à mão divergem da implementação.
Se CLI `schema`, docs e tool-list do MCP saírem do mesmo registrador, a
divergência vira teste e não bug de usuário.

**Evidência no `calc-auy` (o contraexemplo).**
- `crates/calc-auy/src/commands.rs:316-331` — `schema` tem lista literal de
  `modes` com 7 itens.
- `crates/calc-auy/src/config.rs:32-54` — `FormatMode` tem 10 variantes
  (`json`, `html`, `svg` inclusive).
- Executei: o `schema` imprime 7 modos e o binário aceita 10
  (`exemplos/README.md` §3.7-§3.8). É drift silencioso.
- Contraste positivo: `schema/calc_auy.cddl` + `.proto` + `.schema.json`
  versionados (ADR-013) mostram que o projeto sabe gerar; só não fez para a
  introspecção.

**Encaixe no nosso core.**
- **CLI:** `rich-editor schema` deve emitir `{format, version, schema
  fingerprint, block kinds, marks, commands}`.
- **MCP:** `tools/list` com `inputSchema` derivado do mesmo registrador.
- **Teste:** paridade "todo item do registrador aparece no schema gerado; todo
  item do schema existe no registrador" — o `gen_spec --check` do `html-auy`
  é o modelo de teste.
- **Custo/risco.** Baixo se começarmos com um `const` consumido pelos dois
  lados; gerador completo (codegen) é pós-MVP.

**Recomendação: agora**, na forma mínima: um `Capabilities` derivado de
enums existentes, com teste de paridade. Codegen só quando doer.

---

## 6. Desserialização com orçamento (limites impostos durante a leitura)

**Hipótese.** Abrir documento de terceiro é superfície de DoS. Validar depois
de parsear significa alocar primeiro; impor limites *durante* a leitura recusa
payload hostil antes de construir a árvore.

**Evidência no `calc-auy`.**
- `crates/calc-auy-core/src/ast/wire.rs:1-7` — declara a intenção:
  "impor os `Limits` **durante** a leitura, abortando cedo".
- `wire.rs:172-183`, `:217` — `AstSeed { limits }` como `DeserializeSeed`.
- `wire.rs:22-33`, `:342` — allowlist de campos por variante; campo
  desconhecido é erro.
- `crates/calc-auy-core/src/limits.rs:34-45` — 5 limites com números.

**Encaixe no nosso core.**
- **Modelo/wire:** `Doc::from_json` deve receber `Limits` e falhar com
  `LimitError` por eixo (nós, profundidade, bytes de texto, blocos, tamanho de
  `attrs`).
- **Import/export:** importador de HTML/Markdown usa o mesmo orçamento.
- **Erros:** cada limite é erro com `kind`, nunca truncamento silencioso
  (`plan/06:337-354` já exige).
- **MCP:** um documento grande vindo de tool call não derruba o servidor.

**Custo/risco.** Médio: `DeserializeSeed` por tipo de nó é verboso e fácil de
errar. Alternativa mais barata e quase tão eficaz: pré-varredura de bytes e
contagem de tokens antes do `serde`, mais orçamento no `apply`. O ganho real
do `calc-auy` é **não alocar antes de medir**; se nosso parser já é streaming,
a pré-varredura basta.

**Recomendação: agora** para bytes/nós/blocos via pré-varredura; **pós-MVP**
para profundidade durante o streaming. Não trazer o `DeserializeSeed` inteiro
sem medir o benefício.

---

## 7. Corpus de conformidade como especificação executável

**Hipótese.** Invariantes de schema/steps/wire apodrecem sem vetores
executáveis. Um corpus com entrada, saída esperada e digest canônico
transforma "o formato é assim" em teste de CI.

**Evidência no `calc-auy`.**
- `crates/calc-auy-api/tests/corpus/vectors.json` — 32 vetores em 5 seções
  (`arithmetic` 9, `rounding` 14, `pow` 4, `canonical` 3, `limits` 2); contei
  com script.
- `crates/calc-auy-api/tests/corpus.rs:1-6` — "especificação executável por
  regra"; `CORPUS_UPDATE=1` regenera o digest.
- `docs/book/src/processadores.md:20-26` — `xtask schema` verifica
  idempotência.

**Encaixe no nosso core.**
- **Wire:** 20–30 vetores `{doc, steps, expected_json, digest}` cobrindo
  normalização, autofill, marks adjacentes, limite de 4 KiB de texto.
- **Diff:** vetores de `ChangeSet` esperado para uma transação (insert, split,
  join, mark).
- **Import/export:** vetores de `Doc` → HTML → `Doc` (round-trip declarado, com
  o que é lossy).
- **CLI/MCP:** o mesmo corpus roda pela API e pelo adaptador, provando que os
  dois caminhos concordam.

**Custo/risco.** Médio: curadoria de vetores e política de atualização (senão
o digest vira ruído). Mitigação: `--update` explícito, revisão do diff no
commit, e `version` no corpus.

**Recomendação: pós-MVP** — depois que o wire estabilizar. Antes disso, o
corpus muda toda semana e não protege nada.

---

## 8. `Clock` como porta (e a regra "nada volátil no hash")

**Hipótese.** Tempo e aleatoriedade dentro do documento tornam hash, undo e
teste não determinísticos. Uma porta explícita resolve os dois lados: produção
injeta tempo real; teste injeta `MockClock`.

**Evidência no `calc-auy`.**
- `crates/calc-auy-core/src/clock/mod.rs:1-20` — trait `Clock` com
  `now_rfc3339()` e `next_uuid_v7()`.
- `clock/mod.rs:25-34` — **o contraexemplo**: `SystemClock` devolve
  `"2024-01-01T00:00:00Z"` fixo com `TODO`.
- `crates/calc-auy/src/main.rs:65-72` — `deterministic: true` troca por
  `MockClock`; o `instance` do erro fica reprodutível (observado,
  `exemplos/README.md` §3.4).
- `docs/adr/ADR-014-clock.md` — payload assinado não contém tempo nem
  aleatoriedade.

**Encaixe no nosso core.**
- **Modelo:** `doc_id`, `created_at`, `modified_at` **fora** do `Doc`.
- **Wire:** se entrarem, entram no `Envelope`, nunca no hash.
- **Erros:** `instance`/`request_id` do erro vêm da porta.
- **Testes:** `MockClock` torna snapshots de erro estáveis.

**Custo/risco.** Baixo. O risco é copiar o *stub*: a lição é a porta, não o
`SystemClock` falso. Implementar tempo real de verdade (ou aceitar
`SystemTime` injetado) desde o início.

**Recomendação: agora** (porta + regra de hash). Sem isso, o corpus (§7) fica
instável.

---

## 9. Descrição semântica derivada do modelo, só para átomos

**Hipótese.** Para conteúdo que não é texto (fórmula, imagem, tabela, código),
uma descrição gerada do modelo é melhor que `alt` vazio e mais consistente que
`alt` escrito por autor. Não se aplica a texto comum — lá o texto **é** a
descrição.

**Evidência no `calc-auy` (positiva e negativa).**
- Positiva: `crates/calc-auy-render/src/html.rs:138-181` injeta `aria-label`
  com a descrição verbal gerada; observei a string real em pt-BR
  (`exemplos/README.md` §3.8).
- Positiva: `crates/calc-auy-output/src/output.rs:219-238` — a SRL é derivada
  do modelo, com 9 realizers (`crates/calc-auy-i18n/src/lib.rs:73-93`).
- Negativa: `crates/calc-auy-render/src/svg.rs:88-99` não emite
  `title`/`desc`/`role`; a mesma informação existe mas não é usada.
- Negativa: a NLG é pesada e mora na camada de saída, não no core.

**Encaixe no nosso core.**
- **a11y:** o core entrega semântica (`plan/06:355-378`); o adaptador DOM
  gera `aria-label`/`alt` a partir do atributo declarado + descrição
  derivada (ex.: tabela → "3 linhas, 4 colunas"), nunca CSS.
- **Modelo:** um `describe(node)` opcional, sem dependência de locale no
  core; locale fica no adaptador.
- **Import/export:** ao importar imagem sem `alt`, marcar
  `ImportReport.needs_alt` em vez de inventar texto.
- **Custo/risco.** Médio-alto se virar NLG completo. Limitar a átomos e a
  templates simples; texto livre fica com o autor.

**Recomendação: pós-MVP.** Fazer antes seria construir um gerador de linguagem
para resolver um caso que o `alt` do autor já cobre.

---

## 10. Verificador de custódia do lado do host, como trait

**Hipótese.** Quando o documento referencia algo externo (hoje: PDF/DOI/URL;
no nosso caso: `RefTarget`/id de outra publicação), a política de validação
pertence ao host, não ao core. Um trait pequeno mantém o core determinístico e
testável com um verificador falso.

**Evidência no `calc-auy`.**
- `crates/calc-auy-core/src/integrity/custody.rs:36-52` — trait
  `CustodyVerifier` com uma função.
- `custody.rs:61-75` — o core delega a verificação de cada elo e não conhece
  I/O.
- `crates/calc-auy-wasm/src/lib.rs:36-60` — o host WASM implementa com um
  `BTreeMap` de elos registrados.
- `crates/calc-auy/src/commands.rs` — a CLI usa `AcceptAll` e deixa a
  verificação real para o subcomando `verify`.

**Encaixe no nosso core.**
- **Modelo:** `RefTarget` "preserva, não resolve" (`plan/06:193-270` já diz
  isso). O trait `RefResolver` permitiria ao host decidir dangling/redirect
  sem o core fazer rede.
- **Erros:** referência não resolvida é `ImportReport`/`Warning`, não
  `ApplyError` — o documento continua válido.
- **CLI/MCP:** a CLI usa um resolvedor que só normaliza; o MCP usa o
  resolvedor do servidor.
- **Custo/risco.** Baixo (um trait, uma chamada). Risco: trait com I/O
  implícito quebra o determinismo do core se o resultado entrar no documento.

**Recomendação: agora** (o trait), com a regra dura de que o resultado da
resolução **não** entra no hash do `Doc`.

---

## Rejeitadas (e por quê)

### R1. Log encadeado tipo Merkle para o histórico de edição

O `calc-auy` encadeia commits para detectar reordenação/remoção
(`integrity/merkle.rs:1-20`; não há ADR específico para o encadeamento — o
ADR-014 é o do relógio). **Rejeitar** para undo/redo: o custo é
um digest por entrada de histórico, a política de eviction fica complicada
(`plan/04:486-513` já limita por bytes) e não existe requisito de "provar que
o histórico não foi adulterado" num editor colaborativo — lá quem garante
ordem é o servidor/CRDT, não uma cadeia local. Se um dia houver auditoria de
publicação, o lugar é o `Envelope` de release, não o histórico de edição.

### R2. Embutir decisão de apresentação na string de saída

Todo formato textual do `calc-auy` (menos o decimal) embrulha o resultado em
`round_{NBR-5891}(expr, 2) = 15109.78`
(`crates/calc-auy-output/src/output.rs`; saída real em `exemplos/README.md`
§3). É correto para auditoria e **errado** para nós: no editor, o análogo
seria colar `schema_fingerprint`/`version` dentro do texto serializado, o que
quebra busca, seleção e export. **Rejeitar.** Metadado vai no envelope; a
string carrega conteúdo.

### R3. `f64` proibido por tipo

Boa ideia no domínio deles (`into_rational.rs:7-14`), **sem análogo** no
editor: não temos aritmética de domínio. O que se transfere é o método —
"impeça por tipo, não por convenção" — e ele já está aplicado em `Pos`/`Size`
do nosso lado.

---

## Prioridade final

| # | Ideia | Recomendação | Por que nessa ordem |
|---|---|---|---|
| 1 | Hash por subárvore | **agora** | habilita diff/memo baratos; depende só do writer canônico |
| 2 | Canonical `Trace`/`Value` + envelope | **agora** | pré-requisito de (1) e de qualquer ETag/hash |
| 4 | Contrato de erro + exit codes | **agora** | barato, e todo adaptador depende |
| 8 | Porta `Clock` | **agora** | sem isso (2) e (7) ficam instáveis |
| 10 | Trait de resolução de referências | **agora** | pequeno; evita que o core faça rede |
| 3 | Typestate de estágios | **agora** (escopo mínimo) | melhora a API; só draft/applied |
| 5 | Introspecção de uma fonte | **agora** (forma mínima) | evita o drift que o `calc-auy` tem |
| 6 | Orçamento na desserialização | **agora** (pré-varredura) / pós-MVP (streaming) | segurança de abrir documento de terceiro |
| 7 | Corpus de conformidade | **pós-MVP** | só protege depois do wire estável |
| 9 | Descrição semântica de átomos | **pós-MVP** | NLG é caro; `alt` do autor cobre o MVP |
| R1 | Merkle no histórico | **rejeitar** | sem requisito; custo em undo |
| R2 | Anotação na string | **rejeitar** | quebra busca/export |
| R3 | `f64` por tipo | **sem análogo** | transferir o método, não a regra |
