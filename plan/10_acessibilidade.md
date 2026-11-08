# 10 — Acessibilidade universal (AUY)

> **Contrato.** Acessibilidade **não é feature opcional, pós-MVP ou responsabilidade exclusiva do
> adaptador**. `auy` significa **acessibilidade + universalidade**: acessibilidade universal. Ela é
> uma invariante de projeto e entra em **toda** decisão de modelo, schema, step, posição, wire,
> comando, import/export, API e UI.
>
> O core continua headless: não emite DOM, ARIA renderizada, CSS, foco ou contraste. O que ele
> **possui** é a **árvore semântica acessível**, a validação, a ordem de leitura, os nomes, as
> alternativas textuais, `lang`/`dir`, os ranges e os IDs. O adaptador traduz essa árvore para
> elementos nativos/ARIA; o host audita a experiência final. A camada muda; a responsabilidade não.

---

## 1. Tese

Um documento rico é inacessível quando a informação existe apenas em pixels, cor, posição visual ou
ponteiro. O core não pode aceitar isso como “detalhe de UI”:

- se o schema não exige `alt`, o documento pode nascer sem alternativa textual;
- se o modelo não distingue heading/level/lista/tabela, o adaptador não tem o que mapear;
- se a validação não conhece nome acessível, link vazio passa como válido;
- se o `ChangeSet` não carrega ranges, o anúncio de leitor de tela vira adivinhação;
- se o import descarta `lang`/`dir`/`alt`, a perda é silenciosa.

Portanto, a acessibilidade é **propriedade do documento e da álgebra**, não um acabamento do DOM.

## 2. Vocabulário

| Termo | Significado |
|---|---|
| **Árvore semântica** | projeção determinística do `Doc` + `Schema`: roles, nomes, descrições, `alt`, `lang`, `dir`, níveis, ranges e filhos. |
| **Role semântica** | intenção do nó (`heading`, `list`, `link`, `image`), não string ARIA. O adaptador escolhe `<h1>`, `<ul>`, `<a>` ou `role`. |
| **Nome acessível** | texto que identifica o controle/objeto: conteúdo textual, `alt`, `title` ou atributo declarado. |
| **Ordem de leitura** | ordem do documento canônico; não é ordem visual, CSS `order` ou posição de canvas. |
| **Issue de a11y** | diagnóstico estável com código, severidade, range e correção sugerida. |

## 3. Modelo proposto

```rust
pub enum A11yRole {
    Document,
    Text,
    Paragraph,
    Heading { level: u8 },
    List,
    ListItem,
    Quote,
    Code,
    CodeBlock,
    Table,
    TableCaption,
    TableRow,
    TableCell { header: bool },
    Link,
    Image,
    Figure,
    Caption,
    Video,
    Audio,
    Attachment,
    Separator,
    /// Role declarado no schema; o adaptador mapeia ou falha, nunca adivinha.
    Custom(RoleId),
}

pub struct RoleId(/* índice opaco na tabela de roles do schema */);

pub enum CellScope { Col, Row, ColGroup, RowGroup }

pub struct A11ySpec {
    pub role: A11yRole,
    pub name: NamePolicy,
    pub alt: AltPolicy,
    pub level: Option<AttrId>,
}

pub enum NamePolicy {
    None,
    TextContent,
    Attr(AttrId),
    Required,
}

pub enum AltPolicy {
    NotApplicable,
    Optional,
    Required,
}

pub struct A11yNode {
    pub id: Option<NodeId>,
    pub role: A11yRole,
    pub name: Option<Box<str>>,
    pub description: Option<Box<str>>,
    pub alt: Option<Box<str>>,
    pub lang: Option<Box<str>>,
    pub dir: Option<Dir>,
    pub level: Option<u8>,
    pub text: Option<Box<str>>,        // conteúdo textual da folha
    pub marks: MarkSet,                // strong/em/underline/strike/sub/sup/link
    pub href: Option<Box<str>>,       // link
    pub src: Option<Box<str>>,        // imagem/vídeo/áudio/anexo
    pub caption: Option<Box<str>>,    // figura/tabela
    pub transcript: Option<Box<str>>, // vídeo/áudio
    pub ordered: Option<bool>,         // lista
    pub scope: Option<CellScope>,      // célula de tabela
    pub colspan: Option<u16>,
    pub rowspan: Option<u16>,
    pub range: Range<Pos>,
    pub children: Box<[A11yNode]>,
}

pub struct A11yTree {
    pub root: A11yNode,
    pub lang: Option<Box<str>>,
    pub dir: Option<Dir>,
}

pub enum A11ySeverity { Error, Warning }

pub enum A11yArg {
    Text(Box<str>),
    Number(u32),
    NodeKind(NodeKind),
    AttrId(AttrId),
}

pub struct A11yIssue {
    pub code: A11yCode,       // código estável, publicado por `schema`
    pub severity: A11ySeverity,
    pub path: DocPath,        // caminho do nó/campo
    pub range: Range<Pos>,
    pub args: Box<[A11yArg]>, // dados para o catálogo do CLI/adaptador; sem mensagem no core
}

pub struct A11yReport {
    pub errors: Box<[A11yIssue]>,   // ordenados por (path, code)
    pub warnings: Box<[A11yIssue]>, // ordenados por (path, code)
}
```

