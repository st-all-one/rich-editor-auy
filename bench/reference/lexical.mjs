import {Window} from 'happy-dom';
const window = new Window();
globalThis.window = window;
globalThis.document = window.document;
const {createHeadlessEditor} = await import('@lexical/headless');
const {HeadingNode, $createHeadingNode} = await import('@lexical/rich-text');
const {ListNode, ListItemNode, $createListNode, $createListItemNode} = await import('@lexical/list');
const {LinkNode, $createLinkNode} = await import('@lexical/link');
const {$getRoot, $createParagraphNode, $createTextNode} = await import('lexical');
const editor = createHeadlessEditor({namespace:'bench', nodes:[HeadingNode,ListNode,ListItemNode,LinkNode], onError: (e)=>{throw e;}});
editor.update(() => {
  const root = $getRoot();
  for (let i=0;i<500;i++) {
    const h=$createHeadingNode('h2'); const ht=$createTextNode('Título '); ht.toggleFormat('bold'); const ht2=$createTextNode('rico'); h.append(ht,ht2); root.append(h);
    const p=$createParagraphNode();
    p.append($createTextNode('Texto '));
    const b=$createTextNode('forte'); b.toggleFormat('bold'); p.append(b);
    p.append($createTextNode(' e '));
    const em=$createTextNode('ênfase'); em.toggleFormat('italic'); p.append(em);
    p.append($createTextNode(' com '));
    const code=$createTextNode('código'); code.toggleFormat('code'); p.append(code);
    p.append($createTextNode(' e '));
    const link=$createLinkNode('https://example.test/a'); link.append($createTextNode('link')); p.append(link);
    p.append($createTextNode('.'));
    root.append(p);
    for (let j=0;j<2;j++) {
      const list=$createListNode('bullet'); const li=$createListItemNode(); const lp=$createParagraphNode(); lp.append($createTextNode('Item de lista')); li.append(lp); list.append(li); root.append(list);
    }
    const hp=$createHeadingNode('h3'); hp.append($createTextNode('Subseção')); root.append(hp);
    const lp=$createParagraphNode(); lp.append($createTextNode('Parágrafo longo para medir texto e marcas. ')); const nb=$createTextNode('negrito'); nb.toggleFormat('bold'); lp.append(nb); lp.append($createTextNode(' normal ')); const it=$createTextNode('itálico'); it.toggleFormat('italic'); lp.append(it); root.append(lp);
  }
}, {discrete:true});
const serialized = JSON.stringify(editor.getEditorState().toJSON());
console.log('Lexical nodes', editor.getEditorState().toJSON().root.children.length, 'bytes', serialized.length);
const measure=(label,fn,n=50)=>{for(let i=0;i<3;i++)fn(); const t=performance.now(); for(let i=0;i<n;i++)fn(); console.log(label,((performance.now()-t)/n).toFixed(3),'ms');};
measure('Lexical parseEditorState', () => editor.parseEditorState(serialized), 50);
measure('Lexical toJSON', () => JSON.stringify(editor.getEditorState().toJSON()), 50);
let parsed;
measure('Lexical parse+set+toJSON', () => { const s=editor.parseEditorState(serialized); editor.setEditorState(s); parsed=editor.getEditorState().toJSON(); }, 20);
