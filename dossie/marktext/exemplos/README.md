# Exemplos — MarkText / Muya

Este diretório contém **um** exemplo: `minimal.ts` + `index.html`, escrito a partir do "Quick
start" oficial (`ref/marktext/packages/muya/README.md:31-95`) e da lista canônica de plugins do
demo do repositório (`ref/marktext/packages/muya/examples/src/main.ts:60-81`).

O arquivo `minimal.ts` é de nossa autoria (não é cópia); as APIs que ele usa são reais e estão
citadas por linha dentro dos comentários. O projeto de origem é MIT.

---

## 1. Status de execução

**NÃO EXECUTADO.** Motivo concreto:

```
$ cd ref/marktext && ls -d node_modules
ls: cannot access 'node_modules': No such file or directory
$ ls -d packages/muya/node_modules
ls: cannot access 'packages/muya/node_modules': No such file or directory
```

Sem `node_modules/` não há `vite`, não há resolução de `@muyajs/core` e não há CSS processado.
Instalar exigiria `pnpm install` no monorepo inteiro (Electron 45, 28 dependências de runtime da
engine, rebuild de módulos nativos), o que está fora do escopo permitido. Portanto: nenhuma saída
é apresentada como observada.

A rodada de revisão reexecutou só os comandos baratos: `ls -d node_modules` e
`ls -d packages/muya/node_modules` continuam falhando (exit 2) e `pnpm-lock.yaml` existe, então
`test:spec` e Playwright **também não foram executados**. `minimal.ts` é código completo (173
linhas, sem placeholders), mas não compilado nem carregado neste ambiente.

---

## 2. Como rodar (duas rotas)

### Rota A — dentro do próprio monorepo de referência (recomendada)

O repositório já tem um workspace de demo; é o caminho com menos atrito e o que o próprio projeto
mantém:

```sh
cd ref/marktext
pnpm install
pnpm --filter muya-examples dev:demo
```

Fonte: `ref/marktext/packages/muya/README.md:236` e
`ref/marktext/packages/muya/examples/package.json:9-11` (`"dev:demo": "vite"`).
O demo real fica em `packages/muya/examples/src/main.ts` e serve como referência de integração
completa (locale, opções, rebuild, eventos).

### Rota B — projeto Vite isolado com este exemplo

```sh
mkdir muya-demo && cd muya-demo
npm create vite@latest . -- --template vanilla-ts
npm install
npm install @muyajs/core
cp <este diretório>/minimal.ts src/main.ts
cp <este diretório>/index.html .
npm run dev
```

Observações que valem como evidência de leitura de código:

- `new Muya(el, opts)` **substitui** o elemento passado por um novo `div` contenteditable
  (`ref/marktext/packages/muya/src/muya.ts:1618-1646`). Depois do init, guarde `muya.domNode`;
  a variável `host` fica órfã.
- Nada é renderizado antes de `muya.init()` (`src/muya.ts:246-253`).
- `Muya.use(Plugin)` é **global na classe** e deduplica por `pluginName`
  (`src/muya.ts:142-152`): chamar duas vezes não duplica a toolbar.
- `import '@muyajs/core/lib/style.css'` só existe no pacote **buildado** (o `publishConfig` aponta
  para `lib/`). Consumindo o workspace por `exports: "./src/*"`, o CSS já é importado por
  `src/muya.ts:43-48`.
- `flush()` é obrigatório antes de ler para persistir
  (`src/editor/transactions/index.ts:167-190`); sem ele, a última edição pode estar pendente.

### O que o exemplo demonstra

| Recurso | Onde no exemplo | API real |
|---|---|---|
| Registro de plugins de UI | bloco "1. Plugins" | `Muya.use` — `src/muya.ts:142-152` |
| Criação + init | bloco "3. Criação" | `new Muya` — `src/muya.ts:202-210`; `init` — `:246` |
| Leitura de Markdown | botão `getMarkdown()` | `src/muya.ts:284-286` |
| Leitura do estado bruto | botão `getState()` | devolve clone — `src/state/index.ts:135-136` |
| Undo/redo | botões | `undo()` — `src/muya.ts:308`; `redo()` — `:314` |
| Troca de documento desfazível | `replaceDocument()` | `replaceContent` — `src/muya.ts:380-395` |
| Snapshot de seleção | bloco "6. Seleção" | `getSelectionState`/`setSelection` — `src/muya.ts:651-657` |
| Evento de mudança | `json-change` | emitido em `src/editor/index.ts:587` (dentro de `_commitTransaction`, `:572-616`) |
| Desmontagem | `muya.destroy()` no `beforeunload` | libera listeners e floats |

---

## 3. Exemplo de uso interno (como o app consome a engine)

Se a rota standalone não rodar, o caminho de integração real e verificado por leitura é o do
desktop:

- **Registro de plugins e opções**: `ref/marktext/packages/desktop/src/renderer/src/components/editorWithTabs/editor.vue:1693-1779`
- **Instanciação**: `const muya = markRaw(new Muya(ele, options))` seguido de
  `muya.init({ focus })` — `editor.vue:1792-1794`
- **Sincronização de conteúdo**: `editor.value.on('json-change', …)` → `getMarkdown()` →
  `editorStore.LISTEN_FOR_CONTENT_CHANGE` — `editor.vue:1907-1926`
- **Gravação em disco**: `writeMarkdownFile` com conversão de line ending e erro de encoding —
  `packages/desktop/src/main/filesystem/markdown.ts:134-163`

Esse é o ciclo completo que qualquer host precisa reproduzir:

```
markdown ──setContent──▶ engine ──edição──▶ json-change ──getMarkdown──▶ store ──IPC──▶ disco
```

---

## 4. O que não foi possível verificar

| Item | Como verificar |
|---|---|
| Que o exemplo compila e renderiza | Rota A ou B acima |
| Que os plugins aparecem e funcionam | Abrir a página e passar o mouse numa seleção (toolbar), digitar `/` (menu) |
| Que `getMarkdown()` preserva formatação | Digitar Markdown, clicar em `getMarkdown()` e comparar |
| Que IME funciona com entrada real | Testar com pinyin/japonês em Chromium e Firefox |
| Que o CSS da engine é suficiente | Rodar a Rota B com o import de `lib/style.css` após build |

Nenhuma dessas verificações foi feita. O dossiê marca isso explicitamente em
`avaliacao.md` §3.
