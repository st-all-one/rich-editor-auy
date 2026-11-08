# 02 — Decisões: opções, recomendação e o que cada uma previne

> **Como ler.** Cada decisão tem um nível de bloqueio e um status. O status `aceita` significa
> "o dossiê adota como base; mudar exige nova entrada em `03_decisoes-fechadas.md`". O status
> `proposta` significa "recomendação forte, ainda sujeita a revisão". O status `adiada` significa
> "pós-MVP, mas o desenho atual não pode impedir".
>
> **Ratificação de 2026-11-08.** Todas as decisões não adiadas foram aceitas por instrução do
> projeto; as que continuam `adiada` são D08, D26, D28, D34, D40 e D50. A avaliação determinística
> das estruturas de dados está em §K; o registro fechado está em
> [`03_decisoes-fechadas.md`](03_decisoes-fechadas.md) §C.
>
> **Legenda de bloqueio:** 🔴 decide antes de escrever código do núcleo · 🟠 decide antes da feature
> que depende dela · 🟡 pode esperar, mas fica registrado.

---

## A. Modelo e representação

| # | Bloqueio | Status | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|---|---|
| **D01** | 🔴 | aceita | Forma do documento | (a) árvore genérica de nós; (b) árvore de blocos + inline plano; (c) lista flat de blocos; (d) rope de eventos | **(b)** — árvore de blocos com *textblocks* de conteúdo inline plano. É o modelo do ProseMirror (guia oficial) e o que dá posições por offset dentro do parágrafo. | Árvore de marcas, O(n) para aplicar marca, ambiguidade de representação, renderização que precisa reconstruir runs |
| **D02** | 🔴 | aceita | Ownership e snapshots | (a) clonar o doc a cada transação; (b) `Arc<DocInner>` + COW com `try_unwrap`/`make_mut`; (c) arena mutável + snapshots; (d) vetor persistente | **(b)** — o caminho comum (dono único) muta no lugar; se a UI retém um snapshot, o COW copia o vetor de `Arc<Block>` (ponteiros), não os blocos. | Cópia O(documento) no caminho comum; `unsafe` de arena; complexidade de vetor persistente no MVP |
| **D03** | 🔴 | aceita | Armazenamento de texto | (a) um `String` por textblock; (b) `String` por run com teto de 4 KiB; (c) gap buffer; (d) rope; (e) piece table | **(b)** — runs canônicos com teto de 4 KiB, divisão em fronteira de escalar. O teto limita o memmove de uma inserção e o número de runs por bloco (≈256 num bloco de 1 MiB). | Memmove de 1 MiB por tecla em parágrafo gigante; complexidade de rope/gap buffer no MVP; alocação por caractere |
| **D04** | 🔴 | aceita | Unidade de posição | (a) byte UTF-8 + tokens estruturais; (b) escalar Unicode (`char`); (c) unidade UTF-16; (d) grapheme cluster | **(a)** — `Pos(u32)` em bytes UTF-8, com 1 posição por token estrutural (abertura/fechamento/átomo). Conversão explícita para UTF-16 e para grapheme na borda da UI. | Indexação O(n) por posição; divergência Rust/JS; confusão entre offset de bytes, code units e grafemas |
| **D05** | 🔴 | aceita | Identidade de nó | (a) sem IDs; caminho/posição; (b) âncoras estáveis opcionais no doc; (c) IDs efêmeros só na view | **(a)** por padrão + **(b)** `NodeId` opcional para referência entre publicações + **(c)** IDs de view pós-MVP. A âncora é atributo reservado, dada pelo host, preservada e determinística; não é ID de CRDT nem identidade de UI. | Sem ID não há referência estável; IDs em todo nó incham o wire; confundir âncora com identidade de UI/CRDT |
| **D06** | 🟠 | aceita | Estrutura do topo | (a) `Vec<Arc<Block>>`; (b) `Vec<Block>` inline; (c) `ChunkedVec`/RRB persistente | **(a)** no MVP; medir COW em 100k blocos; **(c)** só se o orçamento estourar. | Cópia de conteúdo de blocos; complexidade prematura de vetor persistente |
| **D07** | 🟠 | aceita | Limites | (a) sem limites; (b) limites fixos no código; (c) `Limits` configurável com defaults | **(c)** — doc 64 MiB, texto por bloco 1 MiB, profundidade 64, nós 1M, attrs 32, runs/bloco 4096, attr string 64 KiB. Toda construção/parse valida. | OOM, stack overflow, input hostil, documento “válido” que derruba a UI |
| **D08** | 🟡 | adiada | Codec binário | (a) JSON só; (b) JSON + binário no MVP; (c) JSON + binário pós-MVP | **(c)** — JSON é o contrato; binário é otimização posterior atrás de um trait `Codec`. | Escopo duplicado de serialização no MVP |

### D01 em uma frase
Conteúdo inline é uma **sequência plana de runs com marcas como metadado**, não uma árvore de
marcas. Isso é o que permite posições baratas, aplicação de marca por split de run e renderização
direta. O preço é que conteúdo inline aninhado (um link contendo um bloco) não é representável; se
o consumidor precisar, isso é um átomo/bloco com conteúdo próprio.

### D02 em uma frase
O documento é imutável **para o consumidor**, mas o caminho comum não paga clone: `Doc` é um
`Arc<DocInner>`; quando a contagem é 1, a transação muta o vetor no lugar; quando a UI guardou um
snapshot, o COW copia o vetor de ponteiros e o bloco tocado. É o melhor dos dois mundos sem
`unsafe` e sem vetor persistente.

