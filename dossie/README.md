# Dossiês de referência

Estudos de caso dos projetos em `ref/`, feitos para decidir o que o `rich-editor-auy` deve copiar,
adaptar ou evitar. Comece pelo [`_GUIA.md`](_GUIA.md); a proveniência das cópias está em
[`_PROVENIENCIA.md`](_PROVENIENCIA.md); a rubrica de notas está em [`_RUBRICA.md`](_RUBRICA.md).

| Projeto | O que é | Dossiê | Revisão independente |
|---|---|---|---|
| Lexical | framework de editor da Meta | [`lexical/`](lexical/) | 87% |
| ProseMirror | toolkit canônico de editor estruturado | [`prosemirror/`](prosemirror/) | 88% |
| Quill | editor com modelo Delta | [`quill/`](quill/) | 87% |
| Summernote | WYSIWYG jQuery/Bootstrap | [`summernote/`](summernote/) | 86% |
| Tiptap | framework sobre ProseMirror | [`tiptap/`](tiptap/) | 88% |
| taino-edit | editor Rust/Leptos | [`taino-edit/`](taino-edit/) | 85% |
| html-auy | HTML tipado em Rust (não é editor) | [`html-auy/`](html-auy/) | 88% |
| knudge | memória/processo em Rust (não é editor) | [`knudge/`](knudge/) | 88% |
| MarkText | app Markdown WYSIWYG (Muya) | [`marktext/`](marktext/) | 82% (13/13; sem execução) |
| Zettlr | app Markdown/Zettelkasten | [`Zettlr/`](Zettlr/) | 88% |
| akribie / akribiechart | compilador de diagramas em Rust (não é editor) | [`akribiechart/`](akribiechart/) | 90% (13/13) |
| calc-auy | calculadora exata/auditável em Rust (não é editor) | [`calc-auy/`](calc-auy/) | 87% (12/13; performance não medida) |

O quadro comparativo final está em [`QUADRO-COMPARATIVO.md`](QUADRO-COMPARATIVO.md); as ideias
cruzadas de akribie/calc-auy para o `rich-editor-auy` estão em
[`IDEIAS-akribie-calc.md`](IDEIAS-akribie-calc.md). Cada pasta tem `README.md`, `arquitetura.md`,
`avaliacao.md`, `codigo/`, `exemplos/` e `revisao.md`; a confiança mede evidência verificada, não
entusiasmo.
