/**
 * TRECHO COPIADO PARA ANÁLISE — NÃO REUTILIZAR
 * Origem: ref/Zettlr/source/app/service-providers/links/index.ts:96-178
 * Projeto: Zettlr 4.9.0 — Copyright (c) Hendrik Erz
 * Licença: GNU GPL v3 (ref/Zettlr/LICENSE). Cópia parcial para estudo;
 *          o rich-editor-auy não incorpora este código.
 * Por que: é o coração da resolução de referências entre arquivos
 *          (inbound/outbound e busca exata por ID ou basename).
 */

  /**
   * Finds the descriptor for a file query based on a ZKN id, file name, or file
   * base path. This logic is copied from the FSAL `findExact` method to avoid
   * the performance impacts of calling `getAllLoadedDescriptors` on every
   * execution.
   */
  private async findExact (query: string, descriptors: MDFileDescriptor[]): Promise<MDFileDescriptor|undefined> {
    const { zkn } = this._config.get()
    const idRe = getIDRE(zkn.idRE, true)

    const isQueryID = idRe.test(query)
    const hasMdExt = hasMarkdownExt(query)

    for (const descriptor of descriptors) {
      if (isQueryID && descriptor.id === query) {
        return descriptor
      }

      if (hasMdExt && descriptor.name === query) {
        return descriptor
      }

      if (descriptor.name === query + descriptor.ext) {
        return descriptor
      }
    }
  }

  /**
   * Retrieves a set of links to the file given as argument
   *
   * @param   {string}    sourceFilePath  The source file's path
   *
   * @return  {string[]}                  A list of all files linking to sourceFile
   */
  retrieveInbound (sourceFilePath: string): string[] {
    const id = this._idDatabase.get(sourceFilePath)

    if (id === undefined) {
      return [] // Not part of the ID map
    }

    const sourceFiles: string[] = []

    const linkWithExt = path.basename(sourceFilePath)
    const linkWoExt = path.basename(sourceFilePath, path.extname(sourceFilePath))

    // Search all recorded links
    for (const [ file, outbound ] of this._fileLinkDatabase.entries()) {
      if (outbound.includes(id) || outbound.includes(linkWithExt) || outbound.includes(linkWoExt)) {
        sourceFiles.push(file)
      }
    }

    return sourceFiles
  }

  /**
   * Retrieves a set of files the given source file links to
   *
   * @param   {string}    sourceFilePath  The source file's path
   *
   * @return  {string[]}                  A list of outbound links from source
   */
  async retrieveOutbound (sourceFilePath: string): Promise<string[]> {
    const dbLinks = this._fileLinkDatabase.get(sourceFilePath)
    if (dbLinks === undefined) {
      return []
    }

    const loadedDescriptors = (await this._fsal.getAllLoadedDescriptors())
      .filter(descriptor => descriptor.type === 'file')

    const outboundLinks: string[] = []
    for (const link of dbLinks) {
      const descriptor = await this.findExact(link, loadedDescriptors)

      if (descriptor !== undefined) {
        outboundLinks.push(descriptor.path)
      }
    }

    return outboundLinks