### D03 em uma frase
Runs de texto têm teto de 4 KiB. O teto não é semântico: é uma **invariante canônica** que limita
o custo de inserção e o número de runs por bloco. Sem ele, um parágrafo de 1 MiB paga memmove de
1 MiB por tecla; com ele, paga no máximo 4 KiB.

### D04 em uma frase
A posição é um byte UTF-8 (ou um token estrutural). A UI converte para a unidade dela — UTF-16 no
navegador, `char` no terminal, o que for — com funções explícitas. O core não finge que existe uma
unidade universal.

---

## B. Schema

| # | Bloqueio | Status | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|---|---|
| **D09** | 🔴 | aceita | Fonte do schema | (a) builder de runtime só; (b) macro só; (c) builder de runtime + `schema!` como front-end | **(c)** — o builder é a fonte; a macro valida em compilação e emite o mesmo builder. Consumidores que carregam schema dinâmico usam o builder. | Duas fontes de verdade; impossibilidade de schema dinâmico; falha em runtime onde a macro poderia ter falhado em compilação |
| **D10** | 🔴 | aceita | Expressões de conteúdo | (a) lista de filhos permitidos; (b) regex; (c) gramática PM-like compilada para autômato | **(c)** — `name`, grupos, alternância, `*`, `+`, `?`, `{n}`, `{n,}`, `{n,m}`, parênteses; NFA→DFA; checagem estática de produtividade (sem ciclo obrigatório) e regra do “primeiro tipo” para autofill. | Stack overflow do autofill recursivo (o guia do PM alerta); conteúdo inválido; ambiguidade de filler |
| **D11** | 🔴 | aceita | Representação de marcas | (a) `Vec<Mark>` por run; (b) máscara `u64` + attrs; (c) `MarkSetId` internado | **(b)** — máscara `u64` (até 64 tipos de marca, ordem do schema) + `Option<Box<[MarkAttr]>>` para marcas com atributos (link). `excludes` e `inclusive` declarados no schema e aplicados na edição. | Alocação por run; ordem não determinística; marcas conflitantes (link dentro de link, code com bold) |
| **D12** | 🔴 | aceita | Atributos | (a) `HashMap<String, Value>`; (b) pares ordenados por declaração; (c) JSON arbitrário | **(b)** — `Attrs` como pares `(AttrId, AttrValue)` na ordem de declaração; `AttrValue` fechado em `Null`/`Bool`/`Int`/`Str`/`Array`/`Ref(RefTarget)`; `Ref` é o único objeto aninhado (chaves fixas), sem float, sem objeto arbitrário; defaults e validação por attr. | Ordem instável; float não determinístico; atributo inventado; objeto aninhado sem schema |
| **D13** | 🟠 | aceita | Fingerprint do schema | (a) sem fingerprint; (b) hash do texto canônico; (c) versão manual | **(b)** — **BLAKE3-256** da serialização canônica da declaração, com `derive_key("rich-editor-auy/schema/v1")` (domain separation); 32 bytes em hex minúsculo (64 caracteres) + versão do formato. É fingerprint, **não** assinatura (D64); não usar FNV nem SHA-256. | Carregar doc com schema incompatível em silêncio; hash instável entre versões; confundir fingerprint com integridade |
| **D14** | 🟠 | aceita | Tipo desconhecido no load | (a) ignorar; (b) erro total; (c) erro com path + modo lossy explícito | **(c)** — default estrito com `SchemaMismatch { path, kind }`; `from_json_lossy` substitui por um `Unknown` configurado e devolve `ImportReport`. | Perda silenciosa; derrubar a leitura inteira por um nó futuro |
| **D15** | 🟠 | aceita | Flags de nó | (a) só `inline`/`atom`; (b) incluir `defining`/`isolating`; (c) incluir semântica de UI (`draggable`, `selectable`) | **(b)** — `defining` e `isolating` entram no schema e no replace do F2; flags de UI ficam fora do core. | Split/join imprevisível em nós “transparentes”; core invadindo a UI |

### D09 em uma frase
O schema de runtime é a verdade porque o consumidor pode carregar um schema de um servidor ou
compor extensões em runtime. A macro é um **verificador de compilação** e um gerador de acessores
tipados; ela não pode ser a única porta, ou o projeto deixa de ser agnóstico.

### D10 em uma frase
Expressões de conteúdo são uma gramática pequena, mas precisam ser **compiladas com checagem de
terminação**: um schema em que todo `block` exige outro `block` é um bug de schema, não um stack
overflow em runtime. O autofill é determinístico e documentado.

### D11 em uma frase
A máscara de marcas é o equivalente inline do `MarkSet` do ProseMirror: 8 bytes para o caso comum,
zero alocação para runs sem marca com atributo. O limite de 64 marcas é um contrato de schema, não
um detalhe invisível.

---

## C. Steps, transforms e seleção

