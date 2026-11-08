/**
 * Exemplo real de serialização: chama o md2html do próprio Zettlr.
 * Usado pelo app no preview de impressão (ref/Zettlr/source/win-print/App.vue:89)
 * e como base do export HTML.
 *
 * Fonte: ref/Zettlr/source/common/modules/markdown-utils/markdown-to-html.ts:406
 * Licença: GPL-3.0 (Hendrik Erz) — análise apenas.
 */
import { md2html } from '@common/modules/markdown-utils'

const md = [
  '# Título',
  '',
  'Ver [[20240101120000 Metodo|o método]] e @doe2020, p. 12.',
  '',
  'Uma nota de rodapé[^1] e `código`.',
  '',
  '[^1]: Texto da nota.',
  ''
].join('\n')

const html = await md2html(md, {
  zknLinkFormat: 'link|title',
  referenceSectionTitle: 'Referências',
  onCitation: (items, composite) => {
    const rendered = items.map(i => `${i.id}${composite ? ' (composto)' : ''}`).join('; ')
    return `<span class="citation">${rendered}</span>`
  }
})

console.log(html)
