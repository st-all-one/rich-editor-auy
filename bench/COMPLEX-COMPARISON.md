# Comparativo de desempenho — documento complexo

## 1. Escopo e método

Este é um comparativo **headless de engine**, não de DOM, IME, paint ou leitor de tela. O corpus é
gerado por `bench/examples/all_types.rs` e contém, em 500 repetições, 4.000 blocos de topo e 82.000
bytes de texto (817.979 bytes de JSON canônico):

- documento com `lang`, `dir` e `title`;
- IDs de nó (`id`) em títulos e parágrafos;
- títulos e parágrafos com runs e marcas `strong`, `em`, `code`, `link` e `ref`;
- referência entre publicações (`doc`, `node`, `version`);
- átomo inline `image` com `alt` e `src`;
- listas com `list_item`;
- citação (`blockquote`);
- bloco de código (`code_block`);
- tabela com `table`, `table_row`, `table_header` e `table_cell`;
- separador atômico;
- árvore a11y/roles derivada do schema.

O ProseMirror foi executado com um schema equivalente, incluindo tabela, imagem, separador e as
marcas. O Lexical foi executado no subconjunto que ele representa sem nós customizados: títulos,
parágrafos, listas, link e marcas. Quill Delta e Tiptap não entram na tabela numérica: o primeiro é
uma sequência de ops sem árvore semântica; o segundo compartilha o núcleo do ProseMirror e adiciona
camada de extensões/DOM.

Ambiente: Linux x86_64, Rust 1.99 em `--release`, Node 22.23, ProseMirror 1.25.4, Lexical 0.53.0.
Foram feitas 5 execuções de aquecimento e 50–100 medições por operação. Os números são de uma
execução; não são um gate de CI.

Reprodução:

```sh
cargo run --release --manifest-path bench/Cargo.toml --example all_types
```

Os scripts de referência estão em [`reference/`](./reference/) e usam `prosemirror-model`, `prosemirror-state`, `lexical`,
`@lexical/headless`, `@lexical/rich-text`, `@lexical/list` e `@lexical/link`.

## 2. Resultado principal

Documento de todos os tipos, 4.000 blocos, ~818 KB de JSON:

| Operação | rich-editor-auy | ProseMirror | Leitura |
|---|---:|---:|---|
| Parse canônico (`from_json` / `Node.fromJSON`) | **26,98 ms** | **6,25 ms** | PM ~4,3× mais rápido |
| Parse bruto (`check_json`, sem decodificar Doc) | 11,30 ms | — | AUY gasta ~15,7 ms em decode+validação |
| Serialização canônica | **0,65 ms** | **9,04 ms** | AUY ~13,9× mais rápido |
| Aplicar 1 caractere | **0,003 ms** | **0,038 ms** | AUY ~12,7× mais rápido |
| Validar invariantes | 4,44 ms | sem equivalente no core | — |
| Árvore a11y | 10,10 ms | sem equivalente no core | — |
| Validar a11y | 10,92 ms | sem equivalente no core | — |
| JSON canônico | 817.979 B | 884.979 B | AUY ~8% menor |

O `apply` da AUY inclui `ChangeSet` e gravação de histórico; o `state.apply` do PM não inclui um
plugin de histórico. Mesmo assim a AUY ficou uma ordem de grandeza abaixo.

## 3. Documento comum (subset)

Para separar o efeito de tabela/imagem/refs, o corpus comum tem 3.000 blocos, 784 KB, com títulos,
parágrafos, listas, marcas `strong`/`em`/`code`/`link`/`ref`:

| Operação | AUY | ProseMirror | Lexical* |
|---|---:|---:|---:|
| Parse | 19,36 ms | 3,57 ms | 46,64 ms |
| Parse bruto | 9,17 ms | — | — |
| Serialização | 0,58 ms | 4,90 ms | 12,07 ms |
| Aplicar 1 char | 0,003 ms | 0,022 ms | — |

\* Lexical usou 2.500 nós e 1,3 MB de JSON, porque o formato serializado do Lexical é maior e o
subconjunto não tem refs, tabela ou imagem. Normalizando pelo JSON, o parse Lexical ficou em
~35,9 µs/KB contra ~33,0 µs/KB da AUY; a serialização ficou em ~9,3 µs/KB contra ~0,8 µs/KB. A
comparação por byte é indicativa, não equivalência de modelo.

## 4. O que os números explicam

### Parse: AUY perde para PM, mas entrega validação que o PM deixa para fora

O caminho AUY é: parser JSON estrito → `decode_block` → `Doc::new` → `Doc::validate`. O parser bruto
já custa 11,3 ms; decode e validação somam ~15,7 ms. O ProseMirror faz `Node.fromJSON` em 6,25 ms,
mas não valida a11y, IDs únicos, refs entre documentos ou os mesmos invariantes estruturais. A
comparação honesta é **parse com validação AUY** contra **parse PM + validação externa**, que ainda
não existe pronta no schema PM.