| # | Bloqueio | Status | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|---|---|
| **D16** | 🔴 | aceita | Representação de `Step` | (a) trait object (`Box<dyn Step>`); (b) enum fechado; (c) log de comandos | **(b)** — 6 variantes: `Replace`, `ReplaceAround`, `AddMark`, `RemoveMark`, `SetNodeAttr`, `SetDocAttr`. `Copy` onde couber; serialização direta; `match` exaustivo. | Alocação e dispatch por step; variantes não previstas; serialização por downcast |
| **D17** | 🔴 | aceita | `Slice` | (a) fragmento fechado; (b) fragmento + `open_start`/`open_end`; (c) texto puro | **(b)** — `Slice { content: Fragment, open_start: u8, open_end: u8 }`; validar profundidades contra a árvore. | Split/join inválido; colar no meio de um nó; panic nas bordas |
| **D18** | 🔴 | aceita | `Mapping` | (a) soma de deslocamentos; (b) `StepMap` com ranges e viés; (c) OT completo | **(b)** — ranges `(start, old_len, new_len)`, `assoc` -1/+1, mirror/recover; aritmética checada. O OT completo fica fora do core. | Seleção que “pula”; posição perdida no undo; rebase incorreto |
| **D19** | 🔴 | aceita | Atomicidade | (a) aplicar step a step e deixar estado parcial; (b) transação atômica | **(b)** — todos os steps no working copy; falha em qualquer um devolve o estado original e um erro tipado. | Transação pela metade; histórico inconsistente; UI que não sabe o que aconteceu |
| **D20** | 🟠 | aceita | API de estado | (a) `&mut EditorState`; (b) `apply(&self, tx) -> Result<Applied, ApplyError>`; (c) `apply_owned` | **(b)** — imutável para o consumidor; COW interno. `apply_owned` pode ser otimização posterior, não API inicial. | Aliasing/mutação escondida; dificuldade de snapshot/thread |
| **D21** | 🟠 | aceita | Seleção | (a) só caret; (b) `Caret`/`Text`/`Node`/`None` com `assoc`; (c) multi-seleção nativa | **(b)** no MVP; **(c)** pós-MVP. `assoc` explícito em toda posição de borda. | Seleção ambígua no limite de inserção/deleção; divergência com a UI |
| **D22** | 🟠 | aceita | `storedMarks` | (a) marcas só no doc; (b) conjunto “marcas do próximo caractere” no estado | **(b)** — togglear marca sem seleção precisa de estado; digitar usa `storedMarks`. | “Apertei bold e não aconteceu nada”; toggle inconsistente |
| **D23** | 🟠 | aceita | Normalização | (a) só na construção; (b) após cada step, local; (c) validação total após cada step | **(b)** — normaliza a região afetada; `normalize` é pura, total e idempotente; load valida o doc inteiro. | Doc inválido acumulando; O(doc) por tecla |
| **D24** | 🟠 | aceita | Autofill | (a) não preencher e falhar; (b) preencher com o primeiro tipo válido; (c) preencher com um tipo “default” especial | **(b)** com regra explícita; schema recusa conteúdo obrigatório sem filler possível. | Stack overflow; contêiner vazio que colapsa na UI; filler surpresa |
| **D25** | 🟠 | aceita | ChangeSet | (a) doc inteiro; (b) ranges antigos/novos por bloco + motivo; (c) diff textual | **(b)** — derivado dos steps; a UI invalida só os blocos/runs afetados. | Re-render total; diff caro; perda de informação de seleção |
| **D26** | 🟡 | adiada | Steps no wire | (a) só doc; (b) steps versionados para log/colaboração | **(b)** — steps serializáveis desde o início, com `version`; o core não implementa o transporte. | Log de undo opaco; colaboração impossível sem reescrever |

### D16 em uma frase
Um enum fechado de seis variantes cobre o que um editor precisa e permite `match` exaustivo,
serialização sem `downcast` e zero alocação por step. Se um dia um consumidor precisar de um step
customizado, ele pode expressá-lo como `Replace`/`SetNodeAttr`; um trait object pode ser uma
extensão posterior, não a base.

### D18 em uma frase
Mapear posição não é somar deltas: uma posição na borda de uma inserção pode ficar antes ou depois,
e uma posição dentro de conteúdo deletado precisa de informação de recuperação. `assoc` e
`Mapping` são o contrato; testes de propriedade são obrigatórios.

### D19 em uma frase
Uma transação é uma **unidade atômica de intenção**: ou todas as mudanças entram, ou nenhuma. Isso
é o que permite a UI dizer “apliquei” sem precisar reconstruir o estado em caso de erro.

---

## D. Histórico, comandos e entrada

| # | Bloqueio | Status | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|---|---|
| **D27** | 🔴 | aceita | Agrupamento de undo | (a) por tempo; (b) por transação; (c) `GroupId` explícito + política determinística | **(c)** — a UI/entrada fornece `GroupId` para uma rajada de digitação; sem `GroupId`, a política agrupa inserções/deleções adjacentes no mesmo bloco. Nada de relógio. | Undo que depende de wall clock; testes flaky; agrupamento surpresa |
| **D28** | 🟡 | adiada | Undo seletivo | (a) linear; (b) seletivo com rebase | **(a)** no MVP; **(b)** pós-MVP, com testes diferenciais. | Complexidade de rebase antes de haver colaboração |
| **D29** | 🟠 | aceita | Memória do histórico | (a) snapshots de doc; (b) steps inversos; (c) steps + orçamento | **(b)** + **(c)** — steps inversos; teto de 200 grupos/8 MiB, evicção do mais antigo. | Memória crescendo com o documento; snapshot por tecla |
| **D30** | 🟠 | aceita | Comandos e plugins | (a) registry genérico de plugins; (b) comandos como funções/valores + hooks declarativos; (c) comandos só embutidos | **(b)** no MVP; registry genérico só com evidência. | `Any`/`Box<dyn>` sem necessidade; ciclo de vida de plugin; API pública instável |
| **D31** | 🟠 | aceita | Input rules | (a) `regex` no core; (b) matcher literal/prefixo; (c) sem input rules | **(b)** — `# `, `> `, `- `, `1. `, `**` etc. por prefixo; regex é feature opcional. | Dependência pesada; backtracking catastrófico; input rules não determinísticas |
| **D32** | 🟠 | aceita | Keymap | (a) keymap de SO no core; (b) `Key`/`Mod` declarativo, UI traduz; (c) sem keymap | **(b)** — o core descreve `Mod-b`, `Shift-Enter`; a UI traduz o evento. Nenhum atalho global. | Acoplamento a SO; atalhos que colidem com o host; acessibilidade |
| **D33** | 🟠 | aceita | IME/composição | (a) UI resolve tudo; (b) core recebe transações de composição com metadados; (c) core chama APIs de IME | **(b)** — `CompositionStart/Update/Commit/Cancel` viram metadados/histórico; a UI dirige os eventos. | Undo de estados intermediários; desync entre composição e doc |
| **D34** | 🟡 | adiada | Multi-seleção | (a) uma seleção; (b) `SelectionSet` desde o início | **(a)** no MVP; API preparada para `SelectionSet` depois. | Complexidade de mapear N seleções e comandos em todas |

