# Exemplos e comandos observados

Tudo aqui foi rodado em **Node v22.23.2**, fora do repositório (`/tmp/pm-run`), para não instalar
nada no projeto. Os scripts são cópias verbatim dos que produziram as saídas abaixo.

## 0. De onde veio o código analisado

`ref/prosemirror/` é só o meta-repositório (`README.md:1`, `bin/pm.js:116` — `install` clona os
módulos irmãos). Nenhum `src/` existe ali. A análise usou os **tarballs oficiais do npm**,
extraídos em `/tmp/pm-upstream/` pela coordenação e verificados aqui:

```bash
sha256sum /tmp/pm-upstream/*.tgz
# 0ac7e5de…  prosemirror-model-1.25.12.tgz
# 0b9b224a…  prosemirror-transform-1.12.5.tgz
# ad3eb60b…  prosemirror-state-1.4.4.tgz
# d92a5d70…  prosemirror-view-1.42.9.tgz
# ba3f0ac2…  prosemirror-history-1.5.1.tgz
# 2dcfc635…  prosemirror-collab-1.3.1.tgz
# 0b2479d8…  prosemirror-schema-basic-1.2.5.tgz
```

Reprodução (qualquer máquina com rede):

```bash
mkdir -p /tmp/pm-upstream && cd /tmp/pm-upstream
for p in model@1.25.12 transform@1.12.5 state@1.4.4 view@1.42.9 \
         history@1.5.1 collab@1.3.1 schema-basic@1.2.5; do
  name="prosemirror-${p%@*}"; ver="${p#*@}"
  curl -fsSLO "https://registry.npmjs.org/prosemirror-${p%@*}/-/${name}-${ver}.tgz"
  tar xzf "${name}-${ver}.tgz"
done
```

Para os módulos usados só como contraste (commands, keymap, inputrules, example-setup, e a suíte de
testes da view), clone raso do espelho oficial:

```bash
cd /tmp/pm-src
for p in commands keymap inputrules example-setup history collab view model state transform; do
  git clone --depth 1 -q https://code.haverbeke.berlin/prosemirror/prosemirror-$p
done
```

## 1. Instalação mínima (executado)

```bash
mkdir -p /tmp/pm-run && cd /tmp/pm-run
npm init -y
npm install --no-audit --no-fund \
  prosemirror-model@1.25.12 prosemirror-transform@1.12.5 prosemirror-state@1.4.4 \
  prosemirror-history@1.5.1 prosemirror-schema-basic@1.2.5 prosemirror-commands@1.7.2
```

Saída relevante:

```
$ npm ls --depth=0
pm-run@1.0.0 /tmp/pm-run
├── prosemirror-commands@1.7.2
├── prosemirror-history@1.5.1
├── prosemirror-model@1.25.12
├── prosemirror-schema-basic@1.2.5
├── prosemirror-state@1.4.4
└── prosemirror-transform@1.12.5

$ du -sh node_modules
2,5M	node_modules

$ ls node_modules
orderedmap prosemirror-commands prosemirror-history prosemirror-model
prosemirror-schema-basic prosemirror-state prosemirror-transform prosemirror-view rope-sequence
```

Observação: `prosemirror-view` e `orderedmap`/`rope-sequence` entram como dependências transitivas —
a view vem de `prosemirror-state` (que a declara em `dependencies`, embora só importe tipos), e
`rope-sequence` vem de `prosemirror-history`. O editor mínimo **sem DOM** não é o caso de uso
pensado, mas funciona para model/transform/state (a view chega a ser importável em Node; só
instanciá-la exige `document`).

## 2. `demo.js` — modelo, posições, passos, undo (executado)

```bash
node demo.js
```

Saída integral (reproduzida duas vezes, idêntica):

```
doc.toJSON        = {"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello world"}]}]}
doc.nodeSize      = 15 | paragraph.nodeSize = 13 | text.nodeSize = 11
resolve(6)        = {"pos":6,"depth":1,"parent":"paragraph","parentOffset":5,"start":1,"end":12,"textOffset":5,"index":0,"nodeAfter":" world"}
resolve(12).textOffset (end token) = 0
conteúdo inválido -> RangeError: Invalid content for node paragraph: <paragraph>
createAndFill doc = {"type":"doc","content":[{"type":"paragraph"}]}
steps             = [{"stepType":"replace","from":1,"to":6,"slice":{"content":[{"type":"text","text":"Howdy"}]}}]
texto após step   = "Howdy world"
mapping.map(6, +1)= 6 | map(6, -1) = 6
StepMap ranges    = "[1,5,5]"
invert(step)      = {"stepType":"replace","from":1,"to":6,"slice":{"content":[{"type":"text","text":"Hello"}]}}
AddMarkStep JSON  = {"stepType":"addMark","mark":{"type":"strong"},"from":1,"to":6}
marks no texto    = ["strong"]
undoDepth         = 2
após undo 1 text  = "Howdy world" marks = [] | undoDepth = 1 redoDepth = 1
após undo 2 text  = "Hello world"
após redo text    = "Howdy world"
seleção            = {"from":1,"to":6} -> após insert: {"from":2,"to":7}
```

Leitura dos números (para o dossiê):

- `"Hello world"` = 11 caracteres; `paragraph` com tokens = 13; `doc` = 15 → **+2 por nó não-folha**.
- posição 12 = token de fechamento do parágrafo (`textOffset = 0` porque não está dentro de texto).
- `resolve(6)` cai **dentro** do texto: `textOffset = 5` (depois de `"Hello"`), `nodeAfter = " world"`.
- o replace `1..6` gera `StepMap [1,5,5]` (5 caracteres antigos → 5 novos).
- o inverso do passo recupera `"Hello"`, não `"world"` — confirmando a invertibilidade.
- undo/redo removem e reaplicam **passos**, não snapshots de DOM.

