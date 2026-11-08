// TRECHO VERBATIM — origem: ref/tiptap/packages/core/src/Extendable.ts:587-668
// Projeto: Tiptap (https://github.com/ueberdosis/tiptap) — licença MIT (ref/tiptap/LICENSE.md)
// Cabeçalho adicionado pelo dossiê; o corpo abaixo não foi alterado.

export class Extendable<
  Options = any,
  Storage = any,
  Config =
    | ExtensionConfig<Options, Storage>
    | NodeConfig<Options, Storage>
    | MarkConfig<Options, Storage>,
> {
  type = 'extendable'
  parent: Extendable | null = null

  child: Extendable | null = null

  name = ''

  config: Config = {
    name: this.name,
  } as Config

  constructor(config: Partial<Config> = {}) {
    this.config = {
      ...this.config,
      ...config,
    }

    this.name = (this.config as any).name
  }

  get options(): Options {
    return {
      ...callOrReturn(
        getExtensionField<AnyConfig['addOptions']>(this as any, 'addOptions', {
          name: this.name,
        }),
      ),
    }
  }

  get storage(): Readonly<Storage> {
    return {
      ...callOrReturn(
        getExtensionField<AnyConfig['addStorage']>(this as any, 'addStorage', {
          name: this.name,
          options: this.options,
        }),
      ),
    }
  }

  configure(options: Partial<Options> = {}) {
    const extension = this.extend<Options, Storage, Config>({
      ...this.config,
      addOptions: () => {
        return mergeDeep(this.options as Record<string, any>, options) as Options
      },
    })

    extension.name = this.name
    extension.parent = this.parent

    this.child = null

    return extension
  }

  extend<
    ExtendedOptions = Options,
    ExtendedStorage = Storage,
    ExtendedConfig =
      | ExtensionConfig<ExtendedOptions, ExtendedStorage>
      | NodeConfig<ExtendedOptions, ExtendedStorage>
      | MarkConfig<ExtendedOptions, ExtendedStorage>,
  >(extendedConfig: Partial<ExtendedConfig> = {}): Extendable<ExtendedOptions, ExtendedStorage> {
    const extension = new (this.constructor as any)({ ...this.config, ...extendedConfig })

    extension.parent = this
    this.child = extension
    extension.name = 'name' in extendedConfig ? extendedConfig.name : extension.parent.name

    return extension
  }
}
