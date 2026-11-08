# Avaliação — Zettlr 4.9.0

Escala 0–5. Toda nota aponta evidência (arquivo:linha, teste ou execução
registrada em `exemplos/`). "Não verificado" significa que o app não foi
executado naquele aspecto.

> Nota metodológica: `node_modules` não existe em `ref/Zettlr/` (0 pacotes).
> O app Electron **não foi iniciado**. Rodei apenas módulos puros via Bun
> (ver `exemplos/README.md`), o que cobre ID, índice de links, `replaceLinks` e
> `md2html`. Tudo o resto é leitura de código.

---

## 1. Tabela de notas

| Dimensão | Nota | Evidência | Comentário |
|---|---:|---|---|
| **Usabilidade** | 4 | `win-print/App.vue:66-110`; `autocomplete/files.ts:63-119`; `win-main/sidebar/RelatedFilesTab.vue:147-196`; `keymaps/default.ts:264-302` | Escrita acadêmica bem resolvida: semi-WYSIWYG por decoração, autocomplete de links/citações, atalhos familiares, preview de impressão igual ao documento. Perde por não sinalizar link quebrado, por obrigar Ctrl/Cmd para abrir link e pelos casos de borda de `[[a|]]`/`[[a#b]]`. |
| **API** | 1 | `links/index.ts:46-59`; `documents/index.ts:270-370`; `package.json:16` (`main: .webpack/main/index.cjs`) | Não há API pública. "API" = canais IPC internos sem versionamento, sem tipos exportados e sem build de biblioteca. Inutilizável como dependência. |
| **Neutralidade de estilo** | 2 | `theme/editor.ts:25-210`; `register-themes.ts:60-127`; `editor.css` (313 linhas); `tagClasses()` em `editor-extension-sets.ts:340` | O editor é acoplado a uma paleta própria via ~30 variáveis `--zettlr-editor-*` e a uma folha de app com classes `.dark`. O markdown renderizado herda o CSS do app; não existe "headless sem CSS". Presets são só overrides de variáveis — bom padrão, mas o produto assume que ele é o dono da aparência. |
| **Modelo de dados** | 4 | `types/common/fsal.ts:84-98`; `fsal/util/file-parser.ts:57-158`; `fsal-directory.ts:27-49` | Arquivo `.md` como fonte de verdade, metadados derivados e serializáveis (JSON puro), projeto = lista ordenada. Simples e auditável. Perde por não ter identidade independente do nome (ID opcional) e por `frontmatter: any`. |
| **Extensibilidade** | 2 | `parser/markdown-parser.ts:257-315`; `editor-extension-sets.ts:152-352`; `util/custom-tags.ts:22-50` | Adicionar nó/mark exige tocar 3 arquivos sincronizados manualmente (comentário em `markdown-parser.ts:256-257`) e registrar renderer/linter. Não há API de plugin de terceiros; customização do usuário é CSS, filtros Lua e perfis Pandoc. |
| **Performance** | 3 | `fsal-cache.ts:9-18`; `fsal-file.ts:94-100`; `links/index.ts:63-71`; `fsal/index.ts:418-444` | Cache de metadados sério e limite explícito de 10 MB. Mas o índice de links é reconstruído por inteiro a cada evento do FSAL e a resolução é O(n) por link; sem índice invertido. Adequado a vaults médios, não a "10k notas" sem custo. |
| **Acessibilidade** | 3 | `win-main/MainEditor.vue:3-7`; `win-main/file-manager/FileTree.vue:5`; `common/vue/TabBar.vue:7`; `grep aria-live` = 0 | Regiões nomeadas e `aria-labelledby` nas janelas; conteúdo editável é CodeMirror 6, que já traz `aria-live`/descrições internas. Falta anúncio próprio de mudanças de documento/links, e não há testes de a11y. **Não verificado com leitor de tela.** |
| **Documentação** | 3 | `README.md:229-267` (arquitetura), `:270-380` (comandos); sem `docs/` no repo; documentação de usuário é externa (`docs.zettlr.com`) | O README é acima da média para um app Electron (arquitetura, comandos, flags). Internamente, os cabeçalhos `BEGIN HEADER` são consistentes em quase todo arquivo — excelente para leitura. Não há guia de contribuição de arquitetura além do README. |
| **Manutenção** | 4 | 4.9.0; 514 arquivos TS/Vue; 43 spec files ativos (47 com `.ignore`) em `test/`; `CHANGELOG.md`; `yarn lint` + `vue-tsc`; dependências atualizadas (Electron 44, CM6 6.43) | Projeto maduro e ativo, com testes de unidade reais (`markdown-to-ast.spec.ts`, `replace-links.spec.ts`) e lint tipado. Sinais de dívida: lógica duplicada (`findExact` em dois lugares), TODOs de ID duplicado/otimização de índice. |
| **Adequação ao nosso caso** | 3 | `arquitetura.md` §2–4; `links/index.ts:131-178` | Como **modelo conceitual** de referências: alto valor. Como código: zero reuso (GPL). Como editor headless: inadequado (Electron, CM6 no renderer, IPC). Serve para copiar o desenho de índice e a semântica de ID; não serve como componente. |

