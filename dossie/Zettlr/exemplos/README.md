# Exemplos executáveis — Zettlr 4.9.0

Estes exemplos chamam **código real do Zettlr** (`ref/Zettlr/source/...`) sem
iniciar o Electron e **sem copiar lógica para dentro do exemplo** (exceto onde
declarado). Licença do código executado: GPL-3.0 (Hendrik Erz); uso apenas para
análise.

## Como funciona

`tsconfig.json` mapeia os mesmos aliases do projeto original
(`ref/Zettlr/tsconfig.json:20-25`) para o `ref/`:

```json
"paths": {
  "@common/*": ["../../../ref/Zettlr/source/common/*"],
  "@providers/*": ["../../../ref/Zettlr/source/app/service-providers/*"],
  "@dts/*": ["../../../ref/Zettlr/source/types/*"]
}
```

O [Bun](https://bun.sh) 1.4.2 resolve esses aliases e executa TypeScript
diretamente. **Não há `node_modules` em `ref/Zettlr/`** (0 pacotes). Na primeira
execução, o Bun resolve as dependências (`@lezer/markdown`, `@codemirror/*`,
`underscore`, `katex`, ...) a partir do cache global de pacotes
(`~/.bun/install/cache`), o que leva ~28 s; nada é escrito dentro de `ref/`
(verificado: `ls ref/Zettlr/node_modules` → não existe).

## Comandos e saída observada

```bash
cd dossie/Zettlr/exemplos
bun run link-index.ts
```

Saída (real, 2026-11-07):

```
ID extraído do nome: "20240101120000"
ID de arquivo sem ID: ""
inbound(Metodo): [ "/notes/20240102090000 Revisao.md", "/notes/Sem ID.md" ]
outbound(Revisao): [ "/notes/20240101120000 Metodo.md" ]
outbound(Inexistente) por link quebrado: []
findExact("Metodo") [nome parcial]: null
findExact("20240101120000 Metodo"): /notes/20240101120000 Metodo.md
findExact("Metodo.md") [com extensão]: null
findExact("Revisao") [basename duplicado]: /notes/arquivo/Revisao.md
findExact("Inexistente"): null
```

O que `link-index.ts` usa do original:

- `extractFileId` de `ref/Zettlr/source/app/service-providers/fsal/util/extract-file-id.ts`;
- `extractFromFileDescriptors` de `ref/Zettlr/source/common/util/extract-from-file-descriptors.ts`;
- a lógica de `findExact`, `retrieveInbound` e `retrieveOutbound` foi
  **transcrita** (cabeçalho do arquivo diz isso) porque a classe `LinkProvider`
  importa `electron` e registra `ipcMain.handle`, o que não roda fora do app.

Interpretação dos resultados:

- `findExact("Metodo")` → `null`: não existe casamento *parcial* nem por título.
  O alvo precisa ser o basename exato (com ou sem extensão) ou o ID.
- `findExact("Metodo.md")` → `null` porque o arquivo se chama
  `20240101120000 Metodo.md`.
- Link para `"Inexistente"` desaparece do outbound: **não há diagnóstico**.
- Com dois arquivos de mesmo basename em pastas diferentes, o **primeiro**
  descriptor vence — sem aviso de ambiguidade.

```bash
bun run replace-links.ts
```

Saída:

```
Antes: [[Metodo (Luhmann)]], [[Metodo (Luhmann).md]] e [[Metodo (Luhmann)|título livre]].
Isto NÃO deve mudar: [texto]([[Metodo]]) e `[[Metodo]]` em código inline.

--- casos que o util NÃO cobre ---
link dentro de markdown: [x]([[Metodo]])
link com título: [[Novo|T]]
link com ID: [[20240101120000]]
```

Constatações:

- reescreve as três formas de `[[...]]` e preserva o título;
- **não** reescreve `[texto]([[alvo]])` nem ocorrências em code span;
- link por ID não é alterado (correto: o ID não muda quando o nome muda).

```bash
bun run md2html.ts
```

Saída (reproduzida literalmente, inclusive os espaços extras):

```html
<h1> Título</h1>

<p>Ver  [[o método]] e  <span class="citation"><span class="citation">doe2020 (composto)</span></span>, p. 12.</p>

<p>Uma nota de rodapé<sup><a class="footnote" href="#fnref:1" name="fn:1">1</a></sup> e  <code>código</code>.</p>
<hr>
<div id="footnote-container">


<div class="footnote-ref"><sup class="footnote-ref-label"><a href="#fn:1" name="fnref:1">1</a></sup>  <p> Texto da nota.</p></div>
</div>
```

Constatações:

- o wikilink vira **texto literal** `[[o método]]`, não âncora
  (comparar com `markdown-to-html.ts:365-371`);
- o callback de citação é envolvido por um `<span class="citation">` do próprio
  renderizador — o callback deve devolver *apenas* o conteúdo interno (o app
  real devolve HTML do citeproc);
- há artefatos de espaço (`<h1> Título</h1>`, `Ver  [[`, `<p> Texto da nota.</p>`)
  e linhas em branco entre `<ul>`/`<li>`; o `md2html` é usado para impressão e
  como base de export, então esse viés precisa ser considerado por quem for
  escrever um serializer.

```bash
bun run link-edges.ts
```

Saída (real, 2026-11-07):

```
"[[alvo|título]]" -> target = "alvo" | title = "título"
"[[alvo|]]" -> target = "alvo" | title = ""
"[[|título]]" -> target = "|título" | title = undefined
"[[Metodo#Intro]]" -> target = "Metodo#Intro" | title = undefined
"[[Metodo#Intro|texto]]" -> target = "Metodo#Intro" | title = "texto"
```

Constatações (contrariam o comentário de `zkn-link-parser.ts:68-70`, que diz
proibir pipe nas pontas):

- `[[alvo|]]` gera um `ZknLinkTitle` **vazio**, não um link sem título. Como
  `render-links.ts:44-49` esconde `ZknLinkContent` + `ZknLinkPipe` quando
  conteúdo, título e pipe existem, o link fica **invisível** no editor.
- `[[|título]]` não é dividido: o pipe vira parte do alvo (`"|título"`), que não
  resolve em `findExact` → link quebrado silencioso.
- `[[arquivo#Heading]]` mantém a âncora no alvo. O clique funciona porque
  `force-open.ts:52` corta no `#`, mas `retrieveOutbound`/`retrieveInbound`
  comparam a string inteira e **não** contam esse link.

## O que NÃO foi executado

| Item | Motivo |
|---|---|
| `yarn test` (`mocha`) | `node_modules` ausente; instalar o monorepo inteiro está fora do escopo do dossiê. |
| App Electron (`yarn start`) | Exige build webpack + Electron 44; não foi iniciado. |
| Export Pandoc | Exige binário Pandoc e o app em execução. |
| Testes `test/markdown-to-ast.spec.ts`, `test/replace-links.spec.ts` | Lidos como evidência de intenção; não executados (mesmo motivo do `mocha`). |

## Arquivos

| Arquivo | O que faz |
|---|---|
| `link-index.ts` | Índice de links e resolução de alvos, usando os módulos reais do FSAL. |
| `replace-links.ts` | Reescrita de links na renomeação, usando o util real. |
| `md2html.ts` | Serialização Markdown → HTML com citação, rodando o renderer real. |
| `link-edges.ts` | Campos `target`/`title` de `[[alvo\|]]`, `[[\|título]]` e `[[a#b]]` no parser real. |
| `tsconfig.json` | Aliases que apontam para `ref/Zettlr/source`. |
