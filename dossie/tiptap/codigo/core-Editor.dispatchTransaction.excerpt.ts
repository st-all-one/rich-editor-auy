// TRECHO VERBATIM — origem: ref/tiptap/packages/core/src/Editor.ts:717-809
// Projeto: Tiptap (https://github.com/ueberdosis/tiptap) — licença MIT (ref/tiptap/LICENSE.md)
// Cabeçalho adicionado pelo dossiê; o corpo abaixo não foi alterado.

  /**
   * The callback over which to send transactions (state updates) produced by the view.
   *
   * @param transaction An editor state transaction
   */
  private dispatchTransaction(transaction: Transaction): void {
    if (this.isDestroyed) {
      return
    }

    if (this.isCapturingTransaction) {
      if (!this.capturedTransaction) {
        this.capturedTransaction = transaction

        return
      }

      transaction.steps.forEach(step => this.capturedTransaction?.step(step))

      return
    }

    // Apply transaction and get resulting state and transactions
    const { state, transactions } = this.state.applyTransaction(transaction)
    const selectionHasChanged = !this.state.selection.eq(state.selection)
    const rootTrWasApplied = transactions.includes(transaction)
    const prevState = this.state

    this.emit('beforeTransaction', {
      editor: this,
      transaction,
      nextState: state,
    })

    // If transaction was filtered out, we can return early
    if (!rootTrWasApplied) {
      return
    }

    this.view.updateState(state)

    // Emit transaction event with appended transactions info
    this.emit('transaction', {
      editor: this,
      transaction,
      appendedTransactions: transactions.slice(1),
    })

    if (selectionHasChanged) {
      this.emit('selectionUpdate', {
        editor: this,
        transaction,
      })
    }

    // Only emit the latest between focus and blur events
    const mostRecentFocusTr = transactions.findLast(tr => tr.getMeta('focus') || tr.getMeta('blur'))
    const focus = mostRecentFocusTr?.getMeta('focus')
    const blur = mostRecentFocusTr?.getMeta('blur')

    if (focus) {
      this.emit('focus', {
        editor: this,
        event: focus.event,
        // oxlint-disable-next-lineno-non-null-assertion
        transaction: mostRecentFocusTr!,
      })
    }

    if (blur) {
      this.emit('blur', {
        editor: this,
        event: blur.event,
        // oxlint-disable-next-lineno-non-null-assertion
        transaction: mostRecentFocusTr!,
      })
    }

    // Compare states for update event
    if (
      transaction.getMeta('preventUpdate') ||
      !transactions.some(tr => tr.docChanged) ||
      prevState.doc.eq(state.doc)
    ) {
      return
    }

    this.emit('update', {
      editor: this,
      transaction,
      appendedTransactions: transactions.slice(1),
    })
  }
