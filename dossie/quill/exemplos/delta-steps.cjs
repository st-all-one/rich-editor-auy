// Demonstração dos operadores do quill-delta 5.1.0 (MIT), executada localmente.
const Delta = require('/tmp/qd/package/dist/Delta.js');

const out = [];
const log = (label, value) => out.push([label, value]);

const base = new Delta()
  .insert('Olá')
  .insert(' mundo', { bold: true })
  .insert('\n', { header: 1 });

log('doc.ops', base.ops);
log('doc.length', base.length());

// insert (texto) + delete + retain com atributos = edição de um usuário
const edit = new Delta().retain(4).insert('!').delete(1).retain(5, { italic: true });
log('edit.ops', edit.ops);

const afterEdit = base.compose(edit);
log('compose(doc, edit).ops', afterEdit.ops);

// diff: do documento antigo para o novo, com attributes
const derived = base.diff(afterEdit);
log('diff(doc, afterEdit).ops', derived.ops);
log('diff igual a edit (sem padronização)', JSON.stringify(derived.ops) === JSON.stringify(edit.ops));

// invert: como desfazer a edição a partir do documento base
const undo = edit.invert(base);
log('invert(edit, base).ops', undo.ops);
log('compose(compose(doc,edit),invert) == doc', JSON.stringify(base.compose(edit).compose(undo).ops) === JSON.stringify(base.ops));

// transform: duas edições concorrentes no mesmo documento (OT)
const a = new Delta().retain(4).insert('A');
const b = new Delta().retain(2).insert('B');
log('transform(a,b,priority=false).ops', a.transform(b, false).ops);
log('transform(a,b,priority=true).ops', a.transform(b, true).ops);
log('transformPosition(4) por b', b.transformPosition(4));
log('transformPosition(4) por b com priority', b.transformPosition(4, true));

// atributos: null remove; compose preserva o atributo não mexido
const fmt1 = new Delta().retain(4).retain(6, { bold: null });
const fmt2 = new Delta().retain(4).retain(6, { color: '#ff0000' });
log('compose(fmt1,fmt2).ops', new Delta().compose(fmt1).compose(fmt2).ops);
log('docFormatado.ops', base.compose(fmt1).ops);

// delete + insert no mesmo índice: ordem canônica é insert antes de delete
log('push insertThenDelete.ops', new Delta().delete(2).insert('x').ops);
log('chop remove retain final', new Delta().retain(3).chop().ops);

// embed handler: necessário para compose/transform de retains em objetos
Delta.registerEmbed('img', {
  compose: (x, y) => ({ ...x, ...y }),
  transform: (x, y) => y,
  invert: (x, y) => y,
});
const emb = new Delta().insert({ img: { src: 'a.png' } }).insert('\n');
const embEdit = new Delta().retain(1, { img: { width: 10 } });
log('embed compose.ops', emb.compose(embEdit).ops);
try {
  new Delta().insert({ video: 'x' }).compose(new Delta().retain(1, { video: 'y' }));
} catch (e) {
  log('embed sem handler lança', String(e.message));
}

console.log(JSON.stringify(out, null, 2));