API:

```rust
impl Doc {
    /// Projeção pura; não mede, não desenha, não toca no DOM.
    pub fn a11y_tree(&self, schema: &Schema) -> A11yTree;
    pub fn validate_a11y(&self, schema: &Schema) -> A11yReport;
}
```

`A11yTree` é derivada, determinística e **não entra no wire canônico**. Ela não substitui o `Doc`:
é uma view semântica para adaptador, CLI, testes e LLM. O `Schema` é a fonte dos roles/políticas; o
adapter não inventa semântica.

### 3.1 Códigos iniciais

| Código | Severidade | Condição |
|---|---|---|
| `A001` | erro | imagem/vídeo/áudio/anexo com `alt` obrigatório ausente |
| `A002` | erro | link/controle sem nome acessível |
| `A003` | erro | heading sem texto ou nível fora de 1–6 |
| `A004` | warning | salto de nível de heading (ex.: h1 → h3) |
| `A005` | erro | lista sem item / item fora de lista |
| `A006` | warning | tabela sem cabeçalho ou sem `caption` |
| `A007` | erro | `lang` ausente no documento quando o schema exige |
| `A008` | warning | `dir` inválido ou conflitante com `lang` |
| `A009` | erro | `NodeId` duplicado em âncora (já é invariante do modelo) |
| `A010` | warning | texto puramente visual (separador, espaços) sem alternativa |

Os códigos são **dados**: entram em `schema`, no CLI e nos testes de paridade. Não são strings
soltas no adaptador. `Warning` é diagnóstico de relatório, não dispensa de gate: o CLI pode
promovê-lo a erro com `--strict`, e o teste de cada código falha quando o comportamento regride.

## 4. Invariantes

| # | Invariante | Como é garantida |
|---|---|---|
| A1 | Toda árvore semântica preserva a ordem de leitura do documento | projeção única + golden |
| A2 | Todo `A11yNode` tem role semântica válida para o schema | `Schema` validado + `a11y_tree` |
| A3 | Imagem/átomo com `AltPolicy::Required` sem `alt` é erro | `validate_a11y` + schema |
| A4 | Link/controle com `NamePolicy::Required` sem nome é erro | `validate_a11y` |
| A5 | `lang`/`dir` são herdados do documento/bloco e não inventados | modelo + projeção |
| A6 | Heading tem nível 1–6 e texto não vazio | schema + validação |
| A7 | Lista/tabela mantêm estrutura e relações pai/filho | schema + projeção |
| A8 | Ranges da árvore são os ranges lógicos do `Doc` | `Pos`/`Range` + mapping |
| A9 | `A11yTree` é determinística e independente de layout | sem relógio/RNG/hash map |
| A10 | Toda mudança de conteúdo produz `ChangeSet` suficiente para anúncio | ranges antigos/novos |
| A11 | `A11yTree` não substitui o wire; é derivada e não persistida | wire canônico |
| A12 | Nenhuma release sem `validate_a11y` e goldens da árvore | gate de CI |

## 5. Camadas

| Camada | Possui | Não possui |
|---|---|---|
| **Core** | roles semânticas, nomes, `alt`, `lang`, `dir`, ordem de leitura, ranges, IDs, `A11yTree`, `validate_a11y`, `ChangeSet` com ranges | DOM, ARIA renderizada, foco, contraste, CSS, leitor de tela |
| **Adaptador** | mapeamento role → elemento nativo/ARIA, foco, teclado, IME, seleção DOM, live regions, anúncios | semântica inventada, CSS, contraste, auditoria final |
| **Host** | CSS, contraste, zoom/reflow, foco visível, auditoria WCAG, testes com leitores | semântica do documento |

