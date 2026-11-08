# `exemplos/` — probe real executado

Os scripts aqui **foram executados**. A saída literal está em `saida-observada.txt`.

## O que foi executado, contra o quê

- Pacotes **publicados no npm**, versão `0.53.0` — **não** o checkout `ref/lexical/HEAD`.
- `node v22.23.2`, Linux x86_64.
- `lexical@0.53.0`, `@lexical/rich-text@0.53.0`, `@lexical/history@0.53.0`,
  `@lexical/html@0.53.0`, `happy-dom` (só para `$generateHtmlFromNodes`).
- **Sem React, sem navegador, sem build do monorepo.**

Divergência npm × HEAD: existe, mas **não** em `registerHistory`. Na revisão, `diff -rq` entre
`node_modules/lexical/src` e `packages/lexical/src` mostra 12 arquivos diferentes e
`LexicalWarning.ts` só no HEAD; a assinatura de `registerHistory` é idêntica
(`(editor, historyState, delay, dateNow?, onHistoryStateChange?, maxDepth?)`,
`ref/lexical/packages/lexical-history/src/index.ts:613-620`). Os exemplos usam
`createEmptyHistoryState()` porque é a API documentada, não por divergência.

## Instalação usada (reproduzível)

```sh
mkdir -p /tmp/lexical-probe && cd /tmp/lexical-probe
printf '{"name":"probe","private":true,"type":"module"}' > package.json
npm i --no-audit --no-fund lexical@0.53.0 @lexical/history@0.53.0 \
      @lexical/rich-text@0.53.0 @lexical/clipboard@0.53.0 \
      @lexical/extension@0.53.0 @lexical/html@0.53.0 happy-dom
```

Saída: instalação concluída (exit 0, ~14 s).

## Como rodar os probes

Os scripts importam pacotes por nome (ESM resolve a partir do diretório do arquivo), então é
preciso rodá-los num diretório que tenha `node_modules`:

```sh
cp dossie/lexical/exemplos/*.mjs /tmp/lexical-probe/
cd /tmp/lexical-probe
node 01-headless-minimo.mjs
node 02-html-json-custom-node.mjs
node 03-custo.mjs
```

## Resultados observados (resumo comentado)

Saída completa em `saida-observada.txt`.

### 01 — o núcleo não sabe editar

```
KEY_ENTER_COMMAND tratado só pelo núcleo? false
filhos do root: [ 'paragraph' ]
=== 2) mesmo editor + registerRichText ===
KEY_ENTER_COMMAND tratado? true
filhos do root: [ 'paragraph', 'paragraph' ]
texto: "Hello world\n\n"
```

Este é o achado mais importante do dossiê: **sem o pacote de modo, Enter não faz nada e nenhum
erro é emitido**. O editor aceita conteúdo programático e ignora a intenção do usuário.

### 01 — negrito quebra o texto em runs

```
nós de texto: [{"detail":0,"format":1,...,"text":"Hello",...},
               {"detail":0,"format":0,...,"text":" world",...}]
```

`format: 1` = `IS_BOLD` (`ref/lexical/packages/lexical/src/LexicalConstants.ts:39`).

### 01 — undo/redo funcionam; `CAN_UNDO_COMMAND` engana

```
antes do undo: abc
depois do undo: a
depois do redo: abc
dispatchCommand(CAN_UNDO_COMMAND) direto -> false (não significa "não há undo")
```

`CAN_UNDO_COMMAND` é **notificação** disparada pelo history; `dispatchCommand` só informa se algum
listener tratou. Para ler o estado real é preciso `registerCommand(CAN_UNDO_COMMAND, …)`.

### 02 — HTML exportado, sem tema não há classe

```
<p class="par"><b><strong class="b" style="white-space: pre-wrap;">bold</strong></b>
<span style="white-space: pre-wrap;"> </span>
<i><em class="i" style="white-space: pre-wrap;">italic</em></i>
<span style="white-space: pre-wrap;"> underline</span></p>
```

Três coisas: (a) `par`/`b`/`i` vêm do `theme` que declarei; sem tema, sai sem classe;
(b) `<b>` envolve `<strong>` — compatibilidade com HTML legado/e-mail;
(c) `style="white-space: pre-wrap"` é **injetado pelo export**, não é meu.

### 02 — ida e volta de JSON

```
=== 3) parseEditorState -> setEditorState ===
texto: "from json"
```

### 02 — feature = nó registrado, não flag

```
=== 4) $generateNodesFromDOM (sem @lexical/link instalado) ===
[..., {"format":0,...,"text":" link",...}]
^ o <a> virou texto puro: sem LinkNode registrado, não há regra de import para <a>
```

O `<a href>` foi **descartado** e sobrou o texto. Prova direta de que suporte a link é
`LinkNode` + regra `importDOM`, não configuração.

### 02 — nó customizado

```
execuções do transform: 2
texto final: "HELLO TRANSFORM"
JSON do nó: {"detail":0,"format":0,"mode":"normal","style":"","text":"HELLO TRANSFORM","type":"loud-text","version":1}
```

Duas execuções do `static transform()` por causa do fixpoint; a segunda converge.

### 03 — custo

```
n | build | toJSON | stringify | jsonBytes
100 | 11.6ms | 1.2ms | 0.1ms | 24279
1000 | 13.7ms | 3.9ms | 1.4ms | 242979
5000 | 49.4ms | 10.4ms | 7.2ms | 1218979
200 updates discretos em 1000 parágrafos: 190.9ms total (0.95ms cada)
```

**Reexecutado na revisão** (mesmo script, mesma máquina, 5 execuções): 1.000 parágrafos deu
13,7–63,6 ms de build, 3,9–14,3 ms de `toJSON`, 1,4–5,6 ms de `stringify`; 200 updates discretos
custaram 0,95–3,50 ms cada (190,9–699,1 ms no total). Ou seja, a dispersão chega a ~5×, não ±20%.
Os probes 01 e 02 reproduziram a saída acima **literalmente**; só os tempos mudam. Use ordem de
grandeza, não benchmark.

## O que NÃO foi executado (e por quê)

| Item | Motivo |
|---|---|
| Build, lint, `tsc` do monorepo | `ref/lexical` não tem `node_modules` (`test -d node_modules` → NO); exigiria `pnpm install` de 39 pacotes + Chromium |
| `vitest` / 100 testes unitários do core | idem |
| Playwright e2e (`Composition.spec.mjs`, `Collaboration.spec.mjs`, `History.spec.mjs`, …) | exige build do playground + browsers |
| `pnpm bench` oficial (`__bench__/`) | exige o monorepo instalado |
| Clipboard real (`DataTransfer`) | happy-dom não reproduz o fluxo de paste/copy do navegador de forma fiel |
| IME / composição | exige navegador real e teclado; avaliado por código + testes do repo |
| Yjs / colaboração | exige `y-websocket` e dois clientes |
| Exemplo `examples/vanilla-js` com Vite | exige instalar as deps do exemplo e um browser; o código foi lido, não rodado |

## O que estes probes **não** provam

- Nada sobre a correção do DOM reconciliado em navegador real (não houve browser).
- Nada sobre acessibilidade observada com leitor de tela.
- Nada sobre o comportamento do HEAD do checkout: o npm pode estar atrás (e está, em pelo menos
  `registerHistory`).
