/**
 * Exemplo real: renomeação com reescrita de links internos.
 * Usa o util do app (ref/Zettlr/source/common/util/replace-links.ts:35), que é
 * chamado em ref/Zettlr/source/app/service-providers/commands/file-rename.ts:175.
 *
 * Licença: GPL-3.0 (Hendrik Erz) — análise apenas.
 */
import replaceLinks from '@common/util/replace-links'

const doc = `Antes: [[Metodo]], [[Metodo.md]] e [[Metodo|título livre]].
Isto NÃO deve mudar: [texto]([[Metodo]]) e \`[[Metodo]]\` em código inline.
`

const out = replaceLinks(doc, 'Metodo.md', 'Metodo (Luhmann).md')
console.log(out)
console.log('--- casos que o util NÃO cobre ---')
console.log('link dentro de markdown:', replaceLinks('[x]([[Metodo]])', 'Metodo.md', 'Novo.md'))
console.log('link com título:', replaceLinks('[[Metodo|T]]', 'Metodo.md', 'Novo.md'))
console.log('link com ID:', replaceLinks('[[20240101120000]]', 'Metodo.md', 'Novo.md'))
