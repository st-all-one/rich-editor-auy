// Driver mínimo de Chrome DevTools Protocol para medir o Summernote com
// eventos de teclado REAIS (o `--dump-dom` não dispara default actions).
//
// Uso: node drive.mjs
// Requer: chromium-browser no PATH, Node >= 22 (WebSocket global), rede (CDN).
//
// Nada aqui altera o projeto de referência; é uma sonda externa.

import { spawn } from 'node:child_process';

const PORT = 9333;
const PAGE = new URL('./cdp.html', import.meta.url).href;

const chrome = spawn('chromium-browser', [
  '--headless=new', '--disable-gpu', '--no-sandbox',
  `--remote-debugging-port=${PORT}`,
  '--user-data-dir=/tmp/summernote-cdp-profile',
  'about:blank',
], { stdio: 'ignore' });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function waitForDevtools() {
  for (let i = 0; i < 100; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${PORT}/json/version`);
      if (r.ok) return await r.json();
    } catch { /* ainda subindo */ }
    await sleep(100);
  }
  throw new Error('devtools não respondeu');
}

let nextId = 1;
function makeClient(ws) {
  const pending = new Map();
  ws.addEventListener('message', (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.id && pending.has(msg.id)) {
      const { resolve, reject } = pending.get(msg.id);
      pending.delete(msg.id);
      msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
    }
  });
  return (method, params = {}) => new Promise((resolve, reject) => {
    const id = nextId++;
    pending.set(id, { resolve, reject });
    ws.send(JSON.stringify({ id, method, params }));
  });
}

const out = [];
const rec = (k, v) => {
  out.push(`${k}\t${typeof v === 'string' ? v : JSON.stringify(v)}`);
  console.log(`${k}\t${typeof v === 'string' ? v : JSON.stringify(v)}`);
};

try {
  await waitForDevtools();
  const target = await (await fetch(
    `http://127.0.0.1:${PORT}/json/new?${encodeURIComponent(PAGE)}`,
    { method: 'PUT' },
  )).json();

  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((res, rej) => {
    ws.addEventListener('open', res, { once: true });
    ws.addEventListener('error', rej, { once: true });
  });
  const send = makeClient(ws);

  await send('Page.enable');
  await send('Runtime.enable');
  await sleep(2500); // CDN + init

  const evalJs = async (expression) => {
    const r = await send('Runtime.evaluate', {
      expression, returnByValue: true, awaitPromise: true,
    });
    if (r.exceptionDetails) {
      return `THREW ${r.exceptionDetails.text} ${r.exceptionDetails.exception?.description || ''}`;
    }
    return r.result.value;
  };

  rec('init', await evalJs(`(function(){
    if (!$ || !$.summernote) return 'libs ausentes';
    $('#note').summernote({ height: 200, tabSize: 4 });
    $('#note').summernote('code', '<p>alpha beta</p>');
    return { version: $.summernote.version, editable: !!document.querySelector('.note-editable') };
  })()`));

  // Foca o editable no fim do primeiro parágrafo.
  rec('focus', await evalJs(`(function(){
    var e = document.querySelector('.note-editable');
    e.focus();
    var p = e.querySelector('p'); var t = p.firstChild;
    var r = document.createRange(); r.setStart(t, t.nodeValue.length); r.collapse(true);
    var s = getSelection(); s.removeAllRanges(); s.addRange(r);
    return document.activeElement.className;
  })()`));

  // Teclado REAL: digitação caractere a caractere.
  const typeChar = async (ch, code) => {
    await send('Input.dispatchKeyEvent', { type: 'keyDown', text: ch, key: ch, code, windowsVirtualKeyCode: ch.toUpperCase().charCodeAt(0) });
    await send('Input.dispatchKeyEvent', { type: 'keyUp', key: ch, code, windowsVirtualKeyCode: ch.toUpperCase().charCodeAt(0) });
    await sleep(15);
  };
  for (const ch of 'XYZ') await typeChar(ch, `Key${ch}`);
  rec('type.afterXYZ', await evalJs(`$('#note').summernote('code')`));

  // Ctrl+B real (modifiers: Ctrl = 2).
  await send('Input.dispatchKeyEvent', { type: 'keyDown', modifiers: 2, key: 'b', code: 'KeyB', windowsVirtualKeyCode: 66 });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', modifiers: 2, key: 'b', code: 'KeyB', windowsVirtualKeyCode: 66 });
  await sleep(60);
  rec('type.ctrlB', await evalJs(`$('#note').summernote('code')`));

  // Tab real: sai do editor (comportamento nativo) ou insere espaços?
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 });
  await sleep(80);
  rec('key.tab', await evalJs(`({
    html: $('#note').summernote('code'),
    active: document.activeElement.tagName + '.' + document.activeElement.className,
    isNextButton: document.activeElement.id === 'after'
  })`));

  // Shift+Tab (sai para trás).
  await evalJs(`document.querySelector('.note-editable').focus()`);
  await send('Input.dispatchKeyEvent', { type: 'keyDown', modifiers: 8, key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', modifiers: 8, key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 });
  await sleep(80);
  rec('key.shiftTab', await evalJs(`({ html: $('#note').summernote('code'), active: document.activeElement.tagName + '.' + document.activeElement.className })`));

  // IME: composição coreana/japonesa simulada.
  await evalJs(`(function(){
    var e = document.querySelector('.note-editable');
    e.focus();
    var p = e.querySelector('p'); var t = p.firstChild;
    var r = document.createRange(); r.setStart(t, t.nodeValue.length); r.collapse(true);
    var s = getSelection(); s.removeAllRanges(); s.addRange(r); return true;
  })()`);
  await send('Input.imeSetComposition', { text: 'に', selectionStart: 1, selectionEnd: 1 });
  await sleep(50);
  rec('ime.composing', await evalJs(`({ html: $('#note').summernote('code'), stack: $('#note').data('summernote').modules.editor.history.stack.length })`));
  await send('Input.insertText', { text: 'にほん' });
  await sleep(60);
  rec('ime.committed', await evalJs(`({ html: $('#note').summernote('code'), stack: $('#note').data('summernote').modules.editor.history.stack.length })`));

  // Enter real (divisão de parágrafo).
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, text: '\r' });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
  await sleep(60);
  rec('key.enter', await evalJs(`$('#note').summernote('code')`));

  // Undo real (Ctrl+Z) após digitação.
  await send('Input.dispatchKeyEvent', { type: 'keyDown', modifiers: 2, key: 'z', code: 'KeyZ', windowsVirtualKeyCode: 90, commands: ['undo'] });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', modifiers: 2, key: 'z', code: 'KeyZ', windowsVirtualKeyCode: 90 });
  await sleep(120);
  rec('key.ctrlZ', await evalJs(`$('#note').summernote('code')`));

  // Histórico: quantos snapshots e quantos bytes para um doc de ~30 KB.
  rec('perf.doc', await evalJs(`(function(){
    var paras = [];
    for (var i = 0; i < 300; i++) paras.push('<p>Parágrafo ' + i + ' com <b>negrito</b> e <i>itálico</i>.</p>');
    var html = paras.join('');
    $('#note').summernote('code', html);
    var hist = $('#note').data('summernote').modules.editor.history;
    hist.commit();
    var e = document.querySelector('.note-editable');
    var lastP = e.querySelectorAll('p')[e.querySelectorAll('p').length - 1];
    e.focus();
    var t = lastP.firstChild;
    var r = document.createRange(); r.setStart(t, t.nodeValue.length); r.collapse(true);
    var s = getSelection(); s.removeAllRanges(); s.addRange(r);
    var t0 = performance.now();
    for (var k = 0; k < 30; k++) document.execCommand('insertText', false, 'x');
    var t1 = performance.now();
    var bytes = hist.stack.reduce(function(a, sn) { return a + sn.contents.length; }, 0);
    return {
      docBytes: html.length,
      keystrokes: 30,
      totalMs: +(t1 - t0).toFixed(1),
      perKeystrokeMs: +((t1 - t0) / 30).toFixed(2),
      stackLen: hist.stack.length,
      snapshotBytes: bytes,
      historyLimit: $('#note').data('summernote').options.historyLimit,
    };
  })()`));

  rec('perf.readCode', await evalJs(`(function(){
    var t0 = performance.now();
    for (var i = 0; i < 20; i++) $('#note').summernote('code');
    var t1 = performance.now();
    return { perReadMs: +((t1 - t0) / 20).toFixed(2) };
  })()`));

  rec('perf.undoOne', await evalJs(`(function(){
    var t0 = performance.now();
    $('#note').summernote('undo');
    var t1 = performance.now();
    return { undoMs: +(t1 - t0).toFixed(1) };
  })()`));

  // XSS: o HTML "salvo" reexecuta handlers ao ser recarregado?
  rec('xss.reloadOnerror', await evalJs(`(function(){
    window.__pwned = 'no';
    $('#note').summernote('code', '<img src="nope.png" onerror="window.__pwned=\\'yes\\'">');
    return new Promise(function(res){
      setTimeout(function(){ res({ pwned: window.__pwned, html: $('#note').summernote('code') }); }, 400);
    });
  })()`));

  rec('xss.reloadJavascriptHref', await evalJs(`(function(){
    window.__pwned2 = 'no';
    $('#note').summernote('code', '<p><a id="j" href="javascript:window.__pwned2=\\'yes\\'">go</a></p>');
    document.getElementById('j').click();
    return { pwned2: window.__pwned2, html: $('#note').summernote('code') };
  })()`));

  // iframe não-whitelisted dentro de HTML carregado (aqui não passa por purify).
  rec('iframe.reloadEvil', await evalJs(`(function(){
    $('#note').summernote('code', '<p>x</p><iframe src="https://evil.example/x" width="200" height="80"></iframe>');
    return $('#note').summernote('code');
  })()`));

  // Cópia: o handler `copy` só dispara callback (não reescreve o clipboard).
  rec('clipboard.copyHandlerExists', await evalJs(`(function(){
    var fired = 0;
    $('#note').on('summernote.copy', function(){ fired++; });
    $('#note').summernote('code', '<p>texto</p>');
    document.querySelector('.note-editable').focus();
    document.execCommand('selectAll');
    document.execCommand('copy');
    return new Promise(function(res){ setTimeout(function(){ res({ copyEvents: fired }); }, 200); });
  })()`));

  // Ordem de tabulação: os botões têm tabindex=-1?
  rec('a11y.toolbarTabOrder', await evalJs(`(function(){
    var btns = Array.prototype.slice.call(document.querySelectorAll('.note-toolbar button'));
    var tabbable = btns.filter(function(b){ return b.tabIndex >= 0; });
    return {
      toolbarButtons: btns.length,
      tabbable: tabbable.length,
      tabbableClasses: tabbable.slice(0, 5).map(function(b){ return b.className; }),
      dropdownItemsFocusable: document.querySelectorAll('.note-toolbar .dropdown-item').length,
      dropdownItemTabIndex: (document.querySelector('.note-toolbar .dropdown-item') || {}).tabIndex,
    };
  })()`));


  // Custo isolado de cada snapshot de undo (HTML completo + bookmark).
  rec('perf.recordUndo50', await evalJs(`(function(){
    var hist = $('#note').data('summernote').modules.editor.history;
    $('#note').data('summernote').options.historyLimit = 1000;
    hist.commit();
    var t0 = performance.now();
    for (var i = 0; i < 50; i++) hist.recordUndo();
    var t1 = performance.now();
    var bytes = hist.stack.reduce(function(a, sn) { return a + sn.contents.length; }, 0);
    return { perSnapshotMs: +((t1 - t0) / 50).toFixed(2), stackLen: hist.stack.length, totalBytes: bytes };
  })()`));

  // Sequência de Tab a partir do editor (ordem real de foco).
  rec('a11y.tabSequence', await evalJs(`(function(){
    document.querySelector('.note-editable').focus();
    var seen = [];
    return new Promise(function(res){
      var n = 0;
      function step(){
        if (n++ >= 6) return res(seen);
        seen.push(document.activeElement.tagName + '#' + (document.activeElement.id||'') + '.' + (document.activeElement.className||'').slice(0,40));
        document.activeElement.blur();
        res(seen);
      }
      step();
    });
  })()`));

  rec('done', 'ok');
  ws.close();
} catch (e) {
  rec('ERROR', `${e.name}: ${e.message}`);
} finally {
  chrome.kill('SIGKILL');
}
