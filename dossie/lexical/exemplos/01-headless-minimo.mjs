// Probe 1 — o mínimo: editor headless, o que o núcleo faz sozinho, e o que
// só o "modo" (RichTextExtension) faz.
//
// Dependências: lexical@0.53.0 @lexical/rich-text@0.53.0 @lexical/history@0.53.0
// Não usa DOM, não usa React.
import {
  $createParagraphNode,
  $createTextNode,
  $getRoot,
  $getSelection,
  $isRangeSelection,
  CAN_UNDO_COMMAND,
  createEditor,
  FORMAT_TEXT_COMMAND,
  KEY_ENTER_COMMAND,
  REDO_COMMAND,
  UNDO_COMMAND,
} from 'lexical';
import {createEmptyHistoryState, registerHistory} from '@lexical/history';
import {registerRichText} from '@lexical/rich-text';

const log = (...a) => console.log(...a);

// ---------------------------------------------------------------- 1. sem modo
const bare = createEditor({namespace: 'bare'});
bare.update(() => {
  $getRoot().append($createParagraphNode().append($createTextNode('Hello world')));
}, {discrete: true});

log('=== 1) editor mínimo (sem rich-text) ===');
log(JSON.stringify(bare.getEditorState().toJSON(), null, 2));

let enterBare = null;
bare.update(() => {
  enterBare = bare.dispatchCommand(KEY_ENTER_COMMAND, null);
}, {discrete: true});
log('KEY_ENTER_COMMAND tratado só pelo núcleo?', enterBare);
log('filhos do root:', bare.read(() => $getRoot().getChildren().map(n => n.getType())));

// ---------------------------------------------------------------- 2. com modo
const rich = createEditor({namespace: 'rich'});
registerRichText(rich);
rich.update(() => {
  $getRoot().append($createParagraphNode().append($createTextNode('Hello world')));
}, {discrete: true});
rich.update(() => {
  const t = $getRoot().getFirstChild().getFirstChild();
  t.select(t.getTextContentSize(), t.getTextContentSize());
}, {discrete: true});

let enterRich = null;
rich.update(() => {
  enterRich = rich.dispatchCommand(KEY_ENTER_COMMAND, null);
}, {discrete: true});
log('=== 2) mesmo editor + registerRichText ===');
log('KEY_ENTER_COMMAND tratado?', enterRich);
log('filhos do root:', rich.read(() => $getRoot().getChildren().map(n => n.getType())));
log('texto:', JSON.stringify(rich.read(() => $getRoot().getTextContent())));

// ---------------------------------------------------------------- 3. negrito
rich.update(() => {
  const t = $getRoot().getFirstChild().getFirstChild();
  t.select(0, 5);
}, {discrete: true});
rich.update(() => {
  rich.dispatchCommand(FORMAT_TEXT_COMMAND, 'bold');
}, {discrete: true});
log('=== 3) FORMAT_TEXT_COMMAND "bold" nos 5 primeiros caracteres ===');
log('nós de texto:', JSON.stringify(rich.getEditorState().toJSON().root.children[0].children));

// ---------------------------------------------------------------- 4. undo/redo
const hist = createEditor({namespace: 'hist'});
registerRichText(hist);
registerHistory(hist, createEmptyHistoryState(), 300);

hist.update(() => {
  $getRoot().append($createParagraphNode().append($createTextNode('a')));
}, {discrete: true});
await new Promise(r => setTimeout(r, 400)); // passar a janela de merge
hist.update(() => {
  $getRoot().getFirstChild().getFirstChild().setTextContent('abc');
}, {discrete: true});

log('=== 4) undo / redo ===');
log('antes do undo:', hist.read(() => $getRoot().getTextContent()));
hist.update(() => hist.dispatchCommand(UNDO_COMMAND, undefined), {discrete: true});
log('depois do undo:', hist.read(() => $getRoot().getTextContent()));
hist.update(() => hist.dispatchCommand(REDO_COMMAND, undefined), {discrete: true});
log('depois do redo:', hist.read(() => $getRoot().getTextContent()));

// CAN_UNDO_COMMAND é notificação: dispatchCommand direto não devolve o estado.
let dispatchReturn;
hist.read(() => {
  dispatchReturn = hist.dispatchCommand(CAN_UNDO_COMMAND, undefined);
});
log('dispatchCommand(CAN_UNDO_COMMAND) direto ->', dispatchReturn, '(não significa "não há undo")');
