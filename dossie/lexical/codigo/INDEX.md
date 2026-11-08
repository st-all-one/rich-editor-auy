# `codigo/` — cópias reais e sua origem

**Licença de todos os arquivos abaixo:** MIT.
Titular: Meta Platforms, Inc. and affiliates. Texto integral em `ref/lexical/LICENSE`.
A cópia é permitida com aviso de copyright; cada arquivo mantém o cabeçalho original, e os
trechos montados receberam um bloco `TRECHO COPIADO DE` adicional.

Nada aqui deve ser reutilizado diretamente no `rich-editor-auy`: o projeto é TS/JS sobre DOM e o
nosso é Rust headless. O valor é **arquitetural**.

---

## Índice

| Arquivo local | Origem | Linhas | Por que foi copiado | Alterações |
|---|---|---:|---|---|
| `LexicalEditorState.ts` | `ref/lexical/packages/lexical/src/LexicalEditorState.ts` | 331 (arquivo inteiro) | É o **coração do modelo**: `EditorState._nodeMap`, `_selection`, `_readOnly`, `clone`, `read`, `toJSON` legado vs. compacto, e os metadados de intenção (`_movedNodes`, `_splitNodes`, `_textEdits`) que a colaboração usa. Mostra que "documento" é um `Map<NodeKey, Node>` imutável e que a serialização tem **duas** formas. | Nenhuma. |
| `LexicalRichTextExtension.ts` | `ref/lexical/packages/lexical-rich-text/src/LexicalRichTextExtension.ts` | 141 (arquivo inteiro) | Prova o ponto central do dossiê: **"modo de edição" é uma extensão, não o núcleo**. 141 linhas que declaram `nodes`, `dependencies`, `conflictsWith`, `mergeConfig` e o `register` que chama `registerRichText`. Também mostra como config default + override se mesclam. | Nenhuma. |
| `LexicalUpdates-transforms.ts` | `ref/lexical/packages/lexical/src/LexicalUpdates.ts` (251-275, 288-306, 333-428) | 164 | O **algoritmo de transforms**: despacho por `__type` com cache, normalização de folhas e o fixpoint documentado "leaves antes de elements". É a peça mais diretamente portável para Rust. | Só concatenação de intervalos + cabeçalho de origem. |
| `LexicalTextNode-DOM-e-tema.ts` | `ref/lexical/packages/lexical/src/nodes/LexicalTextNode.ts` (168-192, 194-265, 902-924) | 144 | Mostra a **projeção modelo → DOM**: `format` vira tag semântica (`<strong>`, `<em>`, `<code>`, `<mark>`, `<sub>`, `<sup>`) e o tema vira classe. Inclui o caso especial `underlineStrikethrough` — a cicatriz de CSS atômico. | Só concatenação de intervalos + cabeçalho de origem. |
| `exemplo-main.ts` | `ref/lexical/examples/vanilla-js/src/main.ts` | 28 (arquivo inteiro) | **Inicialização mínima real, sem React**: `buildEditorFromExtensions` + `setRootElement`. É o "primeiro contato" honesto da biblioteca. | Nenhuma. |
| `exemplo-AppExtension.ts` | `ref/lexical/examples/vanilla-js/src/AppExtension.ts` | 45 (arquivo inteiro) | O que é preciso declarar para ter um editor que funciona: `$initialEditorState`, dependências (rich-text, clipboard, history) e `theme`. Também mostra o `registerUpdateListener` serializando o estado. | Nenhuma. |

Total copiado: **853 linhas**.

## O que ficou de fora (e por quê)

| Candidato | Linhas | Motivo da exclusão |
|---|---:|---|
| `LexicalSelection.ts` | 4.762 | Grande demais para cópia integral; os pontos citados estão referenciados por linha no `arquitetura.md` (§5) |
| `LexicalReconciler.ts` | 2.316 | idem; trechos citados em §4 |
| `LexicalSchema.ts` | 3.386 | idem; é o sistema **experimental** que recomendamos não replicar |
| `LexicalNode.ts` | 2.196 | idem; contrato do nó citado em §2 e §6.3 |
| `lexical-gc.ts`/`LexicalGC.ts` | 202 | Relevante, mas secundário frente aos seis acima |
| `LexicalBuilder.ts` (completo) | 524 | Só `buildEditorFromExtensions` + `fromExtensions`/`constructEditor`/`buildEditor` foram citados (linhas 81-160, 178-227); a cópia parcial não acrescentaria |

## Como os trechos foram montados

Os dois arquivos de trecho — `LexicalUpdates-transforms.ts` e `LexicalTextNode-DOM-e-tema.ts` —
foram gerados por concatenação mecânica dos intervalos listados, sem reescrita de código. O
cabeçalho MIT original foi preservado no topo e um bloco `TRECHO COPIADO DE` foi inserido entre o
cabeçalho e o código, com os intervalos exatos.

Comando usado (registrado para reprodutibilidade):

```sh
sed -n '168,192p;194,265p;902,924p' \
  ref/lexical/packages/lexical/src/nodes/LexicalTextNode.ts
sed -n '251,275p;288,306p;333,428p' \
  ref/lexical/packages/lexical/src/LexicalUpdates.ts
```

## Aviso

Projetos GPL não foram copiados aqui. Todos os arquivos deste diretório são MIT e podem,
legalmente, ser reutilizados com o aviso de copyright — mas a recomendação técnica permanece:
**não reutilizar código**, e sim o desenho descrito em `../arquitetura.md` §14.

## Verificação de integridade (revisão independente)

- `LexicalEditorState.ts`, `LexicalRichTextExtension.ts`, `exemplo-main.ts` e
  `exemplo-AppExtension.ts` são **byte a byte idênticos** aos originais em `ref/lexical/`
  (`diff` sem saída).
- `LexicalUpdates-transforms.ts` (164 linhas) e `LexicalTextNode-DOM-e-tema.ts` (144 linhas)
  contêm exatamente os intervalos declarados, na ordem declarada, com os marcadores
  `// ===== arquivo:faixa =====`.
- Total: 853 linhas, como na tabela.
