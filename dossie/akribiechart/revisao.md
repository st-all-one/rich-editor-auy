# akribie — revisão independente

> Revisão do dossiê [`dossie/akribiechart/`](.) contra a fonte `ref/akribiechart`, commit
> `7275fedfe81f2d4ac1df4e92a580c05b90a81049` (v0.1.3, 2026-11-03, `MIT OR Apache-2.0`).
> Revisor independente; nada em `ref/` foi alterado (`git -C ref/akribiechart status --porcelain`
> → vazio). Builds em `CARGO_TARGET_DIR=/tmp/dossie-akribie-review`.

---

## 1. Método

Não li o dossiê como fonte: reexecutei os pontos checáveis e comparei com o código no commit.
Rodei tudo em `ref/akribiechart` (leitura) e em `/tmp` (build e fixtures). O `/tmp` é um tmpfs de
8,7 GB compartilhado com outros dossiês; o build completo estourou a quota duas vezes e só passou com
`CARGO_PROFILE_DEV_DEBUG=0` (sem debuginfo). Isso não altera código nem testes.

Comandos executados nesta revisão:

```sh
cd ref/akribiechart
CARGO_PROFILE_DEV_DEBUG=0 CARGO_TARGET_DIR=/tmp/dossie-akribie-review cargo test --workspace --locked
CARGO_PROFILE_DEV_DEBUG=0 CARGO_TARGET_DIR=/tmp/dossie-akribie-review \
    cargo build -p akribie-core --no-default-features          # falha esperada: E0433/E0432
```

Mais, contra os binários (release pré-existente do dossiê e o `debug` compilado na revisão):
`check`, `render` (svg/scene/excalidraw/png), `fmt`, `diff`, `schema`, `prime`, `import`,
`check --batch`, `render --batch --out-dir`, MCP por stdio, `xmllint`, `sha256sum`, `wc -l` e
medição de tempo com mediana (11 execuções, com aquecimento).

**Verificações de integridade**

- `sha256sum` dos 6 arquivos de `codigo/` contra a origem: **6/6 idênticos**; 1 246 linhas no total.
- Contagem de testes: `grep -rn '#\[test\]' crates --include=*.rs | wc -l` → **230**; a suíte
  reexecutada soma exatamente **230 passed, 0 failed, 0 doc-tests**.
- `ref/` limpo; nenhuma alteração fora de `dossie/akribiechart/`.

## 2. O que foi confirmado (sem alteração)

| Afirmação central | Reexecução / evidência |
|---|---|
| 230 testes, 0 falhas, 0 doc-tests, com a quebra por alvo | Suíte reexecutada: core 117, lex 7, parse 13, cli unit 7, check 22, diff 10, docs 4, fmt 11, import 6, init 2, prime 2, render 23, mcp 1+5, adapters 0 = **230** |
| 6 cópias verbatim com SHA | `sha256sum` confere nos 6 arquivos; `codigo/` = 1 246 linhas |
| Determinismo de bytes | SVG `0f98809a…`, scene `ca83980b…`, excalidraw `a02bf55e…`, PNG `99a56460…` (1672×344 RGBA), todos iguais em duas execuções; **debug e release dão o mesmo sha256** |
| Feature `raster` não compila | `--no-default-features` → `E0433` (`image`, `usvg`, `tiny_skia`, `resvg`) + `E0432`; 8 erros, todos em `emit/raster.rs`; `grep 'feature = "raster"'` → 0 |
| `<desc>` português com `lang=en` | `render` de `chart … lang=en` emite `lang="en"` e `<desc>2 nós, 1 aresta. Nós: …` |
| `role="list"` prometido e não emitido | `grep -c 'role="list'` → 0; `tabindex` → 0; o emissor usa `<g id="nodes">` sem `role` |
| `warnings` sempre vazio | `check --strict --json` mostra os `W002` em `data.diagnostics[]` e `"warnings":[]`; campo só recebe `Vec::new()` em `envelope.rs:46,60,78` |
| `import` lossy | Round-trip produz `layout=free`, `shape=rounded` para todos, sem `curve=arc`, com `at=`/`size=` e prefixo `akr-` |
| `edge a->b` = E007 | `check` → `E007 malformed-line: token inválido 'a->b'`, exit 8 |
| `--out-dir` exige diretório | `render --batch … --out-dir saida` → exit 3 |
| Exit codes | 0 (`check` ok), 1 (`fmt --check` e `diff`), 2 (formato/`--params`), 3 (arquivo/`--out-dir`), 8 (`.akrc` inválido); 5 e 70 vêm da tabela de `error.rs:37-46` |
| 16 códigos de diagnóstico | `diag::ALL: [Code; 16]`; `akr schema` traz 16 entradas e `akribie_version: "0.1.3"` |
| `akr prime` 2 742 / 6 110 bytes | `wc -c` confere |
| MCP estável | `initialize` anuncia `2024-11-05`; `chart_check` devolve `content[]` e o processo não cai |
| Docs desatualizados | `00-panorama.md:39-40` cita 4 portas (`Env`/`Logger`); `DETERMINISM.md:32` cita `Env`; `A11Y.md:16,25` promete `role="list"`/`listitem` |
| Lookup linear | `Chart::node`/`label_of` usam `iter().find` (`model/mod.rs:151-174`) |
| Licença | `MIT OR Apache-2.0`; `LICENSE` Apache-2.0 integral; fontes OFL-1.1 citadas e **não** copiadas |
| Performance (ordem de grandeza) | big1000: **16,6 ms `check` / 85,2 ms SVG / 62,3 ms scene** (dossiê: 14,4 / 80,8 / 53,1); razão ~5,1× (dossiê ~5,6×). Mesma ordem; diferença compatível com máquina compartilhada |
| Forma canônica idempotente | `fmt(fmt(x))` dá o mesmo md5 (`79174740…`) |
| XML válido | `xmllint --noout` aceita o SVG |
| Um único commit | `git log --oneline` → só `7275fed` |

