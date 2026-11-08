# 06 — API e integração com a UI

> **Contrato.** O core é headless: ele não desenha, não mede, não lê I/O e não conhece o sistema
> operacional. A UI é um cliente que traduz eventos, desenha e mede. Este documento define o que
> cada lado recebe, o que cada lado deve fazer e o que nenhum dos dois pode assumir. Para integração
> PHP/TypeScript, zero estilo e referências entre publicações, veja
> [`09_frontend-e-referencias.md`](09_frontend-e-referencias.md).

---

## 1. A fronteira em uma tabela

| O core fornece | A UI fornece |
|---|---|
| `Doc` canônico, schema, posições lógicas | pixels, fontes, layout, linhas visuais, scroll |
| `EditorState` imutável, seleção lógica, `storedMarks` | geometria da seleção, caret, highlight |
| `Transaction`, `Step`, `Mapping`, `ChangeSet` | tradução de eventos (tecla, mouse, toque, IME) |
| Comandos embutidos e input rules determinísticas | keymap do SO, menus, toolbar, atalhos globais |
| `Slice` para copiar/colar, JSON canônico | clipboard do SO, drag & drop, sanitização de HTML |
| `A11yTree` determinística: roles, nomes, `alt`, `lang`/`dir`, ordem de leitura, ranges, `validate_a11y` | mapeamento para elementos nativos/ARIA, foco, teclado, IME, live regions, contraste, leitores de tela |
| IDs estáveis e `RefTarget` (preserva, não resolve) | resolução no corpus, backlinks, política de dangling |
| Limites, erros tipados, `ImportReport` | mensagens, fallback, logging, telemetria |
| Snapshots `Arc` para análise em background | orquestração de threads, cache, invalidação |

O core **não** promete ARIA, foco, contraste ou WCAG renderizados; ele **promete** a árvore
semântica, a validação e os ranges que tornam esses gates possíveis. Performance de layout e
fidelidade visual continuam da UI. A acessibilidade não é uma camada opcional: sem a árvore, a UI
não tem o que mapear.

## 2. Superfície pública (esboço)

### 2.1 Estado

```rust
pub struct EditorState { /* doc, selection, stored_marks, schema, history, composition */ }

impl EditorState {
    pub fn new(schema: Arc<Schema>) -> Result<Self, SchemaError>;
    pub fn doc(&self) -> &Doc;
    pub fn selection(&self) -> Selection;
    pub fn stored_marks(&self) -> &MarkSet;
    pub fn schema(&self) -> &Arc<Schema>;

    pub fn transaction(&self) -> Transaction;

    /// Rota imutável: o estado original permanece válido em caso de erro.
    pub fn apply(&self, tx: Transaction) -> Result<Applied, ApplyError>;

    /// Rota owned: move o estado; em caso de erro, devolve o estado restaurado.
    pub fn apply_owned(self, tx: Transaction) -> Result<Applied, ApplyError>;

    pub fn undo(&self) -> Result<Option<Applied>, ApplyError>;
    pub fn redo(&self) -> Result<Option<Applied>, ApplyError>;
}

pub struct Applied {
    pub state: EditorState,
    pub changes: ChangeSet,
}
```

`apply` é a rota de correção (COW, estado original intacto em erro). `apply_owned` é a rota quente
(muta no lugar quando o `Arc` é único; restaura pelos inversos em erro). A UI pode começar com
`apply` e migrar para `apply_owned` depois de medir.

### 2.2 Transação

```rust
pub struct Transaction { /* steps, selection, origin, group, add_to_history */ }

impl Transaction {
    pub fn insert_text(&mut self, at: Pos, text: &str) -> Result<(), StepError>;
    pub fn delete(&mut self, range: Range<Pos>) -> Result<(), StepError>;
    pub fn replace(&mut self, range: Range<Pos>, slice: Slice) -> Result<(), StepError>;
    pub fn split_block(&mut self, at: Pos) -> Result<(), StepError>;
    pub fn join_backward(&mut self, at: Pos) -> Result<(), StepError>;
    pub fn add_mark(&mut self, range: Range<Pos>, mark: Mark) -> Result<(), StepError>;
    pub fn remove_mark(&mut self, range: Range<Pos>, mark: Mark) -> Result<(), StepError>;
    pub fn set_stored_mark(&mut self, mark: Mark);
    pub fn set_block_type(&mut self, at: Pos, kind: NodeKind) -> Result<(), StepError>;
    pub fn set_node_attr(&mut self, at: Pos, attr: AttrId, value: AttrValue) -> Result<(), StepError>;
    pub fn set_doc_attr(&mut self, attr: AttrId, value: AttrValue) -> Result<(), StepError>;
    pub fn set_node_id(&mut self, at: Pos, id: Option<&str>) -> Result<(), StepError>;
    pub fn apply_composition(&mut self, event: CompositionEvent) -> Result<(), StepError>;
    pub fn set_selection(&mut self, selection: Selection);
    pub fn set_origin(&mut self, origin: Origin);
    pub fn set_group(&mut self, group: GroupId);
    pub fn set_add_to_history(&mut self, add: bool);
    pub fn steps(&self) -> &[Step];
}
```

