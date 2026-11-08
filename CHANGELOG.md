# Changelog

Todas as mudanças relevantes deste projeto são registradas aqui. O formato segue Keep a Changelog
e o projeto usa Versionamento Semântico.

## [Não publicado]

### Adicionado

- Workspace Rust `rich-editor-auy` com core `no_std`, macros, bindings WASM e testkit.
- Modelo canônico de blocos, schema builder, expressões de conteúdo, posições e wire JSON.
- Fingerprint BLAKE3-256 e envelope durável.
- Árvore semântica de acessibilidade, validação com códigos estáveis e projeção JSON canônica.
- Steps estruturais: join de blocos, inversão de split e slices de fragmento.
- Índice ordenado de início de blocos (`Doc::block_at`) para consultas locais.
- API WASM para split, delete, mark, join e árvore de acessibilidade.
- Adaptador DOM semântico sem `class`/`style`, com renderização de headings, listas, links, mídia e
  tabelas, e restauração do host em `destroy()`.
- Pacotes Deno/JSR com testes; pacote Composer com validação estrita de envelope.
- CLI `diff` e `a11y-tree`.
- Testes de alocação, memória, visitas; benchmarks `criterion`; alvo de fuzz; exemplos TS/PHP/host.
- Gate local `make check`, `make visits`, `make check-ecosystem` e pipeline de CI com wasm/js/php/bench/fuzz.
- Registro de divergências em `DIVERGENCES.md`.

### Alterado

- `schema!` deixou de ser placeholder: proc-macro real para doc, nós, marcas, grupos, conteúdo e
  atributos tipados, com teste de integração e expansão `no_std`.
- Parser JSON decodifica surrogate pairs, reporta offsets de origem nos erros e aplica os limites
  de attrs, marcas, runs e texto no documento final.
- A árvore de acessibilidade carrega `DocPath` e a validação usa o path real do documento.
- Substituições de bloco no caminho quente reutilizam o índice de inícios e revalidam apenas o
  bloco alterado.
- Blocos de topo passaram a ser armazenados em chunks de 128; S2 caiu de ~106 µs para ~3,2 µs.

### Corrigido

- Inversão de múltiplos steps, `StepMap` de split, preservação de marcas em `Slice::runs`, remoção
  de attrs por `Null`, validação estrutural no wire, allowlist de URLs no DOM e envelope no CLI/TS.

### Removido

- Nada ainda.
