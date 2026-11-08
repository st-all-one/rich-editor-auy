# `codigo/` — índice das cópias

Todos os arquivos aqui são **trechos reais copiados do Zettlr 4.9.0** e existem
apenas como evidência de leitura. **Nada será reutilizado** no
`rich-editor-auy`: o Zettlr é GPL-3.0 (`ref/Zettlr/LICENSE:1`,
`ref/Zettlr/package.json:14`) e incorporar qualquer trecho obrigaria o projeto
inteiro a adotar a mesma licença.

Cada arquivo recebeu um cabeçalho no topo com origem exata, licença e motivo da
cópia. Os comentários originais (`BEGIN HEADER`) foram preservados nos arquivos
copiados integralmente.

| Arquivo local | Origem (`ref/Zettlr/...`) | Tamanho | Por que foi copiado |
|---|---|---|---|
| `links-provider-retrieve.ts` | `source/app/service-providers/links/index.ts:96-178` | 93 linhas | Coração da resolução de referências: `findExact` (ID → basename com extensão → basename sem extensão), `retrieveInbound`, `retrieveOutbound` e o descarte silencioso de links quebrados. |
| `extract-file-id.ts` | `source/app/service-providers/fsal/util/extract-file-id.ts:1-57` (íntegro) | 66 linhas | Identidade de uma publicação: ID vem do nome, senão do conteúdo; o guarda de `[[` antes do match funciona, o de `]]` é código morto (`match[1]` é o grupo capturado). |
| `zkn-link-parser.ts` | `source/common/modules/markdown-editor/parser/zkn-link-parser.ts:1-90` (íntegro) | 99 linhas | Parser inline Lezer de `[[alvo|título]]`, com suporte aos dois formatos e a guarda `pipeIdx > 0` — que na prática só rejeita o pipe à esquerda (ver `exemplos/link-edges.ts`). |
| `replace-links.ts` | `source/common/util/replace-links.ts:1-75` (íntegro) | 84 linhas | Reescrita de links internos na renomeação, usando `targetRange` do AST e aplicação em ordem reversa. |
| `remote-doc-authority.ts` | `source/common/modules/markdown-editor/plugins/remote-doc.ts:54-164` | 120 linhas | Sincronização editor ↔ autoridade via `@codemirror/collab`: pull em loop, push com versão e `reloadStateEffect` quando dessincroniza. |

## Como conferir a origem

```bash
cd /home/one/projects/one/rich-editor-auy
sed -n '96,178p' ref/Zettlr/source/app/service-providers/links/index.ts
diff <(sed -n '11,93p' dossie/Zettlr/codigo/links-provider-retrieve.ts) \
     <(sed -n '96,178p' ref/Zettlr/source/app/service-providers/links/index.ts)
```

(`codigo/links-provider-retrieve.ts` tem 10 linhas de cabeçalho próprio — o
conteúdo original começa na linha 11; os demais têm 9 linhas de cabeçalho, e o
conteúdo original começa na linha 10, inclusive `zkn-link-parser.ts`, que usa
cabeçalho `--` por ser um arquivo TypeScript mas o comentário escolhido é
linear.)

## Licença

Zettlr — GNU General Public License v3.0, Copyright (c) Hendrik Erz e
colaboradores. Cópia parcial para análise; ver `ref/Zettlr/LICENSE`.
