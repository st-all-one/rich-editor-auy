/**
 * Exemplo real: usa os módulos de fato do Zettlr (via tsconfig `paths`) para
 * reproduzir o índice de links do LinkProvider.
 *
 * Fonte usada:
 *   - ref/Zettlr/source/app/service-providers/fsal/util/extract-file-id.ts
 *   - ref/Zettlr/source/common/util/extract-from-file-descriptors.ts
 *   - lógica de ref/Zettlr/source/app/service-providers/links/index.ts:131-178
 *     (transcrita aqui porque a classe depende de `electron` e de IPC)
 *
 * Licença do código original: GPL-3.0 (Hendrik Erz). Este exemplo é apenas
 * análise/estudo; nada é incorporado ao rich-editor-auy.
 */
import path from 'path'
import extractFileId from '@providers/fsal/util/extract-file-id'
import { extractFromFileDescriptors } from '@common/util/extract-from-file-descriptors'

const ID_PATTERN = '(\\d{14})' // default de zkn.idRE (get-config-template.ts:374)

interface MdDescriptor {
  type: 'file'
  path: string
  name: string
  ext: string
  id: string
  links: string[]
  tags: string[]
  citekeys: string[]
}

function makeFile (absPath: string, markdown: string, links: string[]): MdDescriptor {
  const name = path.basename(absPath)
  return {
    type: 'file',
    path: absPath,
    name,
    ext: path.extname(absPath),
    id: extractFileId(name, markdown, ID_PATTERN),
    links,
    tags: [],
    citekeys: []
  }
}

// --- "workspace" simulado -------------------------------------------------
const files: MdDescriptor[] = [
  makeFile('/notes/20240101120000 Metodo.md', '# Metodo\n', []),
  makeFile('/notes/20240102090000 Revisao.md', '# Revisao\n', ['20240101120000', 'Metodo', 'Inexistente']),
  makeFile('/notes/Sem ID.md', '# Sem id\n', ['20240101120000']),
  // Duplicata de basename em outro diretório: os índices são Maps por caminho,
  // mas findExact retorna o PRIMEIRO descriptor que casa (links/index.ts:102-124).
  makeFile('/notes/arquivo/Revisao.md', '# Outra revisao\n', [])
]

// --- reindex(): links/index.ts:80-86 --------------------------------------
const fileLinkDatabase = new Map(extractFromFileDescriptors(files as any, 'links'))
const idDatabase = new Map(extractFromFileDescriptors(files as any, 'id'))

// --- findExact(): links/index.ts:102-124 ---------------------------------
function findExact (query: string): MdDescriptor | undefined {
  const isQueryID = new RegExp(`^${ID_PATTERN}$`).test(query)
  const hasMdExt = /\.(md|markdown)$/i.test(query)
  for (const d of files) {
    if (isQueryID && d.id === query) return d
    if (hasMdExt && d.name === query) return d
    if (d.name === query + d.ext) return d
  }
}

// --- retrieveInbound(): links/index.ts:131-158 ---------------------------
function retrieveInbound (sourceFilePath: string): string[] {
  const id = idDatabase.get(sourceFilePath)
  if (id === undefined) return []
  const linkWithExt = path.basename(sourceFilePath)
  const linkWoExt = path.basename(sourceFilePath, path.extname(sourceFilePath))
  const out: string[] = []
  for (const [ file, outbound ] of fileLinkDatabase.entries()) {
    if (outbound.includes(id) || outbound.includes(linkWithExt) || outbound.includes(linkWoExt)) {
      out.push(file)
    }
  }
  return out
}

// --- retrieveOutbound(): links/index.ts:160-178 --------------------------
function retrieveOutbound (sourceFilePath: string): string[] {
  const dbLinks = fileLinkDatabase.get(sourceFilePath)
  if (dbLinks === undefined) return []
  const out: string[] = []
  for (const link of dbLinks) {
    const d = findExact(link)
    if (d !== undefined) out.push(d.path)
  }
  return out
}

const target = '/notes/20240101120000 Metodo.md'
console.log('ID extraído do nome:', JSON.stringify(files[0].id))
console.log('ID de arquivo sem ID:', JSON.stringify(files[2].id))
console.log('inbound(Metodo):', retrieveInbound(target))
console.log('outbound(Revisao):', retrieveOutbound('/notes/20240102090000 Revisao.md'))
console.log('outbound(Inexistente) por link quebrado:', retrieveOutbound('/notes/Nao existe.md'))
console.log('findExact("Metodo") [nome parcial]:', findExact('Metodo')?.name ?? null)
console.log('findExact("20240101120000 Metodo"):', findExact('20240101120000 Metodo')?.path ?? null)
console.log('findExact("Metodo.md") [com extensão]:', findExact('Metodo.md')?.path ?? null)
console.log('findExact("Revisao") [basename duplicado]:', findExact('Revisao')?.path ?? null)
console.log('findExact("Inexistente"):', findExact('Inexistente') ?? null)
