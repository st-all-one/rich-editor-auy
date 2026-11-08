// Demo headless de prosemirror-model/transform/state/history (sem DOM).
// Node 22, pacotes npm: model 1.25.12, transform 1.12.5, state 1.4.4,
// history 1.5.1, schema-basic 1.2.5.
const {schema} = require("prosemirror-schema-basic")
const {EditorState, TextSelection} = require("prosemirror-state")
const {history, undo, redo, undoDepth, redoDepth} = require("prosemirror-history")

const log = (...a) => console.log(...a)

// 1. Documento: posições e "tokens"
let doc = schema.node("doc", null, [
  schema.node("paragraph", null, [schema.text("Hello world")])
])
log("doc.toJSON        =", JSON.stringify(doc.toJSON()))
log("doc.nodeSize      =", doc.nodeSize, "| paragraph.nodeSize =", doc.child(0).nodeSize,
    "| text.nodeSize =", doc.firstChild.firstChild.nodeSize)

// 2. ResolvedPos
let $p = doc.resolve(6)
log("resolve(6)        =", JSON.stringify({
  pos: $p.pos, depth: $p.depth, parent: $p.parent.type.name,
  parentOffset: $p.parentOffset, start: $p.start(), end: $p.end(),
  textOffset: $p.textOffset, index: $p.index(), nodeAfter: $p.nodeAfter.text
}))
log("resolve(12).textOffset (end token) =", doc.resolve(12).textOffset)

// 3. Schema: expressão de conteúdo rejeita conteúdo inválido
try {
  schema.node("paragraph", null, [schema.node("paragraph")])
} catch (e) {
  log("conteúdo inválido ->", e.constructor.name + ": " + e.message)
}
log("createAndFill doc =", JSON.stringify(schema.topNodeType.createAndFill().toJSON()))

// 4. Transaction = Transform + seleção + meta
let state = EditorState.create({schema, doc, plugins: [history()]})
let tr = state.tr
tr.replaceWith(1, 6, schema.text("Howdy"))     // "Hello" -> "Howdy"
log("steps             =", JSON.stringify(tr.steps.map(s => s.toJSON())))
log("texto após step   =", JSON.stringify(tr.doc.textContent))
log("mapping.map(6, +1)=", tr.mapping.map(6, 1), "| map(6, -1) =", tr.mapping.map(6, -1))
log("StepMap ranges    =", JSON.stringify(tr.mapping.maps[0].toString()))
log("invert(step)      =", JSON.stringify(tr.steps[0].invert(state.doc).toJSON()))
let state1 = state.apply(tr)

// 5. Marks via AddMarkStep
let tr2 = state1.tr
tr2.addMark(1, 6, schema.marks.strong.create())
log("AddMarkStep JSON  =", JSON.stringify(tr2.steps[0].toJSON()))
log("marks no texto    =", JSON.stringify(tr2.doc.firstChild.firstChild.marks.map(m => m.type.name)))
let state2 = state1.apply(tr2)

// 6. Undo/redo com o plugin history
let s = state2
log("undoDepth         =", undoDepth(s))
undo(s, t => { s = s.apply(t) })
log("após undo 1 text  =", JSON.stringify(s.doc.textContent),
    "marks =", JSON.stringify(s.doc.firstChild.firstChild.marks.map(m => m.type.name)),
    "| undoDepth =", undoDepth(s), "redoDepth =", redoDepth(s))
undo(s, t => { s = s.apply(t) })
log("após undo 2 text  =", JSON.stringify(s.doc.textContent))
redo(s, t => { s = s.apply(t) })
log("após redo text    =", JSON.stringify(s.doc.textContent))

// 7. Seleção sobrevive ao mapeamento
let st = EditorState.create({schema, doc, selection: TextSelection.create(doc, 1, 6)})
let t3 = st.tr
t3.insertText("X", 1)
log("seleção            =", JSON.stringify({from: st.selection.from, to: st.selection.to}),
    "-> após insert:", JSON.stringify({from: t3.selection.from, to: t3.selection.to}))
