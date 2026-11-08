import {readFileSync} from 'node:fs';
import {Schema, Node} from 'prosemirror-model';
import {EditorState} from 'prosemirror-state';
const schema = new Schema({
  nodes: {
    doc: {content: 'block+', attrs: {lang: {default: null}, dir: {default: null}}},
    paragraph: {content: 'inline*', group: 'block'},
    heading: {content: 'inline*', group: 'block', attrs: {level: {default: 1}}, defining: true},
    bullet_list: {content: 'list_item+', group: 'block'},
    list_item: {content: 'block+', defining: true},
    text: {group: 'inline'},
  },
  marks: {
    strong: {}, em: {}, code: {},
    link: {attrs: {href: {}}, inclusive: false},
    ref: {attrs: {target: {}}},
  },
});
const json = JSON.parse(readFileSync(process.argv[2] ?? '/tmp/complex.json', 'utf8'));
let node;
const time = (label, fn, n = 100) => {
  for (let i=0;i<5;i++) fn();
  const t0=performance.now();
  for (let i=0;i<n;i++) fn();
  const ms=(performance.now()-t0)/n;
  console.log(label, ms.toFixed(3), 'ms');
};
time('PM fromJSON', () => { node = Node.fromJSON(schema, json); }, 100);
time('PM toJSON', () => { JSON.stringify(node.toJSON()); }, 100);
const state = EditorState.create({schema, doc: node});
let next;
time('PM apply insertText', () => {
  const tr = state.tr.insertText('x', 2);
  next = state.apply(tr);
}, 100);
console.log('PM doc blocks', node.childCount, 'size', node.content.size);