Os métodos de edição são **construtores de steps**: eles validam o que é barato validar e empilham
o step. A validação profunda e a normalização acontecem no `apply`, de forma atômica.

### 2.3 Documento e views

```rust
impl Doc {
    pub fn size(&self) -> Size;
    pub fn attrs(&self) -> &Attrs;
    pub fn blocks(&self) -> BlockIter<'_>;
    pub fn block(&self, index: u32) -> Option<&Block>;
    pub fn block_range(&self, index: u32) -> Option<Range<Pos>>;
    pub fn resolve(&self, pos: Pos) -> Option<ResolvedPos<'_>>;
    pub fn text_into(&self, out: &mut String);
    pub fn anchors(&self) -> AnchorIter<'_>;
    pub fn references(&self) -> ReferenceIter<'_>;
    /// Projeção semântica determinística; não mede nem desenha.
    pub fn a11y_tree(&self, schema: &Schema) -> A11yTree;
    pub fn validate_a11y(&self, schema: &Schema) -> A11yReport;
    pub fn to_json_into(&self, out: &mut impl core::fmt::Write) -> core::fmt::Result;
    pub fn to_json_string(&self) -> Result<String, LimitError>;
}

impl Block {
    pub fn kind(&self) -> NodeKind;
    pub fn id(&self) -> Option<&NodeId>;
    pub fn attrs(&self) -> &Attrs;
    pub fn is_textblock(&self) -> bool;
    pub fn is_atom(&self) -> bool;
    pub fn as_textblock(&self) -> Option<&TextBlock>;
    pub fn children(&self) -> Option<BlockIter<'_>>;
    pub fn inline(&self) -> Option<InlineIter<'_>>;
}

impl TextBlock {
    pub fn items(&self) -> &[InlineItem];
    pub fn len(&self) -> Size;
    pub fn is_empty(&self) -> bool;
    pub fn run_at(&self, offset_in_block: Size) -> Option<(usize, Size)>;
}
```

`InlineItem` tem uma **view emprestada** para a UI e uma forma owned para o modelo; a UI nunca vê
o layout interno de `Run` como algo mutável.

```rust
pub enum InlineView<'a> {
    Text { range: Range<Pos>, text: &'a str, marks: &'a MarkSet },
    Atom { range: Range<Pos>, atom: &'a InlineAtom },
}

pub struct Anchor {
    pub id: NodeId,
    pub range: Range<Pos>,
}

pub struct Reference {
    pub source: Range<Pos>,
    pub target: RefTarget,
}
```

`InlineIter` devolve `InlineView` com o range absoluto de cada item. A UI desenha runs e átomos;
não precisa reconstruir a árvore de marcas.

### 2.4 Seleção e posições

```rust
pub enum Selection {
    None,
    Caret { pos: Pos, assoc: Assoc },
    Text { anchor: Pos, head: Pos, assoc: Assoc },
    Node { pos: Pos },
}

impl Selection {
    pub fn anchor(&self) -> Option<Pos>;
    pub fn head(&self) -> Option<Pos>;
    pub fn range(&self) -> Option<Range<Pos>>;
    pub fn is_empty(&self) -> bool;
    pub fn map(&self, mapping: &Mapping) -> Selection;
}

pub fn pos_to_utf16(doc: &Doc, pos: Pos) -> Option<u32>;
pub fn utf16_to_pos(doc: &Doc, block: u32, utf16: u32) -> Option<Pos>;
pub fn snap_to_grapheme(doc: &Doc, pos: Pos) -> Pos;
```

- `Pos` é byte UTF-8 + token estrutural (`D04`). A conversão para UTF-16 é explícita e O(texto do
  bloco); a UI pode cachear por bloco.
- `assoc` (`-1`/`+1`) resolve o caso de borda: uma posição no ponto de inserção pode ficar antes ou
  depois do conteúdo novo.
