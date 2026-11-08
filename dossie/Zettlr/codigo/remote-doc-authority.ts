/**
 * TRECHO COPIADO PARA ANÁLISE — NÃO REUTILIZAR
 * Origem: ref/Zettlr/source/common/modules/markdown-editor/plugins/remote-doc.ts:54-183
 * Projeto: Zettlr 4.9.0 — Copyright (c) Hendrik Erz
 * Licença: GNU GPL v3. Cópia para estudo; nada é reutilizado.
 * Por que: sincronização editor <-> autoridade de documento via
 *          @codemirror/collab (versões, pull/push, reload forçado).
 */

export function hookDocumentAuthority (
  filePath: string,
  startVersion: number,
  pullUpdates: PullUpdateCallback,
  pushUpdates: PushUpdateCallback
): Extension {
  const plugin = ViewPlugin.fromClass(class {
    private isCurrentlyPushing: boolean
    private pluginDestroyed: boolean

    constructor (private readonly view: EditorView) {
      this.isCurrentlyPushing = false
      this.pluginDestroyed = false

      // Immediately enter a loop to pull updates to the document. The pull
      // method will create a Promise that links this plugin (and, by extension,
      // the editor state) over the IPC or websocket bridge until there are any
      // updates available.
      this.pull().catch(err => { console.error(`Pulling updates failed: ${String(err.message)}`, err) })
    }

    update (update: ViewUpdate): void {
      // Whenever the doc changed, sync those changes with the document authority
      if (update.docChanged) {
        this.push().catch(err => { console.error(`Pushing updates failed: ${String(err.message)}`, err) })
      }
    }

    async push (): Promise<void> {
      if (this.isCurrentlyPushing) {
        return // There's another push going on atm
      }

      const updates = sendableUpdates(this.view.state)
      if (updates.length === 0) {
        return // No updates available
      }

      this.isCurrentlyPushing = true
      const version = getSyncedVersion(this.view.state)
      const serializedUpdates: Update[] = updates.map(u => {
        return { clientID: u.clientID, changes: u.changes.toJSON() }
      })

      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const success = await pushUpdates(filePath, version, serializedUpdates)

      // Allow another push, if new updates have amassed during the push
      this.isCurrentlyPushing = false
      setTimeout(() => {
        this.push().catch(err => { console.error(`Pushing updates failed: ${String(err.message)}`, err) })
      }, 100)
    }

    async pull (): Promise<void> {
      if (this.pluginDestroyed) {
        return // Do not attempt a new pull
      }

      const version = getSyncedVersion(this.view.state)
      const updates = await pullUpdates(filePath, version)
      // NOTE: At this point, hours may have passed, so we have to re-check the
      // current state, because the view will remain valid for as long as the
      // editor leaf exists. *However* if the editor now shows a different file,
      // the state will have a different remote-doc plugin that already takes
      // care of communicating with the API, so we have to remove this "dangling
      // promise".
      const currentFilePath = this.view.state.field(configField).metadata.path
      if (this.pluginDestroyed || filePath !== currentFilePath) {
        return // These updates shall not be applied to the current state
      }

      if (updates === false) {
        // By returning `false`, the authority told us that we've lost
        // synchronization and there is no way to re-synchronize in this method.
        // This means that here we need to emit an effect that must be captured
        // by the MainEditor instance to perform a full reload of the document
        // state. After that, we break out of the pull loop and let the new
        // plugin instance take over.
        this.view.dispatch({ effects: reloadStateEffect.of(true) })
        return
      }

      try {
        // Deserialize & apply updates
        const deserializedUpdates = updates.map(u => {
          return {
            clientID: u.clientID,
            changes: ChangeSet.fromJSON(u.changes)
          }
        })
        const transaction = receiveUpdates(this.view.state, deserializedUpdates)
        this.view.dispatch(transaction)
      } catch (err: any) {
        console.error(`Pulling updates for failed (retrying): ${String(err.message)}`, err)
      }

      // Whether there was an error or not, schedule another pull
      this.pull().catch(err => { console.error(`Pulling updates failed: ${String(err.message)}`, err) })
    }

    destroy (): void {
      // This ensures that the client stops trying to receive updates after the
      // editor has been removed (hence, no dangling promises that will prevent
      // a smooth shutdown of the application)
      this.pluginDestroyed = true
    }
  })

  return [ collab({ startVersion }), plugin ]
}
