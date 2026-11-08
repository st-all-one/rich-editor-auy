// Probe 2 — serialização (JSON e HTML), parse de volta e nó customizado.
//
// Dependências: lexical@0.53.0 @lexical/rich-text@0.53.0 @lexical/html@0.53.0 happy-dom
// O happy-dom entra só para dar um `document` a $generateHtmlFromNodes.
import {Window} from 'happy-dom';

const win = new Window();
globalThis.window = win;
globalThis.document = win.document;
Object.defineProperty(globalThis, 'navigator', {value: win.navigator, configurable: true});

const {
  $createParagraphNode,
  $createTextNode,
  $getRoot,
  $getSelection,
  createEditor,
  TextNode,
} = await import('lexical');
const {registerRichText} = await import('@lexical/rich-text');
const {$generateHtmlFromNodes, $generateNodesFromDOM} = await import('@lexical/html');

const log = (...a) => console.log(...a);

const THEME = {
  paragraph: 'par',
  text: {
    bold: 'b',
    italic: 'i',
    underline: 'u',
    strikethrough: 's',
    code: 'c',
    underlineStrikethrough: 'us',
  },
};

// ------------------------------------------------ 1. HTML exportado com tema
const ed = createEditor({namespace: 'html', theme: THEME});
registerRichText(ed);
ed.update(() => {
  $getRoot().append($createParagraphNode().append($createTextNode('bold italic underline')));
}, {discrete: true});
ed.update(() => {
  const t = $getRoot().getFirstChild().getFirstChild();
  t.select(0, 4);
  $getSelection().formatText('bold');
}, {discrete: true});
ed.update(() => {
  const runs = $getRoot().getFirstChild().getChildren();
  const t = runs.find(r => r.getTextContent().includes('italic'));
  if (t) {
    const i = t.getTextContent().indexOf('italic');
    t.select(i, i + 6);
    $getSelection().formatText('italic');
  }
}, {discrete: true});

log('=== 1) HTML exportado (com tema) ===');
log(ed.read(() => $generateHtmlFromNodes(ed, null)));
log('=== 2) mesmos nós em JSON ===');
log(JSON.stringify(ed.getEditorState().toJSON().root.children[0].children));
log('runs de texto:', ed.read(() => $getRoot().getFirstChild().getChildren().length));

// ------------------------------------------------ 3. parseEditorState round-trip
const ed2 = createEditor({namespace: 'json'});
registerRichText(ed2);
const state = ed2.parseEditorState({
  root: {
    children: [
      {
        children: [
          {type: 'text', text: 'from json', format: 1, detail: 0, mode: 'normal', style: '', version: 1},
        ],
        type: 'paragraph',
        version: 1,
      },
    ],
    type: 'root',
    version: 1,
  },
});
ed2.setEditorState(state);
log('=== 3) parseEditorState -> setEditorState ===');
log('texto:', JSON.stringify(ed2.read(() => $getRoot().getTextContent())));

// ------------------------------------------------ 4. HTML -> nós (link cai)
const dom = new win.DOMParser().parseFromString(
  '<p>hi <strong>bold</strong> <a href="https://example.com">link</a></p>',
  'text/html',
);
ed2.update(() => {
  const nodes = $generateNodesFromDOM(ed2, dom);
  $getRoot().clear().append(...nodes);
}, {discrete: true});
log('=== 4) $generateNodesFromDOM (sem @lexical/link instalado) ===');
log(JSON.stringify(ed2.getEditorState().toJSON().root.children[0].children));
log('^ o <a> virou texto puro: sem LinkNode registrado, não há regra de import para <a>');

// ------------------------------------------------ 5. nó customizado + transform
let transformRuns = 0;
class LoudTextNode extends TextNode {
  static getType() {
    return 'loud-text';
  }
  static clone(node) {
    return new LoudTextNode(node.__text, node.__key);
  }
  static transform() {
    return node => {
      transformRuns++;
      const t = node.getTextContent();
      if (t !== t.toUpperCase()) {
        node.setTextContent(t.toUpperCase());
      }
    };
  }
}

const ed3 = createEditor({namespace: 'custom', nodes: [LoudTextNode]});
ed3.update(() => {
  $getRoot().append($createParagraphNode().append(new LoudTextNode('hello transform')));
}, {discrete: true});
log('=== 5) nó customizado com static transform() ===');
log('execuções do transform:', transformRuns);
log('texto final:', JSON.stringify(ed3.read(() => $getRoot().getTextContent())));
log('JSON do nó:', JSON.stringify(ed3.getEditorState().toJSON().root.children[0].children[0]));