## 3. `extra.js` — agrupamento de undo e flags de mapeamento (executado)

```bash
node extra.js
```

```
texto: abc de21f | undoDepth após 2 digitações adjacentes: 1
após 1 undo: abc def
delete 2..5 -> "adef"
mapResult(2) -> {"pos":2,"deleted":true,"deletedBefore":false,"deletedAfter":true,"deletedAcross":false}
mapResult(3) -> {"pos":2,"deleted":true,"deletedBefore":true,"deletedAfter":true,"deletedAcross":true}
mapResult(4) -> {"pos":2,"deleted":true,"deletedBefore":true,"deletedAfter":true,"deletedAcross":true}
mapResult(5) -> {"pos":2,"deleted":false,"deletedBefore":true,"deletedAfter":false,"deletedAcross":false}
Slice JSON: {"content":[{"type":"text","text":"adef"}]}
nodeBefore: a | nodeAfter: def
```

Notas honestas:

- `insertText("1", 7)` e `insertText("2", 7)` inserem **antes do `f`** (posição 7 em `"abc def"`),
  daí `"abc de21f"`. O ponto do teste é o **agrupamento**: duas edições adjacentes dentro de 500 ms
  viraram **um** evento de undo (`undoDepth = 1`) e um único undo restaurou `"abc def"`.
- `mapResult(2)` marca `deletedAfter` mas não `deletedBefore`; `mapResult(5)` (fim do range
  apagado) marca `deletedBefore` e não `deleted`. É a semântica de borda que permite a uma
  decoração/âncora escolher de que lado ficar.

## 4. Tamanho dos pacotes (executado)

```bash
for p in prosemirror-model prosemirror-state prosemirror-transform prosemirror-view; do
  printf "%-24s gzip=%s\n" $p "$(gzip -c node_modules/$p/dist/index.js | wc -c)"
done
```

```
prosemirror-model        gzip=29930
prosemirror-state        gzip=9405
prosemirror-transform    gzip=20363
prosemirror-view         gzip=61210
```

(`dist/index.js` não minificado; gzip do arquivo como publicado. Não é bundle final de aplicação:
falta tree-shaking/minificação e os demais módulos.)

## 5. Metadados de maturidade (executado)

```bash
npm view prosemirror-model time.created time.modified version
# time.created  = '2016-09-21T13:53:08.530Z'
# time.modified = '2026-09-21T13:05:07.365Z'
# version       = '1.25.12'
npm view prosemirror-view time.created
# 2016-09-21T13:53:28.098Z
```

E `prosemirror-model-1.25.12/CHANGELOG.md:1` → `## 1.25.12 (2026-09-21)`.

## 6. `minimal-editor.html` — **não executado**

Arquivo real de schema/editor mínimo (schema básico + keymap com Mod-b/Mod-i/Mod-z + history +
view + `attributes` de acessibilidade). **Não foi executado**: exige navegador e rede
(importmap via `esm.sh`) ou bundler, e não há navegador/jsdom neste ambiente.

Como verificar:

```bash
cd dossie/prosemirror/exemplos
python3 -m http.server 8080      # ou: npx serve .
# abrir http://localhost:8080/minimal-editor.html
```

O que se espera observar (inferido do código, **não observado**): digitar texto; Mod-b alternar
`<strong>`; Mod-z/Shift-Mod-z desfazer/refazer; botão "Ver JSON" imprimir o documento; sem o
`<link>` do `prosemirror.css`, quebras de linha e o contorno de nó selecionado ficam errados.

## 7. Suítes de teste — clonadas, **não executadas**

```bash
wc -l /tmp/pm-src/prosemirror-*/test/*.ts | tail -1
#   8932 total          (todos os pacotes clonados)
wc -l /tmp/pm-src/prosemirror-view/test/*.ts | tail -1
#   3271 total          (só a view: 3216 em webtest-*.ts + 55 em view.ts)
ls /tmp/pm-src/prosemirror-view/test/
# view.ts webtest-clipboard.ts webtest-composition.ts webtest-decoration.ts
# webtest-domchange.ts webtest-draw-decoration.ts webtest-draw.ts webtest-endOfTextblock.ts
# webtest-markview.ts webtest-nodeview.ts webtest-selection.ts webtest-view.ts
```

As `webtest-*` exigem navegador real (`@marijn/testtool` + browser), motivo pelo qual **nenhum
teste foi executado**. Os testes de model/transform/state (`test-*.ts`) são Node, mas exigem
build (`pm-buildhelper`/`@marijn/buildtool`) e `prosemirror-test-builder`, não instalados — também
não executados.

## 8. O que não foi possível rodar (e por quê)

| item | motivo | consequência |
|---|---|---|
| view em navegador | sem browser/jsdom; `contenteditable`/`Selection`/`MutationObserver` reais | afirmações de renderização/IME/clipboard vêm de código e das webtests (não executadas) |
| benchmarks (`ref/prosemirror/demo/bench/`) | exige build do meta-repo + browser | nenhum número de throughput |
| suíte oficial | dependências de build/browser ausentes; proibido instalar o monorepo | cobertura de comportamento limitada a execução headless |
| `prosemirror-tables` | clone falhou (repositório não encontrado no espelho usado) | tabelas tratadas só como "existe em módulo separado" |
