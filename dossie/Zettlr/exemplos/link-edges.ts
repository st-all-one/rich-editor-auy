/**
 * Exemplo real: casos de borda do parser de wikilinks do Zettlr.
 * Usa o parser e o extrator de AST do próprio app (via tsconfig `paths`).
 *
 * Fonte usada:
 *   - ref/Zettlr/source/common/modules/markdown-utils (markdownToAST, extractASTNodes)
 *
 * Licença do código original: GPL-3.0 (Hendrik Erz). Este exemplo é apenas
 * análise/estudo; nada é incorporado ao rich-editor-auy.
 */
import { markdownToAST, extractASTNodes } from '@common/modules/markdown-utils'

/** Forma mínima que este exemplo lê de um nó ZettelkastenLink. */
interface ZknLinkView {
  target?: string
  title?: { value?: string }
}

function isZknLink (node: unknown): node is ZknLinkView {
  return typeof node === 'object' && node !== null && 'target' in node
}

const inputs = [
  '[[alvo|título]]',
  '[[alvo|]]',
  '[[|título]]',
  '[[Metodo#Intro]]',
  '[[Metodo#Intro|texto]]'
]

for (const md of inputs) {
  const first = extractASTNodes(markdownToAST(md), 'ZettelkastenLink')[0]
  const link = isZknLink(first) ? first : undefined
  console.log(
    JSON.stringify(md),
    '-> target =', JSON.stringify(link?.target),
    '| title =', JSON.stringify(link?.title?.value)
  )
}
