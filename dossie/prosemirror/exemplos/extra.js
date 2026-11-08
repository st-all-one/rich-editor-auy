const {schema} = require("prosemirror-schema-basic")
const {EditorState} = require("prosemirror-state")
const {history, undo, undoDepth} = require("prosemirror-history")
let doc = schema.node("doc", null, [schema.node("paragraph", null, [schema.text("abc def")])])
let st = EditorState.create({schema, doc, plugins: [history()]})
// duas inserções adjacentes em sequência rápida
let s = st
for (const ch of ["1", "2"]) {
  let tr = s.tr.insertText(ch, 7)
  s = s.apply(tr)
}
console.log("texto:", s.doc.textContent, "| undoDepth após 2 digitações adjacentes:", undoDepth(s))
undo(s, t => s = s.apply(t))
console.log("após 1 undo:", s.doc.textContent)
// mapResult com deleção
let tr = st.tr.delete(2, 5)
console.log("delete 2..5 ->", JSON.stringify(tr.doc.textContent))
for (const p of [2, 3, 4, 5]) {
  const r = tr.mapping.mapResult(p)
  console.log(`mapResult(${p}) ->`, JSON.stringify({pos: r.pos, deleted: r.deleted, deletedBefore: r.deletedBefore, deletedAfter: r.deletedAfter, deletedAcross: r.deletedAcross}))
}
console.log("Slice JSON:", JSON.stringify(tr.doc.slice(1, 5).toJSON()))
let $p = tr.doc.resolve(2)
console.log("nodeBefore:", $p.nodeBefore && $p.nodeBefore.text, "| nodeAfter:", $p.nodeAfter && $p.nodeAfter.text)