Ainda assim, o parser AUY é o gargalo arquitetural: ele materializa uma árvore `JsonValue` antes de
decodificar o documento. O próximo ganho grande é decodificar incrementalmente do parser para
`Block`/`Doc`, ou reutilizar buffers de parse.

### Serialização: streaming ganha da árvore intermediária

O writer AUY percorre `Doc` e escreve direto no `String`, com capacidade pré-calculada e cópia de
trechos sem escape. O PM precisa construir o objeto `toJSON()` inteiro e depois `JSON.stringify`.
Isso aparece como 0,65 ms contra 9,04 ms — e o JSON AUY ainda é 8% menor.

### Edição: chunks e passo invertível barato

O `apply` AUY em um documento de 4.000 blocos ficou em ~3 µs, contra ~38 µs do PM. O AUY copia
apenas o chunk de 128 blocos e o diretório; o PM paga a construção de novos nós/objetos JS e o custo
do JIT. O número é coerente com o orçamento de 100–200 ns por tecla do plano? Não: o orçamento
S1/S2 mede documentos sintéticos simples; neste corpus misto o custo sobe para a ordem de
microssegundos, mas continua muito abaixo de um frame.

### A11y: custo exclusivo do core AUY

A árvore a11y (10,1 ms) e sua validação (10,9 ms) não têm equivalente direto em PM/Lexical: os dois
delegam acessibilidade ao app/DOM. O custo é relevante — ~40% do parse — e deve ser opcional/ lazy
no adaptador, não gratuito em toda edição. Em compensação, o core entrega roles, nomes, `alt`,
`lang`/`dir`, ordem e diagnósticos determinísticos.

### Lexical: modelo mais pesado e formato maior

No subset, Lexical levou 46,6 ms para parsear 1,3 MB e 12,1 ms para serializar. O formato contém
mais metadados por nó (`detail`, `format`, `mode`, `style`, `version`, `direction`, `indent`) e o
`parseEditorState` reconcilia estado de editor, não apenas uma árvore. Ele ganha em modelo de
nós/extensões e integração React; perde em custo de representação e em completude semântica
(refs/tabela/imagem exigem nós customizados).

### Quill, Tiptap e taino

- **Quill**: Delta é uma sequência linear de `retain`/`insert`/`delete`. É excelente para mudanças
  planas; não representa tabela aninhada, refs entre documentos ou identidade de nó sem extensões.
  Comparar parse de Delta com parse de árvore seria comparar modelos diferentes.
- **Tiptap**: o custo de parse/serialize é o do ProseMirror + extensões + camada de comandos. Não há
  um core headless independente para medir; os números do PM são o piso, não o Tiptap completo.
- **taino-edit**: é o porte Rust mais próximo, mas não publica benchmark reproduzível nem orçamento
  de alocação; a comparação fica arquitetural (steps/schema), não numérica.

## 5. Leitura arquitetural

| Dimensão | rich-editor-auy | ProseMirror | Lexical |
|---|---|---|---|
| Modelo | árvore de blocos + inline plano | árvore + inline plano | árvore de nós tipados |
| Serialização | canônica, streaming, bytes estáveis | `toJSON` + JSON.stringify | objeto de estado + JSON |
| Parse | estrito + decodifica + valida invariantes | `Node.fromJSON` + schema | `parseEditorState` + registry |
| Edição | steps + mapping + chunks | steps + transform | update/double buffer |
| A11y | core (árvore + validação) | delegada | parcial/delegada |
| Refs entre docs | `RefTarget` no core | atributo de schema/app | atributo/custom node |
| Runtime | Rust, `no_std`, WASM | JS | JS |
| Risco principal | parse/validação caros | validação/a11y fora do core | formato e modelo mais pesados |

## 6. Conclusões

1. **A AUY ganha claramente em serialização e edição** neste corpus: writer streaming e chunks
   valem mais que a árvore JS.
2. **A AUY perde no parse** para o PM por ~4,3×; parte é validação adicional, parte é a árvore
   `JsonValue` intermediária. O parse bruto de 11,3 ms mostra que o decode é o alvo seguinte.
3. **A11y é o diferencial e o custo**: 10–11 ms para 4.000 blocos; deve ser lazy/opt-in no host.
4. **Lexical não é mais rápido** no subset medido; seu formato maior e o parse de estado cobram
   caro. A vantagem dele é modelo de extensão/React, não throughput de wire.
5. **Quill/Tiptap não são comparações de mesmo modelo**: Delta é plano; Tiptap é PM + camada.
6. O próximo experimento correto é **parse incremental** (parser → decode sem `JsonValue`) e um
   **benchmark de browser/IME** separado; os números acima não medem DOM, paint ou teclado.
