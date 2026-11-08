# Proveniência das cópias em `ref/`

Registrado em 2026-11-07; adendo de akribie/calc-auy em 2026-11-08. Hashes locais; não são releases oficiais. Os dossiês devem citar
também a versão de pacote/crate quando ela existir.

| Projeto | Commit local | Data | Versão declarada | Licença | Observação |
|---|---|---|---|---|---|
| lexical | `5326e06` | 2026-11-06 | `@lexical/monorepo` 0.53.0 | MIT | monorepo completo |
| prosemirror | `c7f2f1d` | 2026-04-01 | `prosemirror` 0.0.0 | MIT | **repositório de encaminhamento**: o código dos pacotes não está aqui; só `bin/pm.js`, demo e docs |
| quill | `539cbff` | 2025-07-25 | `quill-monorepo` 2.0.3 | BSD-3-Clause | monorepo |
| summernote | `ac5460e` | 2026-01-04 | `summernote` 0.9.1 | MIT | jQuery/Bootstrap |
| tiptap | `5fd5d2f` | 2026-11-05 | `@tiptap/core` 3.30.3 | MIT | monorepo |
| taino-edit | `90ba5ed` | 2026-09-05 | 0.7.0 | MIT OR Apache-2.0 | Rust/Leptos |
| html-auy | `c70213e` | 2026-11-07 | 0.1.1 | MIT OR Apache-2.0 | Rust, não é editor |
| knudge | `98e8bac` | 2026-10-29 | 0.6.4 | MIT OR Apache-2.0 | memória/processo, não é editor |
| marktext | `a520bef` | 2026-11-06 | `marktext-monorepo` 0.24.0-dev; `@muyajs/core` 0.2.0 | MIT | Electron + engine Muya |
| Zettlr | `87eac9e` | 2026-11-07 | `zettlr` 4.9.0 | GPL-3.0 | app Electron; copiar só trechos para análise |
| akribie / akribiechart | `7275fed` | 2026-11-03 | 0.1.3 | MIT OR Apache-2.0 | produto **akribie**; checkout em `ref/akribiechart`; CLI `akr`; não é editor |
| calc-auy | `08f5a3f` | 2026-10-04 | 0.1.0 | MPL-2.0 | calculadora exata/auditável em Rust; commit assinado, chave pública ausente no ambiente |

## Consequência para o dossiê do ProseMirror

O `ref/prosemirror/` local **não contém** `packages/*/src`. Ele é o repositório de encaminhamento
que aponta para `code.haverbeke.berlin`. Para um dossiê de código real, é necessário obter os
pacotes publicados (`prosemirror-model`, `prosemirror-transform`, `prosemirror-state`,
`prosemirror-view`, `prosemirror-schema-basic`, `prosemirror-history`, `prosemirror-collab`) ou
o repositório upstream. O dossiê deve declarar essa limitação se não fizer a coleta.