- `snap_to_grapheme` é a única forma de garantir que o caret não cai no meio de um emoji/combining
  mark. O core garante fronteira de escalar; a UI que quer grafema usa esta função.

### 2.5 ChangeSet

```rust
pub struct ChangeSet {
    /// Faixa de blocos do topo potencialmente afetada.
    pub blocks: core::ops::Range<u32>,
    /// Ranges antigos/novos ordenados por `(old.start, new.start)`.
    pub ranges: Box<[RangeChange]>,
    /// A seleção mudou.
    pub selection: bool,
    /// Mudança estrutural que a UI pode preferir re-renderizar inteira.
    pub full: bool,
}

pub struct RangeChange {
    pub old: Range<Pos>,
    pub new: Range<Pos>,
}
```

A UI usa `blocks` para invalidar o cache de layout e `ranges` para repintar só os trechos. `full`
é um aviso: uma mudança estrutural (ex.: lift de lista) pode invalidar mais do que o range sugere.
O core não promete um diff mínimo por pixel; promete um diff **correto e barato** por posição.

### 2.6 Desfazer/refazer

```rust
impl EditorState {
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn undo(&self) -> Result<Option<Applied>, ApplyError>;
    pub fn redo(&self) -> Result<Option<Applied>, ApplyError>;
}
```

A UI deve desabilitar o botão quando `can_undo` é falso e tratar `Ok(None)` como "nada a fazer".
`History` não é serializado por padrão; undo é uma propriedade da sessão, não do documento.

## 3. Entrada: teclado, texto, IME, colagem

### 3.1 Teclado

```rust
pub struct Key {
    pub code: KeyCode,
    pub mods: Mods,
}

pub enum KeyCode {
    Char(char),
    Enter, Tab, Backspace, Delete,
    Left, Right, Up, Down, Home, End, PageUp, PageDown,
    Escape,
}

pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}
```

O core não sabe o que é `Cmd` no macOS ou `Ctrl` no Linux; a UI resolve `Mod` para o modificador
da plataforma e constrói `Key`. O `Keymap` do core é uma tabela `Key -> CommandId`; a UI pode
sobrescrever, ignorar ou adicionar. Nenhum atalho global é registrado pelo core.

### 3.2 Texto e composição

```rust
pub enum CompositionEvent {
    Start { at: Range<Pos> },
    Update { text: String },
    Commit,
    Cancel,
}

pub enum InputEvent {
    InsertText { at: Pos, text: String },
    Delete { range: Range<Pos> },
    Composition(CompositionEvent),
    Paste { at: Pos, slice: Slice },
    Drop { at: Pos, slice: Slice },
}
```

Fluxo de IME:

1. UI recebe `compositionstart` e chama `Start { at }` (range do texto que será substituído).
2. UI recebe `compositionupdate` e chama `Update { text }`; o core substitui o range e marca a
   transação com `Origin::Composition(id)`.
3. UI recebe `compositionend` e chama `Commit`; o core consolida o grupo de histórico como **uma**
   entrada de undo.
4. `Cancel` restaura o texto original e descarta o grupo.

Durante a composição, a UI não deve aplicar marcas nem mover a seleção por conta própria; o core
mantém a seleção no fim do texto composto e o histórico não grava estados intermediários.

### 3.3 Colagem

A UI lê o clipboard e escolhe o melhor formato:

1. `application/x-rich-editor-auy+json` (se o produtor for nosso) → `Slice::from_json`.
2. `text/html` → um adaptador de formato decide; o core recebe um `Slice` e valida.
3. `text/plain` → `Slice::plain_text`.

O core valida o slice contra o schema e devolve `ApplyError` se não couber; a UI decide cair para
texto puro ou mostrar um aviso. O `ImportReport` do adaptador lista o que foi descartado.

### 3.4 Drag and drop

Mesmo contrato da colagem: a UI extrai um `Slice` do range arrastado (`Doc::slice(range)`), valida
no drop e aplica como `replace`. O core não conhece `DataTransfer`.

## 4. Geometria, layout e seleção visual

O core não sabe onde um caractere está na tela. A UI:

1. itera `Doc::blocks()` e `Block::inline()` para obter runs/átomos com ranges;
2. mede cada run com a fonte/layout dela;
3. mantém um mapa `Pos -> (x, y, height)` por bloco/linha visual;
4. pinta a seleção usando `Selection::range()` e o mapa;
5. traduz clique/arrasto em `Pos` usando o mapa inverso e `snap_to_grapheme`;
6. traduz `Up/Down` em movimento **visual** (a UI conhece as linhas); o core só oferece movimento
   lógico (`block_before`, `block_after`, `next_grapheme`, `prev_grapheme`).

