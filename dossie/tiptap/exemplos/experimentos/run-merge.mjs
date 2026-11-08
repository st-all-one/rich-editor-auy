// Executa a função REAL mergeAttributes do Tiptap (cópia verbatim em ./mergeAttributes.ts).
// Sem DOM, sem dependências: só a utilidade de merge de atributos usada por todos os
// renderHTML()/addAttributes() das extensões.
import { mergeAttributes } from './mergeAttributes.ts'

const base = { class: 'prose p-4', style: 'color: red', 'data-id': 'x' }
const fromExtension = { class: 'prose custom', style: 'color: blue; margin: 0' }

const merged = mergeAttributes(base, fromExtension)
console.log('merged =', JSON.stringify(merged))

const protoAttack = JSON.parse('{"__proto__": {"polluted": true}}')
const hardened = mergeAttributes({}, protoAttack)
console.log('obj.polluted =', ({}).polluted)
console.log('own keys     =', Object.keys(hardened))
console.log('hasOwn __proto__ =', Object.hasOwn(hardened, '__proto__'))
