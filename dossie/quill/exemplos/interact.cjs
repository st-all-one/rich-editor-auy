// Executa interações reais (teclado, seleção, clipboard, IME) contra o build publicado 2.0.3.
const { chromium } = require('playwright-core');

const EXE = '/home/one/.cache/ms-playwright/chromium-1248/chrome-linux64/chrome';
const URL = process.env.URL || 'http://127.0.0.1:8765/interactive.html';

(async () => {
  const browser = await chromium.launch({ executablePath: EXE, headless: true });
  const context = await browser.newContext({ permissions: ['clipboard-read', 'clipboard-write'] });
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e)));
  await page.goto(URL);
  await page.waitForFunction(() => window.quill && window.Quill);
  const out = {};
  const state = () => page.evaluate(() => window.getState());
  const reset = async () => {
    await page.evaluate(() => {
      window.quill.setContents([{ insert: '\n' }], 'api');
      window.quill.history.clear();
      window.quill.focus();
      window.quill.setSelection(0, 0, 'user');
    });
    await page.waitForTimeout(30);
  };

  try {
    out.selectionApiWorks = await page.evaluate(() => {
      const r = window.quill.getSelection();
      return r && { index: r.index, length: r.length };
    });

    // digitação real: substitui o parágrafo inicial
    await page.evaluate(() => {
      window.quill.setContents([{ insert: 'initial\n' }], 'api');
      window.quill.history.clear();
      window.quill.focus();
      window.quill.setSelection(0, 7, 'user');
    });
    const t0 = Date.now();
    await page.keyboard.type('Olá mundo');
    const tType = Date.now() - t0;
    out.typing = { ms: tType, ...(await state()) };

    // Enter cria novo parágrafo; formatos de bloco são herdados (handleEnter)
    await page.keyboard.press('Enter');
    await page.keyboard.type('segundo');
    out.enter = await state();

    // seleção com teclado + negrito
    for (let i = 0; i < 7; i += 1) await page.keyboard.press('Shift+ArrowLeft');
    await page.keyboard.press('Control+b');
    out.boldShortcut = await state();

    // undo/redo via teclado
    await page.keyboard.press('Control+z');
    out.undo = await state();
    await page.keyboard.press('Control+Shift+z');
    out.redo = await state();

    // autofill de lista: "- item" + espaço
    await reset();
    await page.keyboard.type('- item');
    await page.keyboard.press('Enter');
    await page.keyboard.type('outro');
    out.listAutofill = await state();

    // Tab indenta item de lista
    await page.keyboard.press('Tab');
    out.listIndent = await state();

    // colar texto simples via clipboard real
    await reset();
    await page.evaluate(() => navigator.clipboard.writeText('colado do clipboard'));
    await page.keyboard.press('Control+v');
    await new Promise((r) => setTimeout(r, 100));
    out.pastePlain = await state();

    // colar HTML externo (Word-like) via evento paste
    await reset();
    await page.evaluate(() => {
      const dt = new DataTransfer();
      dt.setData('text/html', '<p><b>Bold</b> and <i>it</i></p><ul><li>a</li><li>b</li></ul>');
      dt.setData('text/plain', 'Bold and it\na\nb');
      document.querySelector('.ql-editor').dispatchEvent(new ClipboardEvent('paste', { clipboardData: dt, bubbles: true, cancelable: true }));
    });
    await new Promise((r) => setTimeout(r, 150));
    out.pasteHTML = await state();

    // IME: composição via CDP
    await reset();
    const cdp = await context.newCDPSession(page);
    await page.evaluate(() => window.quill.setSelection(7, 0, 'user'));
    await cdp.send('Input.imeSetComposition', { text: 'に', selectionStart: 1, selectionEnd: 1 });
    await cdp.send('Input.imeSetComposition', { text: 'にほ', selectionStart: 2, selectionEnd: 2 });
    await cdp.send('Input.imeSetComposition', { text: 'にほん', selectionStart: 3, selectionEnd: 3 });
    out.imeDuringComposition = await state();
    await cdp.send('Input.insertText', { text: '日本' });
    await new Promise((r) => setTimeout(r, 100));
    out.imeCommitted = await state();

    // readOnly: digitação do usuário deve ser bloqueada; API continua
    await reset();
    await page.evaluate(() => window.quill.disable());
    await page.keyboard.type('XYZ');
    out.readOnlyTyping = await state();
    out.readOnlyApiStillWorks = await page.evaluate(() =>
      window.quill.insertText(0, 'A', 'api').ops.length);

    // toolbar clicando com o mouse após selecionar texto
    await page.evaluate(() => {
      window.quill.enable();
      window.quill.setContents([{ insert: 'clique aqui\n' }], 'api');
      window.quill.setSelection(0, 5, 'user');
    });
    await page.click('button.ql-bold');
    out.toolbarClick = await state();

    // desempenho de digitação: 120 caracteres
    await reset();
    await page.evaluate(() => window.quill.setSelection(7, 0, 'user'));
    const t1 = Date.now();
    await page.keyboard.type('a'.repeat(120), { delay: 0 });
    out.typing120 = { ms: Date.now() - t1, length: await page.evaluate(() => window.quill.getLength()) };

    out.pageErrors = errors;
    out.ok = true;
  } catch (e) {
    out.error = String((e && e.stack) || e);
    out.pageErrors = errors;
  }
  console.log(JSON.stringify(out, null, 1));
  await browser.close();
})();