O core oferece `ResolvedPos` e `run_at` para a UI não precisar reimplementar a resolução de
posição. Não oferece `pos_to_xy` nem `xy_to_pos`.

## 5. Threads e snapshots

- O estado é da thread de UI (ou do ator que o possui). O core não usa locks.
- Para análise em background (spellcheck, highlight, busca), a UI clona `Doc` (barato, `Arc`) e
  envia para a thread. O resultado é calculado contra aquele snapshot.
- Ao voltar, a UI compara o snapshot com o doc atual: se for o mesmo `Arc` (`Doc::ptr_eq`), aplica;
  se mudou, mapeia os ranges pelo `Mapping` das transações ou descarta e recalcula.
- `Schema` é `Arc` e imutável; pode ser compartilhado.
- `Transaction` é `Send`; um worker pode construir uma transação, mas só o dono do estado aplica.

## 6. Persistência

```rust
// A UI/persistência faz I/O; o core só transforma valores.
let bytes = doc.to_json_string()?;          // nó canônico, interop
let envelope = Envelope::new(&schema, &doc);
let bytes = envelope.to_json_string()?;     // armazenamento durável
let envelope = Envelope::from_json(&bytes)?;
let doc = envelope.into_doc(&schema)?;      // falha com SchemaMismatch
```

- `Doc::to_json` é compatível com o formato ProseMirror-like (type/content/marks/attrs).
- `Envelope` adiciona `format`, `version`, `schema_fingerprint` e, opcionalmente, `doc_id`;
  `schema_fingerprint` é BLAKE3-256 (64 hex) e `doc_id` é metadado de publicação que não entra no
  hash do `Doc`. É o formato recomendado para armazenamento.
- O core não escreve arquivo, não faz rede, não usa `std::fs`.
- O consumidor decide compressão, criptografia, versionamento e migração.

## 7. Erros

```rust
pub enum ApplyError {
    Step(StepError),
    Schema(SchemaError),
    Selection(SelectionError),
    Limit(LimitError),
    Invariant(InvariantError),
}
```

- `ApplyError` é `Send + Sync + 'static`, implementa `core::error::Error` e tem `kind()` estável
  para a UI distinguir "erro de programa" de "erro de input".
- A UI nunca recebe panic. Se um invariante falhar, é um bug do core; o erro carrega caminho e
  posição para o relatório.
- Limites são erros, não truncamentos silenciosos.

## 8. Acessibilidade: o que o core entrega e o que a UI deve fazer

O core entrega uma **árvore semântica**, não ARIA renderizada:

- `A11yTree` determinística com role, nome, descrição, `alt`, `lang`/`dir`, nível, range e filhos;
- roles de bloco/marca/átomo (heading com `level`, lista, citação, código, tabela, link, imagem);
- nomes e `alt` exigidos pelo schema; `A11yReport` com códigos estáveis e ranges;
- ordem de leitura derivada do documento, não do layout;
- texto puro e ranges de seleção/mudança para anúncio;
- `validate_a11y` para falhar fechado quando `alt`/nome/`lang` obrigatório falta.

A UI/adaptador deve:

- mapear `A11yTree` para elementos nativos primeiro e ARIA só quando necessário;
- não inventar role, nome ou hierarquia que a árvore não declara; se não souber mapear, falhar;
- gerenciar foco, ordem de tabulação, skip links, teclado, IME e live regions;
- garantir contraste, alvo de toque, reflow e foco visível (o core não mede CSS);
- anunciar mudanças de seleção/conteúdo usando os ranges do `ChangeSet`.

**A11y não é opcional:** `mount` é sempre semântico e não existe flag para desligar a árvore. `data`
só transfere a responsabilidade de renderização de forma explícita. **O core não promete WCAG
sozinho** — contraste/reflow/leitor dependem do host —, mas nenhuma release passa sem
`a11y_tree`/`validate_a11y`; o gate final é do consumidor, o gate semântico é do core. O adaptador
DOM (ver [`09_frontend-e-referencias.md`](09_frontend-e-referencias.md)) implementa o mapeamento,
teclado e anúncios; ele não define CSS, contraste ou foco visível.

## 9. Exemplo de loop de UI (pseudo-código)