### D27 em uma frase
O histórico não pode depender de `Instant::now`. O agrupamento é uma decisão da entrada (a UI sabe
que uma rajada de teclas é uma rajada) e uma política determinística do core quando não há
`GroupId`. Isso torna undo testável e reproduzível.

### D33 em uma frase
IME é um protocolo, não uma API do sistema. A UI recebe `compositionstart/update/end` do host e
emite transações marcadas; o core agrupa no histórico e não registra estados intermediários. O
core nunca chama o SO.

---

## E. Serialização e interop

| # | Bloqueio | Status | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|---|---|
| **D35** | 🔴 | aceita | Formato canônico | (a) JSON compacto; (b) JSON pretty; (c) binário; (d) ambos | **(a)** — JSON compacto, UTF-8 cru, inteiros, ordem de schema; pretty é uma saída humana **não canônica**. | Hash instável; diff ruidoso; floats; ambiguidade de whitespace |
| **D36** | 🔴 | aceita | Codec JSON | (a) `serde_json` obrigatório; (b) writer próprio + parser próprio opcional; (c) writer próprio + `serde` feature | **(b)** com `serde` como feature separada. O writer é sempre nosso; o parser atrás de `json`; testes diferenciais contra `serde_json` em dev. | Dependência obrigatória; perda de `no_std`; erros opacos; parser não testado |
| **D37** | 🟠 | aceita | Envelope persistido | (a) nó JSON puro; (b) envelope com versão/fingerprint; (c) ambos | **(c)** — `Doc::to_json` para interop; `Envelope { format, version, schema_fingerprint, doc }` para armazenamento durável. | Carregar doc com schema errado; formato sem versão |
| **D38** | 🟠 | aceita | HTML/Markdown | (a) no core; (b) crates separados pós-MVP; (c) nunca | **(b)** — cada import devolve `(Doc, ImportReport)` com `mapped`, `approximated`, `dropped` e motivo. | Core inflado; “HTML é o modelo”; perda silenciosa |
| **D39** | 🟠 | aceita | Schema mismatch | (a) erro total; (b) ignorar; (c) erro com path + lossy explícito | **(c)** — igual a D14; `LossyPolicy` explícita; nunca default. | Silêncio; crash; doc “meio carregado” |
| **D40** | 🟡 | adiada | Formato binário | (a) só JSON; (b) binário no MVP | **(a)** no MVP; **(b)** pós-MVP atrás de `Codec`. | Escopo duplicado antes do modelo estabilizar |

### D35 em uma frase
O byte canônico é o contrato: o mesmo doc e o mesmo schema produzem exatamente os mesmos bytes. A
ordem de atributos é a do schema; a ordem de marcas é a do schema; arrays preservam ordem; números
são inteiros; strings são UTF-8 cru com escape mínimo. Um `pretty` existe para humanos, mas não é
hashável.

### D36 em uma frase
O writer é pequeno, determinístico e `no_std`. O parser é a parte arriscada: fica atrás de uma
feature, é fuzzado, tem testes diferenciais contra `serde_json` e devolve erros com offset. Se o
parser próprio atrasar, o fallback é `serde_json` na feature, não no core.

---

## F. Rigor, operação e distribuição

| # | Bloqueio | Status | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|---|---|
| **D41** | 🔴 | aceita | `no_std` | (a) `std` obrigatório; (b) `no_std + alloc`; (c) `no_std` sem `alloc` | **(b)** — `#![no_std]`, `extern crate alloc`; feature `std` só para conveniência; CI builda sem default features. | Acoplamento acidental a `std`/I/O; WASM maior; portabilidade perdida |
| **D42** | 🔴 | aceita | Toolchain | (a) só MSRV; (b) pin = MSRV; (c) pin 1.99 + MSRV 1.92 | **(c)** — `rust-toolchain.toml` 1.99; `rust-version`/`clippy.toml` 1.92; `resolver = "3"`; CI com job 1.92. | MSRV fictícia; “na minha máquina compila”; sugestões de API acima da MSRV |
| **D43** | 🔴 | aceita | Lints | (a) default; (b) html-auy; (c) html-auy + knudge (mais duro) | **(c)** — `forbid(unsafe_code)`, deny `unwrap`/`expect`/`panic` **inclusive em testes**, `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, casts, `disallowed_types`/`methods`/`macros`, `allow_attributes_without_reason`, `#[expect]` com motivo. | Panic em input hostil; overflow silencioso; ordem não determinística; exceção zumbi |
| **D44** | 🔴 | aceita | Determinismo | (a) confiar na disciplina; (b) lints + testes | **(b)** — sem `HashMap`/`HashSet`/`Rc`/`RefCell`, sem relógio/env/RNG, ordem canônica, `golden` byte-for-byte, `DIVERGENCES.md`. | Goldens flaky; hashes diferentes; bugs de ordem |
| **D45** | 🟠 | aceita | Modelo de erro | (a) `Box<dyn Error>`; (b) enum fechado + `core::error::Error`; (c) `anyhow` | **(b)** — `EditError`/`SchemaError`/`ParseError`, `Send + Sync + 'static`, contexto com path/posição; sem `unwrap`. | Erro opaco; alocação no erro; erro não `Send` |
| **D46** | 🟠 | aceita | Estratégia de teste | (a) unidade; (b) unidade + proptest + golden; (c) + diferencial + fuzz + Miri + alocação + iai | **(c)** — o diferencial contra um modelo ingênuo é o que pega “plausível mas errado”; a contagem de alocação é o que pega O(n) escondido. | Testes que só repetem a implementação; regressão de performance invisível |
| **D47** | 🟠 | aceita | Gate de arquivo | (a) sem limite; (b) 300 linhas em `src/` | **(b)** — `scripts/check_file_length.sh`; tabelas geradas e testes isentos com justificativa. | Monolitos; revisão difícil; complexidade escondida |
| **D48** | 🟡 | aceita | Licença e versão | (a) MIT; (b) Apache-2.0; (c) MIT OR Apache-2.0 | **(c)** — dual, `0.x`, breaking sobe minor, `CHANGELOG.md`. | Ambiguidade legal; quebra silenciosa |
| **D49** | 🟡 | aceita | Idioma | (a) tudo PT; (b) tudo EN; (c) docs de plano PT, rustdoc/API EN | **(c)** — `plan/`, `AGENTS.md`, `CHANGELOG` em PT; rustdoc, README público, `llms.txt`/`SKILL.md` em EN; identificadores EN. | Docs.rs inconsistente; barreira para consumidor externo |
| **D50** | 🟡 | adiada | Colaboração | (a) CRDT no core; (b) steps prontos, crate separado; (c) nenhum | **(b)** — steps serializáveis/mapeáveis; `rich-editor-auy-collab` pós-MVP; sem IDs de CRDT no doc canônico. | Core inflado; acoplamento a um CRDT específico |