## 3. Correções factuais aplicadas

1. **`check --batch` não "sempre sai 0".** O processo sai `0` quando **ao menos um** item passa e
   `8` quando **todos** falham (`crates/akribie-cli/src/app/emit/batch.rs:21-24`). Verificado:
   lote 2 ok + 2 erro → `exit 0`; lote só com erro → `exit 8`. Corrigido em:
   - `README.md` §6.8;
   - `avaliacao.md` item 12 e §4.3.6;
   - `arquitetura.md` §9 e §13.6 (título incluído);
   - `exemplos/README.md` §9.1 (título + nota).
2. **Literal do erro de `--out-dir`.** O binário usa crases, não aspas simples:
   ``erro: diretório de saída não encontrado: `saida` `` (`commands/render/batch.rs:87`).
   Corrigido em `README.md` §6.7. O `exemplos/README.md` já estava certo.
3. **Literal do formato inválido.** `erro: formato `pdf` desconhecido…` usa crases; o dossiê citava
   `'pdf'`. Corrigido em `avaliacao.md` (linha da usabilidade de integração).
4. **Contagem de linhas das fixtures.** Com o gerador descrito pelo próprio dossiê:
   `retry.akrc` = **9** (não 10), `star120.akrc` = **241** (não 242), `big1000.akrc` = **2001**
   (não 2000); `big200.akrc` = 401 (correto). Corrigido nas três tabelas de performance
   (`README.md` §3.3, `arquitetura.md` §12, `exemplos/README.md` §5.2). Os tempos foram mantidos —
   reproduzem dentro da variação do ambiente, e uma nota em `README.md` §3.3 registra a remediação.
5. **Harness: "quatro chamadas" → "cinco".** `exemplos/harness/src/main.rs` usa `Font::load`,
   `pipeline::check`, `layout::layout`, `paint::paint`, `emit::svg` (5). O próprio `avaliacao.md`
   já dizia "5 chamadas"; o `README.md` §4.2 foi alinhado.
6. **Exit code `5` (I/O) ausente da tabela.** `error.rs:37-46` o define; adicionado ao `README.md`
   §4.3 como "não exercitado", junto de `70`.