```rust
fn on_event(state: &EditorState, ev: UiEvent) -> Result<Option<Applied>, ApplyError> {
    let mut tx = state.transaction();

    match ev {
        UiEvent::Char(c) => {
            let Some(at) = state.selection().head() else {
                return Ok(None);
            };
            tx.insert_text(at, &c.to_string())?;
        }
        UiEvent::Backspace => {
            let range = state.selection().range()
                .unwrap_or_else(|| prev_grapheme_range(state, state.selection()));
            tx.delete(range)?;
        }
        UiEvent::Key(Key { code: KeyCode::Enter, .. }) => {
            let Some(at) = state.selection().head() else {
                return Ok(None);
            };
            tx.split_block(at)?;
        }
        UiEvent::BoldToggle => {
            let range = state.selection().range();
            match range {
                Some(r) => tx.add_mark(r, Mark::strong())?,
                None => tx.set_stored_mark(Mark::strong()),
            }
        }
        UiEvent::Composition(e) => tx.apply_composition(e)?,
        UiEvent::Paste { at, slice } => tx.replace(at..at, slice)?,
        UiEvent::Noop => return Ok(None),
    }

    let applied = state.apply(tx)?;
    Ok(Some(applied))
}
```

O loop é deliberadamente simples: a UI traduz evento → comando/transação; o core faz o resto. A UI
não muta o doc, não mexe na seleção por dentro e não conhece `Step` se não precisar.

## 10. Checklist de integração

- [ ] Construir o schema (macro ou builder) e tratar erro de schema.
- [ ] Criar `EditorState` e renderizar `blocks`/`inline`.
- [ ] Gerar `A11yTree` e mapear roles/nomes/`alt`/`lang`/`dir` sem inventar semântica.
- [ ] Rodar `validate_a11y` no CI e tratar `A11yReport` como gate, não como aviso.
- [ ] Mapear teclado para `Key`/comandos; usar `Mod` da plataforma.
- [ ] Aplicar transações com `apply` (ou `apply_owned` após medir).
- [ ] Usar `ChangeSet` para invalidar layout/pintura.
- [ ] Converter `Pos` ↔ unidade da UI (UTF-16 no navegador) e usar `snap_to_grapheme`.
- [ ] Implementar IME com `CompositionEvent` e não gravar estados intermediários no undo.
- [ ] Implementar clipboard/drag com `Slice` e fallback para texto puro.
- [ ] Persistir `Envelope` (não o `Doc` interno) e tratar `SchemaMismatch`.
- [ ] Snapshot para análise em background com `Arc` e mapeamento/descarte.
- [ ] Tratar `ApplyError`/`LimitError` sem panic e sem truncamento silencioso.
- [ ] Mapear `A11yTree` para elementos nativos/ARIA; foco, contraste e leitores são auditoria do host.
- [ ] Não guardar `Pos`/`Selection` entre transações sem mapear.
- [ ] Nenhum CSS/`class`/wrapper da lib; o host é seu e `destroy()` restaura.
- [ ] Atribuir `NodeId` ao que é referenciável; resolver `RefTarget` no corpus e definir política
      de dangling.
- [ ] Se usar canvas, fornecer espelho DOM/live region; canvas sozinho não é acessível e o core não
      finge que é.

## 11. O que a UI não pode assumir

1. **Runs são estáveis.** Uma edição pode dividir/fundir runs; use ranges/ChangeSet, não índices.
2. **Blocos são estáveis.** Índices mudam com split/join; use `ResolvedPos`/`NodePath`.
3. **Um parágrafo é um run.** O teto de 4 KiB pode produzir vários runs com a mesma marca.
4. **`Pos` é UTF-16.** É byte UTF-8; converta explicitamente.
5. **Seleção sobrevive à edição.** Mapeie com `Selection::map` ou deixe o core mapear.
6. **Ordem de iteração é livre.** A ordem é a do documento; não dependa de hash.
7. **`Debug` é serialização.** `Debug` é diagnóstico; o wire é `to_json`.
8. **Que o core resolve layout.** Ele não sabe o que é uma linha visual.
9. **Que o core garante ARIA, foco e contraste renderizados.** Ele garante a `A11yTree`/validação;
   a UI/adaptador mapeia para ARIA, foco, teclado e live regions; o host audita contraste/reflow/WCAG.
10. **Que um erro de step é recuperável sem perder o estado.** Use `apply` (imutável) para steps não
    confiáveis; `apply_owned` restaura, mas a UI deve tratar o erro como falha da operação.
11. **Que o core resolve referências.** Ele preserva `RefTarget` e IDs; quem resolve é o corpus/`refs`.
12. **Que a lib dita estilo.** Ela não injeta CSS/`class`/wrapper; o host e o CSS são do consumidor.
    "Não interferir" não impede o CSS global do consumidor de alcançar o host.