### D43 em uma frase
O rigor não é estético: `unwrap`/`panic` em input hostil é negação de serviço; `HashMap` em
emissão é hash instável; `as` em posição é bug de largura; `indexing_slicing` é panic de borda.
Cada lint deny corresponde a uma classe de defeito que o projeto não pode ter.

### D46 em uma frase
Testes de unidade provam que o código faz o que o autor pensou. O teste diferencial contra um
modelo ingênuo prova que o autor pensou certo. O teste de alocação prova que o caminho comum não
virou O(documento). Os três são necessários.

---

## G. Matriz de bloqueio — ordem mínima de decisão

```text
D01–D07, D09–D12, D16–D19, D53  (modelo, schema, steps, IDs)
        │
        ▼
D13–D15, D20–D25          (fingerprint, unknown, estado, seleção, change set)
        │
        ▼
D27–D33, D35–D37          (histórico, comandos, entrada, wire)
        │
        ▼
D41–D47                   (rigor, erros, testes, gate)
        │
        ▼
D38–D40, D48–D50          (interop, binário, licença, idioma, colaboração)
        │
        ▼
D51–D52, D54–D62          (front-end, zero estilo, refs, PHP, a11y)
```

**Regra:** o alicerce está ratificado (D01–D07, D09–D12, D16–D19, D53). F1 começa depois de F0;
D08, D26, D28, D34, D40 e D50 continuam adiadas e não podem ser antecipadas. As decisões marcadas
`aceita` são base do projeto; as `adiada` são pós-MVP. A escolha determinística de cada estrutura
está em §K.

## H. As doze decisões que mais previnem dor

1. **D01/D02** — modelo de dois níveis + COW: definem o custo de toda edição.
2. **D03/D04** — runs limitados + posição em byte: definem o pior caso e a integração com a UI.
3. **D09/D10** — schema runtime + expressões compiladas: definem o que é documento válido.
4. **D16/D18** — enum de steps + mapping com viés: definem undo, colaboração e seleção.
5. **D19/D23** — transação atômica + normalização local: definem a confiabilidade.
6. **D27** — histórico sem relógio: define undo testável.
7. **D35/D36** — JSON canônico + codec próprio: definem o contrato de bytes.
8. **D41/D42** — `no_std` + MSRV real: definem a portabilidade.
9. **D43/D44** — lints + determinismo: definem o que não pode entrar.
10. **D46** — diferencial + alocação: definem como sabemos que está certo e rápido.
11. **D53** — IDs estáveis: definem referências entre publicações; entrar depois exige migração.
12. **D52/D58/D61** — adaptador zero-CSS + a11y em camadas: definem a integração, a árvore semântica e o que a lib não é.

---

## I. Front-end, zero estilo e referências (extensão)

Estas decisões nascem da exigência de uso em PHP/TS, facilidade de integração, **não interferência
na estilização** e referências entre publicações. Elas não mudam D01–D50; delimitam a superfície.

