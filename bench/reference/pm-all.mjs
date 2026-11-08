import {readFileSync} from 'node:fs';
import {Schema, Node} from 'prosemirror-model';
import {EditorState} from 'prosemirror-state';
const base={id:{default:null}};
const schema = new Schema({
  nodes: {
    doc: {content: 'block+', attrs: {lang:{default:null}, dir:{default:null}, title:{default:null}}},
    paragraph: {content: '(text | image)*', group: 'block', attrs: base},
    heading: {content: '(text | image)*', group: 'block', attrs: {id:{default:null}, level:{default:2}}, defining: true},
    bullet_list: {content: 'list_item+', group: 'block'},
    list_item: {content: 'block+', defining: true},
    blockquote: {content: 'block+', group: 'block'},
    code_block: {content: 'text*', group: 'block', marks: ''},
    table: {content: 'table_row+', group: 'block'},
    table_row: {content: '(table_cell | table_header)+'},
    table_cell: {content: 'block+'},
    table_header: {content: 'block+'},
    separator: {group: 'block', atom: true, selectable: false},
    image: {inline: true, group: 'inline', atom: true, attrs: {alt:{}, src:{default:null}}},
    text: {group: 'inline'},
  },
  marks: {
    strong: {}, em: {}, code: {},
    link: {attrs: {href: {}}, inclusive: false},
    ref: {attrs: {target: {}}},
  },
});
const json=JSON.parse(readFileSync('/tmp/complex-all.json','utf8'));
let node;
const time=(label,fn,n=50)=>{for(let i=0;i<5;i++)fn(); const t=performance.now(); for(let i=0;i<n;i++)fn(); console.log(label, ((performance.now()-t)/n).toFixed(3),'ms');};
time('PM all fromJSON',()=>{node=Node.fromJSON(schema,json);},50);
time('PM all toJSON',()=>{JSON.stringify(node.toJSON());},50);
const state=EditorState.create({schema,doc:node});
time('PM all apply',()=>{const tr=state.tr.insertText('x', 2); state.apply(tr);},50);
console.log('blocks',node.childCount,'bytes',JSON.stringify(node.toJSON()).length);