7. **Citações de linha do README do projeto estavam deslocadas** (apontavam para o README do
   dossiê ou para linhas erradas do upstream):
   - envelope `{"success",…}`: `README.md:77-79` → `ref/akribiechart/README.md:89-93`
     (`arquitetura.md`) e `:91` (`ideias.md`);
   - batch "best-effort": `README.md:71-73` → `ref/akribiechart/README.md:92-93` (`README.md`
     §6.8);
   - instalação por `install.sh`: `README.md:26-40` → `ref/akribiechart/README.md:30-51`
     (`avaliacao.md`);
   - "silencioso quando está tudo certo": `README.md:50-52` → `ref/akribiechart/README.md:57`
     (`exemplos/README.md`).
8. **Citação da licença apontava para a linha errada.** `license = "MIT OR Apache-2.0"` está em
   `ref/akribiechart/Cargo.toml:19` (linha 17 é `edition = "2024"`); `codigo/INDEX.md`, `README.md`
   §1 e §8 foram corrigidos para `:19`.
9. **Referências de seção quebradas:**
   - `edge a->b` é erro em **§4.3** (item 5), não §4.2 (`avaliacao.md`, tabela de usabilidade);
   - import lossy está em **§6.6** do README do dossiê, não §4.6 (`avaliacao.md` §4.3.7).

Nenhuma reescrita por estilo: só fatos, números, literais e ponteiros.

## 4. O que o dossiê viu corretamente e merece destaque

- O achado da feature `raster` é real e grave: a matriz de features do CI é teatro.
- A acessibilidade derivada do modelo é verificável no byte e o contraponto de idioma é exato.
- A separação `check` × `render` é medida, não retórica (≈5× mais barato).
- `codigo/` é honesto: verbatim, com hash, licença e origem; OFL não foi copiado.
- A matriz de 13 itens está majoritariamente evidenciada por execução/código/teste; os dois `—`
  (edição interativa e colaboração) são ausência real, não lacuna de leitura.

## 5. Lacunas que permanecem (não verificadas nesta revisão)

- **`make check` / `make ci` completos**: exigem `cargo-deny`, `typos` e `xtask` com rede.
- **`install.sh` e os 6 alvos de release**: exigem rede.
- **`akr init` e `akr --at REV`**: não rodados para não escrever em `ref/`; `--at` exigiria
  histórico git (o clone tem um commit).
- **Leitor de tela real** (NVDA/VoiceOver/Orca): a conclusão sobre `<desc>` é do byte emitido.
- **Harness de biblioteca**: não reexecutado nesta rodada; as 5 chamadas foram conferidas no fonte e
  o comportamento equivalente foi exercitado pela CLI.
- **Números de performance de máquina pequena**: reproduzidos na ordem de grandeza, não ao
  milissegundo, porque o `/tmp` e a CPU são compartilhados com outros dossiês.

## 6. Links e citações

- **Links relativos quebrados: 0** (varredura de todos os 7 `.md` do diretório; alvos existentes).
- **Citações `caminho:linha`: 318** ocorrências nos 6 documentos principais (regex
  `arquivo.ext:linha[-linha]`). O número "305" citado na tarefa é uma contagem com outra regra; o
  valor reprodutível aqui é 318 com 0 links quebrados.
- `ref/` limpo; apenas arquivos dentro de `dossie/akribiechart/` foram editados.

## 7. Confiança final

| eixo | confiança | base |
|---|---|---|
| Arquitetura e modelo de dados | **95%** | pipeline, camadas, portas e determinismo lidos no fonte e exercitados |
| Testes e suíte | **100%** | suíte completa reexecutada: 230/230, 0 doc-tests |
| Determinismo | **100%** (mesma máquina/versão) | sha256 de 4 formatos, debug = release |
| Performance | **85%** | big1000 reproduzido (16,6/85,2/62,3 ms; razão 5,1×); ambiente compartilhado |
| Acessibilidade | **90%** | estrutura/contraste/idioma verificados no byte; sem leitor de tela real |
| CLI/MCP/batch/import | **95%** | comportamentos e exit codes reexecutados |
| Cobertura da matriz de 13 itens | **13/13 com status** (8 por execução, 3 por código/teste/doc, 2 `—` reais) | — |

**Confiança final: 90%.** Acima do piso de 85% porque a matriz está majoritariamente evidenciada e
as afirmações centrais foram reexecutadas; não chega a 95%+ porque `make ci`, `install.sh`, o
harness e um leitor de tela real ficaram fora desta rodada, e os tempos variam com a máquina.