| # | Bloqueio | Status | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|---|---|
| **D51** | 🔴 | aceita | Contrato de integração PHP/TS | (a) porte do core em PHP; (b) JSON canônico como ABI + WASM para TS + SDK PHP fino; (c) só JSON, sem bindings | **(b)** — o JSON é a fronteira; WASM executa o core no browser; PHP só envelope/bytes/CLI. Nenhum SDK reimplementa schema, steps ou normalização. | Segunda fonte de verdade; drift de schema; PHP lento e inseguro; "funciona no TS, quebra no PHP" |
| **D52** | 🔴 | aceita | Escopo do adaptador DOM | (a) só dados; (b) `data`/`bind`/`mount` com render semântico; (c) UI kit/components | **(b)** — `data` para controle total, `bind` para framework, `mount` para o caminho fácil e acessível. `mount` usa semântica nativa (`<h1>`, `<ul>`, `<a>`) + `A11yTree` + `data-auy-*`; `neutral` é opt-in; zero CSS. | Adaptador virar framework visual; CSS global; wrapper que quebra layout; ARIA-only como default |
| **D53** | 🔴 | aceita | Identidade estável | (a) índice de array; (b) hash de conteúdo; (c) `DocId`/`NodeId` opacos, atribuídos pelo host | **(c)** — IDs opacos, opcionais por nó, únicos no documento; sem RNG/hash no core. Split mantém o original; join tem política explícita. | Referência que quebra ao editar; âncora de DOM; hash que muda a cada tecla |
| **D54** | 🟠 | aceita | Payload de referência | (a) URL string; (b) `RefTarget` tipado (`doc`, `node`, `version`, `fragment`); (c) árvore de nós copiada | **(b)** — `AttrValue::Ref`, marca `ref`/átomo `citation`; único objeto aninhado do wire, chaves fixas. O core valida forma e **não resolve**. | URL frágil; parsing de string ambíguo; core virar CMS; referência sem versão |
| **D55** | 🟠 | aceita | IDs em split/join | (a) duplicar ID; (b) descartar silenciosamente; (c) política explícita + relatório | **(c)** — split: original mantém, novo recebe `Option<NodeId>` do host; join: `KeepFirst` default e `dropped_ids` no relatório; `set_node_id` restaura. | IDs duplicados; perda silenciosa de âncora; referência órfã sem aviso |
| **D56** | 🟠 | aceita | Camada `refs` | (a) no core; (b) crate separado de corpus/índice; (c) CMS completo | **(b)** — `Corpus` em memória + `CorpusStore` trait; `resolve`, `backlinks`, `dangling`, `version_mismatch`; sem rede, sem DB, sem auth. | Core inflado; acoplamento a banco; "referências" virarem produto |
| **D57** | 🟠 | aceita | Render no PHP | (a) renderer HTML em PHP; (b) CLI opcional com política; (c) nenhum SSR | **(b)+(c)** — default é nenhum SSR; quando necessário, CLI `render` com `SanitizePolicy` explícita. Nunca `innerHTML` implícito. | XSS; segunda renderização divergente; PHP reimplementando o core |
| **D58** | 🔴 | aceita | Contrato de a11y | (a) core só semântica bruta; (b) core dono da `A11yTree`/`validate_a11y` + adaptador mapeia + host audita; (c) ARIA/DOM no core | **(b)** — a11y é invariante de core: roles, nomes, `alt`, `lang`/`dir`, ordem de leitura, ranges e validação vivem no core; o adaptador traduz para nativo/ARIA, foco/teclado/IME; o host garante contraste/reflow/WCAG. Sem flag opcional. | A11y pós-MVP; documento sem `alt`/nome; adaptador inventando semântica; ARIA divergente; leitor de tela quebrado |
| **D59** | 🟡 | aceita | Presets de schema | (a) sem presets; (b) presets no core; (c) presets nos bindings, emitindo JSON | **(c)** — `basic`/`document` em TS/PHP emitem schema canônico; o Rust valida. O core não carrega presets de produto. | Core com opinião de produto; schema inválido aceito no cliente |
| **D60** | 🟡 | aceita | Wrappers de framework | (a) mantidos pelo core; (b) comunidade/opcionais; (c) nenhum | **(b)** — React/Vue/Svelte são wrappers finos sobre `@rich-editor-auy/dom`; não entram no workspace publicado do core. | Core virar framework; matriz de versões; manutenção infinita |
| **D61** | 🔴 | aceita | Árvore semântica | (a) adaptador constrói; (b) core expõe `A11yTree`/`A11yRole`; (c) sem árvore | **(b)** — projeção determinística de `Doc`+`Schema`: roles, nomes, `alt`, `lang`/`dir`, níveis, ranges e filhos; sem DOM/ARIA. É derivada, não wire. | Adaptador inventa semântica; a11y não testável no core; documento sem hierarquia/ordem |
| **D62** | 🟠 | aceita | Validação de a11y | (a) sem validação; (b) `validate_a11y` com códigos/severidade; (c) só no adaptador | **(b)** — `A11yCode` estável, `A11yIssue` com `path`/`range`, erros para `alt`/nome obrigatórios e warnings para hierarquia; entra no `make check`. | `alt`/nome vazio passa; perda silenciosa; validação só no browser |

### D52 em uma frase
O adaptador é dono da **estrutura que ele mesmo criou** e dos eventos; o consumidor é dono do CSS.
O host é a raiz, não há wrapper, e `destroy()` restaura tudo. Se o consumidor quer o DOM dele, usa
`data`/`bind`; se quer o caminho fácil, usa `mount` com semântica nativa e `A11yTree`.

### D53 em uma frase
Referência estável não é índice nem hash: é um ID opaco que o host atribui e que o core preserva
através de split, join, undo e mapping. O core não gera IDs porque não tem RNG e porque a identidade
é uma decisão de publicação, não de documento.

### D54 em uma frase
Uma referência é um alvo tipado, não uma URL. O core guarda `doc`/`node`/`version`/`fragment`,
valida a forma e mapeia o range; quem sabe se o alvo existe é o corpus, não o documento.

### D56 em uma frase
O core não é um banco de publicações. Ele entrega o índice de referências de **um** documento; o
corpus, os backlinks e a política de dangling são de um crate separado que não conhece DOM, rede
ou autenticação.

---

## J. Lacunas de escopo (ratificadas como escopo, implementação pós-MVP)

Estas decisões fecham o que o escopo de produto exige e o plano ainda não declarava. Todas mantêm
o core sem I/O, sem layout e sem dependência de runtime/UI; `blake3` é a única dependência normal
externa do core (F30). D63/D65 são implementação pós-MVP; D64 é um crate pós-MVP.

