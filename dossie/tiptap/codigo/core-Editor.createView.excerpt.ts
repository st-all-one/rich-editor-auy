// TRECHO VERBATIM — origem: ref/tiptap/packages/core/src/Editor.ts:633-715
// Projeto: Tiptap (https://github.com/ueberdosis/tiptap) — licença MIT (ref/tiptap/LICENSE.md)
// Cabeçalho adicionado pelo dossiê; o corpo abaixo não foi alterado.

  private createView(element: NonNullable<EditorOptions['element']>): void {
    const { editorProps, enableExtensionDispatchTransaction } = this.options
    // If a user provided a custom `dispatchTransaction` through `editorProps`,
    // we use that as the base dispatch function.
    // Otherwise, we use Tiptap's internal `dispatchTransaction` method.
    const baseDispatch =
      (editorProps as DirectEditorProps).dispatchTransaction || this.dispatchTransaction.bind(this)
    const dispatch = enableExtensionDispatchTransaction
      ? this.extensionManager.dispatchTransaction(baseDispatch)
      : baseDispatch

    // Compose transformPastedHTML from extensions and user-provided editorProps
    const baseTransformPastedHTML = (editorProps as DirectEditorProps).transformPastedHTML
    const transformPastedHTML = this.extensionManager.transformPastedHTML(baseTransformPastedHTML)

    this.editorView = new EditorView(element, {
      ...editorProps,
      attributes: this.getEditorViewAttributes(),
      dispatchTransaction: dispatch,
      transformPastedHTML,
      state: this.editorState.reconfigure({ plugins: [] }),
    })

    const state = this.hasMounted
      ? this.editorState
      : this.editorState.reconfigure({
          plugins: [...this.extensionManager.plugins, ...this.editorState.plugins],
        })

    this.hasMounted = true
    this.view.setProps({
      state,
      markViews: this.extensionManager.markViews,
      nodeViews: this.extensionManager.nodeViews,
    })

    this.prependClass()
    this.injectCSS()

    // Let’s store the editor instance in the DOM element.
    // So we’ll have access to it for tests.
    // @ts-ignore
    const dom = this.view.dom as TiptapEditorHTMLElement

    dom.editor = this
  }

  /**
   * Creates all node and mark views.
   */
  public createNodeViews(): void {
    if (!this.isMounted) {
      return
    }

    this.view.setProps({
      markViews: this.extensionManager.markViews,
      nodeViews: this.extensionManager.nodeViews,
    })
  }

  /**
   * Prepend class name to element.
   */
  public prependClass(): void {
    this.view.dom.className = `${this.className} ${this.view.dom.className}`
  }

  public isCapturingTransaction = false

  private capturedTransaction: Transaction | null = null

  public captureTransaction(fn: () => void) {
    this.isCapturingTransaction = true
    fn()
    this.isCapturingTransaction = false

    const tr = this.capturedTransaction

    this.capturedTransaction = null

    return tr
  }