**Regra dura:** o adaptador **não pode** criar um documento semanticamente mais pobre que a
`A11yTree`. Se ele não sabe mapear um role, falha explicitamente; não emite `div` genérico em
silêncio.

## 6. Gate de decisão

Toda decisão — modelo, schema, step, wire, comando, import/export, API, adaptador, CLI — deve
responder a estas perguntas antes de ser aceita:

1. **Role:** que role semântica o item tem (ou por que não tem)?
2. **Nome/alt:** como o usuário não visual o identifica? O schema exige nome/alt?
3. **Ordem:** a ordem de leitura continua correta sem posição visual?
4. **Não visual:** funciona sem cor, ponteiro, hover, animação ou áudio?
5. **Estado:** seleção, foco, composição, erro e mudança são anunciáveis?
6. **Falha:** o que acontece quando `alt`/nome/`lang` falta? Falha fechada ou aviso?
7. **Teste:** qual golden/validação/propriedade prova a árvore, não só o pixel?

Se a resposta for “o adaptador resolve”, a decisão ainda não está pronta: o **dado** precisa existir
no core.

## 7. Testes e CI

- **Golden de árvore**: `doc → a11y_tree` byte/estrutura determinística para corpus fixo,
  incluindo `text`/`marks`/`href`/`src`/`caption`/`transcript`, listas ordenadas e células com
  `scope`/`colspan`/`rowspan`.
- **Validação**: um teste por `A11yCode`, com `path`/`range` e severidade.
- **Propriedade**: todo doc válido gera árvore com a mesma ordem de texto; `validate_a11y` é
  determinístico e não entra em pânico.
- **Import/export**: `ImportReport` conta perda de `alt`, `lang`, `dir`, heading, tabela.
- **Adaptador**: `axe-core`, teclado, foco, IME e leitor de tela real; axe verde não é WCAG.
- **CI**: `make check` inclui os testes de a11y; release sem a suíte verde é bloqueado.
- **CLI**: `validate --a11y` devolve `A11yReport`; `render --a11y-tree` é diagnóstico, não wire.

## 8. Roadmap

| Fase | A11y obrigatória |
|---|---|
| **F1** | `A11yRole`/`A11ySpec` no schema; `A11yNode`/`A11yTree`; `lang`/`dir`/`alt`; primeiros goldens |
| **F2** | steps preservam ranges/IDs da árvore; `ChangeSet` com ranges para anúncio; estrutura de heading/lista preservada |
| **F3** | `Doc::a11y_tree`, `validate_a11y`, comandos e `ImportReport`; tabela/tabela-cabeçalho |
| **F3.5** | adaptador mapeia a árvore; `mount` sempre semântico; `data` assume responsabilidade explícita |
| **F4** | fuzz da árvore, auditoria de leitor de tela, gate de release |
| **F5** | HTML/Markdown preservam a11y e reportam perda; `refs` expõe âncoras |

A11y não é um marco separado: é critério de aceite em todos.

## 9. Limites honestos

- O core **não** garante WCAG sozinho: contraste, foco visível, zoom, reflow e comportamento de
  leitores dependem de CSS, adaptador e host.
- O core **não** emite ARIA: ele emite roles semânticas. ARIA é uma tradução do adaptador, e ARIA
  errada é pior que elemento nativo.
- Canvas não é acessível por si; o host precisa de espelho DOM/live region. O adaptador não finge
  que resolve.
- Texto alternativo automático não é acessibilidade: `alt` obrigatório é validado, mas a qualidade
  é editorial e do host.

## 10. Decisões

- **D58** — a11y é invariante de core; árvore semântica + validação no core; adaptador mapeia;
  host audita.
- **D61** — `A11yTree`/`A11yRole` como API pública derivada, determinística e testada.
- **D62** — `validate_a11y` com códigos estáveis, severidade e `path`/`range`, no `make check`.

Estas decisões são pré-requisito de F1; nenhuma feature entra antes de o schema declarar role,
nome, `alt`, `lang` e `dir` quando aplicável.