| # | Bloqueio | Status | Decisão | Opções | Recomendação | Previne |
|---|---|---|---|---|---|---|
| **D63** | 🟠 | aceita | Formatos de escritório | (a) parser no core; (b) crates `docx`/`odf` separados; (c) nenhum | **(b)** — `rich-editor-auy-docx`/`-odf` importam/exportam com `ImportReport`/`ExportReport`, preservam o que o schema suporta e declaram perda de estilo; pós-MVP, fora do core. | Core inflado por OOXML/ODF; “Word é o modelo”; perda silenciosa de estilo |
| **D64** | 🔴 | aceita | Integridade e assinatura | (a) FNV do schema como prova; (b) crate `integrity` sobre bytes canônicos; (c) cripto no core | **(b)** — **BLAKE3-256 + Ed25519** no crate separado, sobre o `Envelope` canônico; chaves e política no host. BLAKE3 é o único hash do projeto; FNV/SHA-256 não entram. O fingerprint de schema (D13) é BLAKE3 domain-separated, **não** assinatura. | Confundir fingerprint com assinatura; cripto no core; “assinatura” que não cobre metadados |
| **D65** | 🟠 | aceita | Diff e auditoria | (a) diff textual no CLI; (b) crate `diff` semântico + `ValidationReport`; (c) diff no core | **(b)** — compara `Doc` por `NodeId`/caminho, produz mudanças estruturadas; `ChangeSet` continua por steps; CLI agrega `validate`/`a11y`/`diff`. | Diff por posição que quebra; auditoria como texto solto; core inflado |
| **D66** | 🔴 | aceita | Conteúdo rico | (a) schema/preset + `A11yTree`; (b) tipos de tabela/mídia no core; (c) só consumidor | **(a)** — tabela/célula/caption/spans, imagem/vídeo/áudio/anexo e colunas são schema + preset `document`; o core guarda estrutura/attrs/alt/transcript, não layout, upload ou fetch. | Base que não representa o conteúdo prometido; mídia sem semântica; colunas como geometria no core |
| **D67** | 🟠 | aceita | i18n estrutural | (a) core `lang`/`dir`/`writingMode` + mensagens fora; (b) catálogo de locale no core; (c) tudo no host | **(a)** — o core preserva `lang`/`dir` e a ordem lógica (bidi é UI), valida tags quando possível e emite `A11yCode` + `args`; tradução/UI strings ficam no CLI/adaptador/host. | Texto sem direção; mensagens do core não localizáveis; locale no core |

**Regra:** D63/D65 são crates opcionais pós-MVP; D64 é um crate de integridade separado; D66/D67
definem schema/semântica do core, mas não trazem layout, upload, fetch ou catálogo de mensagens. Se
qualquer uma virar produto/UI dentro do core, o projeto deixa de ser uma base.

---

## K. Estruturas de dados Rust — escolha determinística (ratificada)

> **Regra.** Toda coleção que chega ao wire, ao schema compilado ou a um relatório tem **ordem
> total** definida pela declaração/documento. `BTreeMap`/`BTreeSet` são aceitos para lookup e
> determinização; antes de emitir ou hashear, viram `Box<[T]>` ordenado. `HashMap`/`HashSet`
> continuam proibidos no core (F12). `blake3` é o único hash (F30).