**Média simples:** 2,9/5.

---

## 2. Matriz de cobertura de usabilidade

Legenda: `C` código, `T` teste do repositório, `E` exemplo executado por mim,
`D` documentação, `—` não verificado.

| Item | Status | Evidência |
|---|---|---|
| Instalação e pré-requisitos | `D` | `README.md:87-115` (instaladores, pacotes, Pandoc empacotado: `config/get-config-template.ts:362`) |
| Inicialização mínima | `C` | `main.ts` → `lifecycle` → `app-service-container` descrito em `README.md:238-256`; providers fazem `boot()` (`links/index.ts:61-75`). Não iniciado. |
| Edição básica (texto, parágrafo, negrito/itálico/link) | `C`, `E` | `keymaps/default.ts:264-266`; `markdownToAST`/`md2html` executados em `exemplos/md2html.ts` |
| Seleção, teclado, undo/redo, clipboard | `C` | `keymaps/default.ts:274-277` (undo/redo), `:238-239` (colar limpo/copiar HTML), `editor-extension-sets.ts:186` (múltiplas seleções) |
| IME | `—` | Nenhum código próprio; depende do CodeMirror 6. Não exercitado. |
| Schema e restrições de conteúdo | `C` | `parser/markdown-parser.ts:257-315` (`defineNodes`), `zkn-link-parser.ts:70` (guarda de pipe — só a ponta esquerda funciona), `file-parser.ts:39-49` (allowlist de frontmatter) |
| Custom node/mark/plugin | `C`, `T` | Nós ZknLink/Citation/Admonition (`markdown-parser.ts:257-315`); teste de AST `test/markdown-to-ast.spec.ts` |
| Persistência e serialização | `C`, `E` | `documents/index.ts:1527-1600` (BOM/linefeed em `:1582`), `.ztr-directory` JSON (`fsal-directory.ts:60-77`), `md2html` executado (`exemplos/md2html.ts`) |
| Renderização sem interferir no CSS | `C` | `render-links.ts:18-86`, `base-renderer.ts`, `theme/editor.ts` — decorações e variáveis; não há modo "sem estilo". |
| Acessibilidade | `C` | `MainEditor.vue:3-7`, `FileTree.vue:5`, `TabBar.vue:7`, `aria-labelledby` em `win-about/App.vue:19`. Sem teste. |
| Colaboração | `C` | Só renderer↔main: `remote-doc.ts:54-164` + `@codemirror/collab`; nenhum servidor/rede (`grep websocket` sem provider). |
| Performance e limites | `C` | `fsal-file.ts:94-100` (10 MB), `fsal-cache.ts:9-18`, `links/index.ts:63-71` |
| Erros, migração e diagnóstico | `C`, `E` | `render-citations.ts:72-75` (citação com erro), `documents/index.ts:711-718` (documento fora de sincronia), link quebrado sem diagnóstico (`links/index.ts:160-178`, confirmado em `exemplos/link-index.ts`), frontmatter inválido ignorado (`file-parser.ts:154-156`) |
| Licença e maturidade | `D` | `LICENSE:1` (GPL-3.0), `package.json:13-14`, `CHANGELOG.md` |

**Cobertura:** 13/13 itens tocados; 1 explicitamente `—` (IME) e 2 parciais
(acessibilidade sem leitor de tela; colaboração apenas interna). Considerando
"evidenciado" = `C`/`T`/`E`/`D` com caminho, a matriz está em **~92%**
(12/13 itens com evidência de código/doc, sendo 3 de 12 parciais). Não reivindico
100% porque nada foi observado rodando o app.

---

## 3. Lições de UX de referências (o que o usuário sente)

1. **Digitar `[[` já é o gesto de linkar.** O autocomplete se ativa quando
   `linkStartBefore` é verdadeiro (`autocomplete/files.ts:108-119`), e a
   completion insere alvo + título + `]]` conforme configuração
   (`:63-100`). Copiar isso é barato e melhora muito a descoberta.
2. **Título separado do alvo é a decisão mais importante.** `[[alvo|título]]`
   deixa o texto legível e o alvo estável; o renderer esconde o alvo e o pipe,
   mostrando só o título quando fora do cursor (`render-links.ts:40-51`).
   Nosso editor deve tratar isso como cidadão de primeira classe — inclusive
   porque o Zettlr não trata `[[alvo|]]` (título vazio): o link some da tela.
3. **Links quebrados são invisíveis — e isso é ruim.** No Zettlr, um link
   quebrado parece um link normal e o clique não faz nada
   (`force-open.ts:50-61`). A única rede de segurança é a completion com
   `id: ''` para alvos inexistentes (`MainEditor.vue:653-655`). Para nós:
   marcar visualmente e oferecer "criar arquivo" explícito.
