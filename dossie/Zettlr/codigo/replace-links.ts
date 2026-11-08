/**
 * TRECHO COPIADO PARA ANÁLISE — NÃO REUTILIZAR
 * Origem: ref/Zettlr/source/common/util/replace-links.ts:1-75 (arquivo completo)
 * Projeto: Zettlr 4.9.0 — Copyright (c) Hendrik Erz
 * Licença: GNU GPL v3. Cópia para estudo; nada é reutilizado.
 * Por que: reescrita de links internos quando um arquivo é renomeado
 *          (usada em commands/file-rename.ts:175).
 */

/**
 * @ignore
 * BEGIN HEADER
 *
 * Contains:        Replace Links
 * CVM-Role:        Utility
 * Maintainer:      Hendrik Erz
 * License:         GNU GPL v3
 *
 * Description:     This function can replace a set of links across a file
 *
 * END HEADER
 */

import { extractASTNodes, markdownToAST } from '../modules/markdown-utils'
import { type ZettelkastenLink } from '../modules/markdown-utils/markdown-ast'
import path from 'path'

/**
 * Takes a Markdown document and replaces all occurrences of a link to
 * oldTarget with links to newTarget.
 * 
 * NOTEs:
 *
 * * Do not remove the file extensions, the function will do that for you!
 * * The function will not work with the old/hacky way of linking within a
 *   Markdown link (e.g., `[Do not use this syntax]([[filename]])`)
 *
 * @param   {string}   markdown   The document in question.
 * @param   {string}   oldTarget  The old link target (with filename extension)
 * @param   {string}   newTarget  The new link target (with filename extension)
 *
 * @return  {string}              The new document
 */
export default function replaceLinks (markdown: string, oldTarget: string, newTarget: string): string {
  // Users can link both to `Zettelkasten.md` as well as to `Zettelkasten`.
  // The "withoutExtension" vars will equal the arguments to the function if
  // there is no extension given.
  const oldTargetNoExt = path.basename(oldTarget, path.extname(oldTarget))
  const newTargetNoExt = path.basename(newTarget, path.extname(newTarget))

  // Quick check to avoid expensive computing if the target doesn't even exist
  // in the provided Markdown
  if (!markdown.includes(oldTarget) && !markdown.includes(oldTargetNoExt)) {
    return markdown
  }

  // As a first step, parse the content, and get all existing links.
  const ast = markdownToAST(markdown)
  const links = extractASTNodes(ast, 'ZettelkastenLink') as ZettelkastenLink[]

  const replacements: Array<{ from: number, to: number, text: string }> = []
  for (const link of links) {
    if (link.target !== oldTarget && link.target !== oldTargetNoExt) {
      continue // Link does not lead to the target
    }

    // Relevant link found. We want to replace the old target with the new one.
    replacements.push({
      ...link.targetRange,
      text: link.target === oldTarget ? newTarget : newTargetNoExt
    })
  }

  // Now, apply all replacements in reverse order (because replacements may be
  // not the same length as the old content)
  replacements
    .sort((a, b) => a.from - b.from)
    .reverse()
    .map(replacement => {
      markdown = markdown.slice(0, replacement.from) + replacement.text + markdown.slice(replacement.to)
    })

  return markdown
}
