// Exemplo mínimo canônico do Tiptap (Node + jsdom). NÃO EXECUTADO neste dossiê:
// o diretório ref/tiptap não tem node_modules e instalar as dependências não é
// permitido nesta tarefa. Ver exemplos/README.md §4 para como rodar.
//
// O mesmo fluxo foi observado em browser em exemplos/cdn/index.html.
//
// Instalação:
//   npm i @tiptap/core @tiptap/pm @tiptap/starter-kit jsdom
//
// Saída esperada (observada no browser):
//   <p><strong>hello</strong></p>
//   {"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","marks":[{"type":"bold"}],"text":"hello"}]}]}
//   hello

import { JSDOM } from 'jsdom'

const dom = new JSDOM('<!doctype html><html><body><div id="app"></div></body></html>')
globalThis.window = dom.window
globalThis.document = dom.window.document

const { Editor } = await import('@tiptap/core')
const { default: StarterKit } = await import('@tiptap/starter-kit')

const editor = new Editor({
  element: document.getElementById('app'),
  extensions: [StarterKit],
  content: '<p>hello</p>',
  // Para zero CSS no <head>, descomente. O padrão é true (core/src/Editor.ts:112).
  // injectCSS: false,
})

editor.commands.selectAll()
editor.commands.toggleBold()

console.log(editor.getHTML())
console.log(JSON.stringify(editor.getJSON()))
console.log(editor.getText())

// Persistência: o JSON é o formato estável (mesmo do ProseMirror).
const saved = JSON.stringify(editor.getJSON())
editor.destroy()

const restored = new Editor({
  element: document.getElementById('app'),
  extensions: [StarterKit],
  content: JSON.parse(saved),
})
console.log(restored.getHTML())
restored.destroy()