| Estrutura | Representação escolhida | Por que é determinística | Custo/limite |
|---|---|---|---|
| `Doc` / snapshots | `Doc(Arc<DocInner>)`; `Arc::try_unwrap`/`make_mut`; sem mutabilidade interior | snapshots compartilhados são imutáveis; não há `RefCell`/`Cell`/`Rc`; a ordem é a do documento | COW copia ponteiros só quando um snapshot é retido |
| `BlockList` / topo | `Vec<Arc<Block>>` | ordem do vetor = ordem do documento; nenhuma iteração de hash | `ChunkedVec` (D06) só se S4 estourar; API interna preservada |
| `BlockContent`, `InlineItem`, `Step`, `Selection`, `Origin`, `AttrValue`, `A11yArg`, erros | enums fechados, tags fixas, `match` exaustivo | a variante é o discriminante; não há `Box<dyn Trait>` nem downcast | variante nova exige decisão; sem alocação por step quando couber |
| `Run` / texto | `String` mutável por run, ≤ 4 KiB; split em fronteira de escalar UTF-8; merge canônico | a divisão é uma função pura do texto + teto; dois docs iguais produzem os mesmos runs | pior memmove de 4 KiB; ~256 runs num bloco de 1 MiB |
| `Attrs` | `Box<[Attr]>` ordenado por `AttrId`, sem duplicatas; `AttrId` é `u16` na ordem de declaração | a ordem de declaração é parte do schema; lookup por busca binária; defaults são omitidos | O(log n) por attr; n ≤ 32 (D07); sem mapa no wire |
| `MarkSet` | máscara `u64` na ordem de `MarkId` + `Option<Box<[MarkAttr]>>` ordenado por `(MarkId, AttrId)` | a máscara é um bitset com posição fixa; attrs têm ordem total; sem `HashSet` | 8 bytes por run; máximo de 64 marcas |
| Registros do schema | `Box<[NodeSpec]>`/`Box<[MarkSpec]>`/`Box<[AttrSpec]>` densos, indexados por `u16`; nomes congelados em `Box<[NameEntry]>` ordenado | IDs são atribuídos na ordem de declaração; não há `HashMap`; lookup de nome é ordenado | O(1) por ID; O(log n) por nome; builder pode usar `BTreeMap` |
| `ContentExpr` / `ContentMatch` | NFA em `Box<[State]>`; determinização com `BTreeSet<StateId>`; transições em `Box<[Transition]>` ordenadas por `(min, max, next)`; estados numerados em BFS | a closure de estados é ordenada; a escolha do próximo estado não depende de hash; o autofill é o primeiro tipo da ordem da expressão | compilação O(estados × alfabeto); limites de schema rejeitam explosão |
| `StepMap` / `Mapping` | `StepMap { ranges: Box<[StepMapRange]> }` com ranges ordenados por `start`; `Mapping { maps: Box<[StepMap]> }` na ordem dos steps; `assoc: i8` | ranges não se sobrepõem e são percorridos em ordem; composição é associativa e sem float | O(ranges) por map; O(steps) por mapping; aritmética checada |
| `Slice` / `Fragment` | `Fragment(Box<[Arc<Block>]>)`; `open_start`/`open_end` validados contra a árvore | a ordem dos blocos é a do fragmento; profundidades são inteiros pequenos | zero capacidade extra; validação nas bordas |
| `Transaction` | `Vec<Step>` na ordem de emissão | a ordem dos steps é a intenção; `apply` normaliza e falha atomicamente | O(steps); sem ordenação implícita |
| `ChangeSet` | `blocks: Range<u32>` + `Box<[RangeChange]>` ordenado por `(old.start, new.start)` | derivado dos steps, não de diff/hash; ranges têm ordem total | O(ranges afetados); sem varrer o documento |
| `History` | `Vec<Group>` (pilha) com `Vec<Step>` inversos; `GroupId(u64)` monotônico ou do host | pilha não depende de relógio; evicção é por bytes/grupos, não por tempo | teto de 200 grupos/8 MiB (D29); `Vec` é cache-friendly |
| `Limits` | `Copy` com `u32` para bytes, nós, profundidade, attrs e runs | os mesmos limites valem em 32/64-bit e WASM; conversão para `usize` é checada | 64 MiB/doc, 1 MiB/bloco, 64 níveis, 1M nós |
| `DocId`/`NodeId`/`VersionTag` | newtypes sobre `Box<str>` ASCII validado; IDs opacos atribuídos pelo host | o core não gera IDs (sem RNG/hash); `BTreeMap` no `refs`; duplicata no doc é erro | 1..=128 bytes (NodeId/DocId), 0..=64 (VersionTag) |
| `A11yTree` / `A11yReport` | `A11yNode { children: Box<[A11yNode]> }`; argumentos em `A11yArg`; relatório em arrays ordenados por `(path, code)` | a ordem de leitura é a do documento; não há mapa de roles; a árvore é derivada e não wire | O(nós); on-demand, fora do caminho de tecla |
| Hashes | `Fingerprint` no core; `ContentHash`/`TraceHash` no crate `integrity`/pós-MVP; todos `[u8; 32]`; BLAKE3-256 com `derive_key` por domínio; hex minúsculo de 64 caracteres | BLAKE3 é determinístico e portátil; domain separation evita colisão entre schema/conteúdo/trace; sem truncamento | 32 bytes; cálculo sob demanda, nunca por tecla |
| Erros/diagnósticos | enums fechados com `NodePath`/`Box<str>`; `core::error::Error`; `A11yIssue { code, path, args }` | a mensagem não entra no wire nem no hash; a ordem é `(path, code)` | `Send + Sync + 'static`; sem `Box<dyn Error>` |
| Compartilhamento | `Arc` só para `DocInner`, `Block` e `Schema` imutáveis; `Box`/`Vec` para dono único | `Arc` é `Send + Sync`; não há `Rc`/`Weak`/`RefCell`/`Cell`; snapshots não mutam | atomic refcount só onde há compartilhamento real |
| Strings | `Box<str>` para nomes, IDs, attrs e erros imutáveis; `String` só em `Run.text`/builder | `Box<str>` não carrega capacidade sobrando; o wire não depende de `String::capacity` | uma alocação exata; sem `Cow` transitório no modelo |

### K.1 Regras que preservam o determinismo

1. **Ordem explícita.** Toda lista persistida tem uma chave total documentada; `sort` é estável ou
   a chave é única. Nunca ordenar por ponteiro, endereço, `TypeId` ou iteração de hash.
2. **Congelar antes de emitir.** `BTreeMap`/`BTreeSet` são estruturas de trabalho; o wire, o
   fingerprint e os relatórios usam `Box<[T]>` já ordenado. A representação de trabalho não vaza.
3. **BLAKE3 com domínio e comprimento.** `derive_key("rich-editor-auy/<tipo>/v1")` separa schema,
   conteúdo, trace e envelope; o comprimento é implícito no BLAKE3, mas campos variáveis são
   serializados com tamanho/estrutura canônica antes do hash.
4. **Sem relógio, ambiente, RNG ou float.** Tempo, aleatoriedade e configuração entram como dados
   do host (F13); `f32`/`f64` não existem no wire nem no modelo. `as` é proibido; aritmética é
   checada.
5. **IDs não são hashes.** `DocId`/`NodeId` são opacos e atribuídos pelo host; `content_hash` não
   substitui identidade; hash de conteúdo não vira âncora.
6. **Mesma bytes em qualquer alvo.** `u32`/`u16`/`u64` têm largura fixa; o wire não usa `usize`;
   o golden roda em 32-bit, 64-bit e `wasm32` quando o alvo existir.
7. **Teste que trava a borda.** `DIVERGENCES.md` registra cada decisão de ordem/Unicode/hash com o
   teste correspondente; `golden` byte-for-byte e proptest de round-trip são gates.

**Consequência:** o core não tem “ordem que funcionou na minha máquina”. Se uma estrutura nova
precisar de `HashMap`, de relógio ou de float para funcionar, ela está fora do core ou exige uma
nova decisão registrada.