4. **Criar ao clicar é opcional e escondido.** Só funciona com
   `zkn.customDirectory` configurado (`force-open.ts:57`). É uma boa ideia com
   má divulgação.
5. **Renomear pede confirmação e reescreve só um formato.** O diálogo
   "Update N internal links?" (`file-rename.ts:157-166`) é o mínimo aceitável;
   o usuário não é avisado de que `[x]([[alvo]])` ficará quebrado (o próprio
   teste do projeto aceita esse buraco, `test/replace-links.spec.ts:45-46`).
6. **Referências "de publicação" são uma lista, não um grafo.** O usuário
   monta o documento final arrastando arquivos para a lista do projeto
   (`fsal-directory.ts:37-46`) e exporta na ordem. Isso ensina que, para
   publicações, **ordem explícita > grafo implícito**.
7. **O painel "Related Files" separa causas.** Inbound primeiro, depois
   outbound, depois tags compartilhadas, com rótulos
   `bidirectional/inbound/outbound` (`win-main/sidebar/RelatedFilesTab.vue:147-196`). É uma UX
   honesta sobre *por que* dois arquivos estão relacionados.
8. **Citação é texto com semântica, não campo de formulário.** `@chave, p. 12`
   é parseado, renderizado como citação formatada e editado como texto
   (`citation-parser.ts:229-310`, `render-citations.ts:36-83`). Se nosso editor
   transformar citação em "chip" opaco, perdemos essa fluidez.
9. **Autocomplete ordenado por uso** (`autocomplete/citations.ts:47-90`) é um
   detalhe pequeno com grande efeito em textos longos.

---

## 4. Arquitetura reutilizável (o que levar para o núcleo Rust)

| Ideia do Zettlr | Evidência | Como adaptar |
|---|---|---|
| Descritor de arquivo derivado e serializável (`id, tags, links, citekeys`) | `types/common/fsal.ts:84-98` | Manter no núcleo (Rust) como struct `Serialize`; é o que permite cache incremental e consultas sem reabrir arquivos. |
| ID extraído de nome ou conteúdo, ignorando `[[...]]` | `extract-file-id.ts:33-56` | Copiar a *regra* (não o código): prioridade nome → conteúdo, com exclusão de contexto de link. Adicionar detecção de duplicados (o Zettlr só tem TODO, `links/index.ts:43-44`). |
| Índice `path → links[]` e `path → id`, derivado do FSAL | `links/index.ts:80-86` | Derivar do nosso índice de documentos; trocar reindexação total por atualização incremental por evento. |
| Resolução por ID, basename com e sem extensão | `links/index.ts:102-124` | Adicionar caminho relativo e desambiguação determinística + erro de ambiguidade (hoje: primeiro vence). |
| Reescrita de links no rename baseada em AST com `targetRange` | `replace-links.ts:35-75` | Reaproveitar o conceito de range do alvo; estender a links Markdown e frontmatter. |
| Autoridade única + `ChangeSet` versionado (OT) | `documents/index.ts:619-760`, `remote-doc.ts:54-164` | Nosso núcleo Rust já é a autoridade natural; expor `pull/push` com versão e `reload` quando o cliente sai da janela. |
| Autosave `off/delayed/immediate` com timeout e flush no save | `documents/index.ts:745-758`, `:1527-1600` | Modelar como política no núcleo, não no front-end. |
| Preservação de BOM/linefeed na gravação | `documents/index.ts:1581-1582` | Fazer no núcleo; evita diffs falsos. |
| Projeto = lista ordenada + perfis de export | `fsal-directory.ts:37-46`, `dir-project-export.ts:76-98` | Ótimo encaixe para "publicação" no nosso modelo: coleções ordenadas de documentos. |
| Export por defaults declarativos (YAML) + filtros | `commands/exporter/index.ts:164-280` | Equivalente: pipeline de export declarativo e auditável. |
| Presets de tema como override de variáveis | `theme/editor.ts:25-210`, `themes/berlin.ts:26-45` | Manter no front-end TS; o núcleo não deve emitir CSS. |

**O que não levar:** o uso de `@codemirror/collab` como transporte
renderer↔main (dependência pesada para um problema que um RPC versionado
resolve); a reindexação total por evento; a duplicação de `findExact`; a
ausência de diagnóstico de link quebrado; e qualquer código-fonte (GPL).

---

## 5. Lacunas e o que não foi verificado

- App não executado; nenhum comportamento visual observado.
- IME e clipboard não exercitados (código só mapeia teclas).
- Acessibilidade sem teste com leitor de tela; `aria-live` não é usado pelo
  projeto (fica por conta do CodeMirror 6).
- Testes de unidade não rodados: `node_modules` ausente; `mocha` não instalado.
  Os testes citados (`test/markdown-to-ast.spec.ts`,
  `test/replace-links.spec.ts`) foram lidos, não executados.
- Performances citadas são as declaradas em comentários do próprio código
  (`fsal-cache.ts:9-18`), não medidas por mim.
- Exportação Pandoc não executada (depende de binário Pandoc e do app).
