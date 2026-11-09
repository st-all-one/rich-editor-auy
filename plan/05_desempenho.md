# 05 — Desempenho: orçamento, medição e regressão

> **Regra.** Desempenho aqui não é "parece rápido". É um conjunto de **orçamentos** com gates
> determinísticos (alocação, visitas de nó, instruções) e benchmarks de parede apenas informativos.
> Um número sem cenário, build e máquina não é evidência.

---

## 1. Modelo de carga

"Documento de 1 MB" significa 1 MiB de texto UTF-8. Em português com acentos, 1 MiB ≈ 500 mil
caracteres; em ASCII, ≈ 1 milhão; em CJK, ≈ 350 mil. O modelo abaixo usa bytes, não caracteres.

| Cenário | Texto | Blocos | Runs | Uso típico |
|---|---|---|---|---|
| **S0 — micro** | 100 B | 1 | 1 | teste de unidade do caminho quente |
| **S1 — típico** | 10 KiB | 100 | 200 | um artigo/post |
| **S2 — grande** | 1 MiB | 10.000 | 5.000 | documento longo, relatório |
| **S3 — parágrafo patológico** | 1 MiB | 1 | ~256 | um único parágrafo gigante |
| **S4 — extremo** | 10 MiB | 100.000 | 50.000 | "não pode explodir" |

Operações medidas: inserir 1 caractere no início/meio/fim de um run; apagar 1 caractere; apagar um
range de 1 KiB; aplicar marca em 1 KiB; dividir bloco; juntar blocos; aplicar transação de 10
steps; serializar; parsear; validar; undo/redo.

## 2. Orçamentos

### 2.1 Caminho quente (release, x86_64, `apply_owned` com `Arc` único)

| Operação | Cenário | Alvo | Gate |
|---|---|---|---|
| inserir 1 char | S0/S1 | ≤ 100 ns | instruções + alocação |
| inserir 1 char | S2 | ≤ 200 ns | instruções + alocação |
| inserir 1 char | S3 | ≤ 2 µs | instruções |
| inserir 1 char | S4 | ≤ 500 ns | instruções |
| apagar 1 char | S1 | ≤ 100 ns | instruções |
| apagar range 1 KiB | S2 | ≤ 3 µs | instruções + alocação |
| `AddMark` 1 KiB | S2 | ≤ 5 µs | instruções + alocação |
| split/join de bloco | S1/S2 | ≤ 300 ns | instruções |
| transação de 10 steps | S2 | ≤ 10 µs | instruções |
| mapear posição por 10 steps | qualquer | ≤ 50 ns | instruções |

### 2.2 Caminho com snapshot (`apply(&self)`, COW)

| Operação | Cenário | Alvo | Observação |
|---|---|---|---|
| inserir 1 char | S2 (10k blocos) | ≤ 10 µs | cópia do vetor de 10k `Arc` + bloco tocado |
| inserir 1 char | S4 (100k blocos) | ≤ 50 µs | **gatilho para chunking** se passar |
| transação de 10 steps | S2 | ≤ 20 µs | uma cópia de vetor por transação, não por step |

**Decisão de engenharia:** o MVP otimiza `apply_owned`; o caminho `apply` com snapshot é correto e
limitado. Se o orçamento de S4 estourar, a resposta é `ChunkedVec` (D06), não "aceitar 100 µs".

### 2.3 Serialização e parse

| Operação | Cenário | Alvo | Gate |
|---|---|---|---|
| `to_json_into` (writer sem alocação por nó) | S2 | ≤ 5 ms | instruções |
| `to_json_string` (uma alocação com estimativa) | S2 | ≤ 6 ms | instruções |
| `from_json` (parser próprio) | S2 | ≤ 15 ms | instruções + fuzz |
| `check_json` (validação) | S2 | ≤ 33 ms | instruções (2× os 16,5 ms medidos) |
| `validate` total | S2 | ≤ 10 ms | instruções |
| `normalize` local pós-step | S2 | ≤ 1 µs | instruções |

O writer recebe `&mut impl core::fmt::Write`; não monta uma árvore JSON intermediária. O parser lê
`&str` com índice e `try_reserve`; não cria `Value` genérico.

### 2.4 Árvore de acessibilidade

`Doc::a11y_tree` e `validate_a11y` são **on-demand**, não caminho de tecla: percorrem o documento
em O(n) e alocam por nó. Elas entram no orçamento de render/validação (S2/S4), nunca no gate de
1 char. O adaptador pode cachear a árvore entre transações e reconstruí-la no máximo uma vez por
frame; se reconstruir a cada tecla, o custo é do adaptador, não uma desculpa para remover a a11y.
`validate_a11y` roda em CI/`make a11y`, não em `apply`.

Isso mantém a invariante A11: a árvore é derivada e não é wire; o caminho quente continua sendo
`apply`/`to_json`.

### 2.5 Memória

Fórmula de orçamento (capacidade, não `len`):

```text
memória ≤ 2,0 × bytes_de_texto
        + 64 B × blocos
        + 24 B × runs
        + 16 B × inline_items
        + 32 B × attrs
        + 64 KiB (overhead)
```

Para S2 (1 MiB, 10k blocos, 5k runs), o teto é ≈ 2,0 MiB + 640 KiB + 120 KiB + 80 KiB + 64 KiB ≈
2,9 MiB. O teste de memória mede `live_bytes` com um alocador de contagem e falha se exceder.

O histórico tem orçamento separado: 8 MiB / 200 grupos por default. Uma deleção de 1 MiB guarda
1 MiB no inverse; o orçamento pode ser excedido e o grupo mais antigo é evictado. O teste verifica
que o teto é respeitado e que o undo ainda funciona para o que ficou.

## 3. Como medimos

### 3.1 Contagem de alocação (gate determinístico)

Um alocador global de teste conta alocações e bytes vivos. O teste roda operações em S0–S2 e
afirma:

- inserir 1 char em run com capacidade: **0 alocações**;
- inserir 1 char que cresce o `String`: **1 alocação** (amortizada);
- split de run: **1 alocação** para o novo `String` (ou 0 se houver capacidade sobrando);
- `AddMark` em 1 KiB sem splits: **0 alocações**;
- `apply_owned` de 1 step: **0 alocações** além das do step;
- `apply` com snapshot em S2: **1 alocação** (novo vetor) + blocos tocados.

O alocador é `#[cfg(test)]`/feature `test-alloc`; não há `unsafe` no core, mas o alocador global
usa `unsafe` no crate de teste (permitido no testkit, com `SAFETY` e sem entrar no core).

### 3.2 Contagem de visitas (gate de complexidade)

Iteradores e funções internas incrementam um contador `#[cfg(test)]`:

- inserir 1 char em S4 deve visitar **≤ 2 × profundidade + 1 bloco** (não varrer os 100k blocos);
- `normalize` local deve visitar **≤ runs tocados + 2**;
- `resolve` deve visitar **≤ profundidade**;
- `validate` total pode visitar todos (é O(n), por definição).

O teste falha se a contagem crescer com o tamanho do documento em uma operação local. Isso pega
O(n) escondido sem depender de wall clock.

### 3.3 Instruções (`iai-callgrind`)

`iai-callgrind` roda sob Valgrind e mede instruções, cache misses e branches de forma
determinística. É o gate de regressão de performance:

- baseline nativo em `bench/target/iai/main.json`, restaurado e salvo pelo cache do CI
  (a chave cobre `bench/Cargo.lock`, `bench/src/lib.rs` e `bench/benches/iai_hot.rs`);
- `soft_limits` de **5%** em `Ir` no próprio bench; um `Ir` acima da baseline é regressão e o
  runner sai com código 3, então o job quebra (limites absolutos em `hard_limits` não expressam
  "5% acima da baseline");
- o baseline só é gravado quando ainda não existe (`--save-baseline=main` no primeiro run);
- mudança de baseline exige justificativa no PR e commit separado.

Benches de `iai` são pequenos e cobrem S0–S2; S3/S4 entram como "smoke" para provar que não
explodem, não como gate de tempo.

### 3.4 Parede (`criterion`)

`criterion` mede tempo, desvio e throughput; é **informativo**. Não falha o CI por diferença de
máquina. Serve para:

- detectar regressões grosseiras (2×) em revisão manual;
- comparar alternativas de implementação;
- alimentar o relatório `bench/ULTIMO.md`.

### 3.5 Perfil e alocação

- `dhat` para perfil de alocação em S2/S3; o relatório mostra os maiores sítios.
- `perf`/`flamegraph` para o caminho quente; não é gate.
- `cargo llvm-cov` é observação; cobertura alta não substitui teste de propriedade.

## 4. Análise de complexidade por operação

| Operação | Custo | Por quê |
|---|---|---|
| `resolve(pos)` | O(profundidade) | desce a árvore; profundidade ≤ 64 |
| `insert_char` | O(run) ≤ 4 KiB + O(runs no bloco) | encontra o run e faz memmove; runs limitados |
| `delete_range` | O(runs tocados + texto removido) | divide runs nas bordas |
| `add_mark` | O(runs tocados) | divide runs nas bordas e aplica máscara |
| `split_block` | O(run + irmãos do bloco) | copia o vetor de irmãos no nível; profundidade |
| `join_blocks` | O(blocos dos dois lados) | move conteúdo; normaliza |
| `apply(tx)` | O(steps × custo do step + blocos tocados) | COW só nos tocados |
| `serialize` | O(nós + texto) | uma passada |
| `parse` | O(bytes) | uma passada, sem árvore intermediária |
| `validate` | O(nós + texto) | uma passada |
| `undo` | O(steps do grupo) | aplica inversos |

O caminho quente **não** é O(documento). As únicas operações O(documento) são as que
intrinsecamente leem tudo: serializar, parsear, validar e buscar. Elas não rodam por tecla.

## 5. COW: os dois caminhos, com números

Suponha S2: 10.000 blocos, vetor de `Arc<Block>` com 80 KiB de ponteiros.

- **`apply_owned`, `Arc` único:** nenhum clone do vetor; o bloco tocado é mutado no lugar (ou
  clonado uma vez se o `Arc<Block>` estiver compartilhado). Custo ~100–200 ns.
- **`apply(&self)`, snapshot retido:** `DocInner` é clonado; o vetor de 10k `Arc` é copiado
  (80 KiB) e o bloco tocado é clonado. Custo ~3–10 µs, dependendo do alocador.
- **S4 (100k blocos):** a cópia é 800 KiB; ~30–50 µs. Se o gate de 50 µs for violado, o próximo
  passo é `ChunkedVec` (blocos em chunks de 16–64, com prefix sums), que reduz a cópia a
  O(log n) chunks.

A UI deve preferir `apply_owned` e descartar o estado antigo depois de renderizar. Se precisar de
snapshot (ex.: animação, histórico visual, thread de análise), use `apply` e aceite o custo; o
benchmark cobre os dois.

## 6. Texto: por que runs de 4 KiB

Um `String` de 1 MiB inserido no meio paga ~1 MiB de memmove por tecla. A 30 GB/s, isso é ~33 µs —
acima do orçamento e suficiente para engasgar 60 fps em um documento grande. Com runs de 4 KiB, o
pior memmove é 4 KiB (~0,13 µs) e o número de runs de um bloco de 1 MiB é ~256.

Custo do teto:

- mais runs por bloco (metadado ~40 B cada; 256 runs = 10 KiB para 1 MiB de texto — 1%);
- a UI precisa mesclar runs adjacentes com a mesma marca para desenhar (barato);
- a serialização canônica divide o texto de forma determinística, então o mesmo conteúdo produz os
  mesmos runs.

A divisão é em fronteira de escalar UTF-8, não de grapheme. Runs não são unidades semânticas; a
seleção e a edição é que precisam de grapheme, tratado em `motion`.

## 7. Casos patológicos e mitigação

| Caso | Risco | Mitigação | Teste |
|---|---|---|---|
| Parágrafo de 1 MiB | memmove O(n) | runs de 4 KiB | S3: insert ≤ 2 µs |
| 100k blocos + snapshot | COW O(#blocos) | `ChunkedVec` se > 50 µs | S4 smoke |
| Aninhamento 64 | stack overflow na validação/serialização | travessia iterativa com pilha explícita; limite de profundidade | teste de profundidade |
| Tabela 100×100 | editar célula copia 100 irmãos | chunking de filhos ou estrutura de tabela dedicada pós-MVP | S2 table smoke |
| Deleção de 1 MiB no undo | memória do inverse | orçamento de histórico + evicção | teste de memória do histórico |
| Atributo string de 64 KiB | hash/serialização | limite + `try_reserve` | teste de limite |
| 64 marcas | máscara `u64` | schema recusa > 64 | teste de schema |
| Muitos runs por bloco | scan O(runs) | teto de 4 KiB limita a 256; índice local se medido | S3 |
| Serialização de 10 MiB | alocação dupla | writer em `fmt::Write`, sem árvore intermediária | S4 serialize |
| Parse hostil | memória/tempo | limites antes de alocar; parser fuzzado | fuzz + limites |

## 8. Suíte de benchmark

`bench/` (workspace próprio, `harness = false`):

```text
bench/
  Cargo.toml
  src/lib.rs          # geração de documentos S0–S4 determinística
  benches/hot.rs      # criterion: insert, delete, mark, split, join
  benches/wire.rs     # criterion: serialize, parse, validate
  benches/iai_hot.rs  # iai-callgrind: mesmos cenários, gate de instruções
  target/iai/         # baseline de instruções (nativo, cacheado pelo CI)
  ULTIMO.md           # relatório legível da última execução (commitado)
```

Os documentos de bench são gerados por uma função pura (sem RNG) a partir de um seed fixo; o
mesmo corpus é usado por todos os benches. Nada de `rand` no caminho de medição.

## 9. Política de regressão

1. **PR que toca `model`, `step`, `transform`, `state`, `history` ou `wire` roda o bench.**
2. Regressão > 5% em instruções no caminho quente: bloqueia merge até explicação/otimização.
3. Aumento de alocação no caminho quente: bloqueia merge, salvo justificativa e novo gate.
4. Aumento de memória acima da fórmula: bloqueia merge.
5. Mudança de baseline: commit separado, com antes/depois e razão.
6. Bench que só passa em uma máquina não é gate; `iai` no runner fixo é o gate determinístico.

## 10. O que não é gate

- Tempo absoluto de parede em máquina de dev.
- FPS da UI, LCP/INP, tempo de layout, custo de fonte.
- Tamanho do binário final do consumidor (é consequência, não contrato).
- Cobertura de linha (é observação; testes de propriedade valem mais).
- "Número de LOC" (o gate é 300 linhas por arquivo, não o total).

## 11. Calibração inicial

Os alvos de §2 são **hipóteses de engenharia**, não medições. A calibração executada:

1. caminho quente de S0/S1 implementado e medido;
2. `criterion` mede parede e `iai-callgrind` mede instruções; `tests/alloc.rs` conta alocações;
3. baseline fixado com **2× de folga** sobre o medido, para não transformar ruído em falha;
4. recálculo só com mudança de desenho registrada, não para "passar o CI".

Resultado medido (host de desenvolvimento, release; detalhes em `bench/ULTIMO.md`) e alvo
calibrado com 2× de folga:

| Operação | Cenário | Medido | Alvo calibrado | Motivo da calibração |
|---|---|---|---|---|
| inserir 1 char | S0 | 510–571 ns | ≤ 1,2 µs | custo fixo do step/histórico (10 alocações), sem `unsafe` |
| inserir 1 char | S1 | 0,97–1,10 µs | ≤ 2,2 µs | idem |
| inserir 1 char | S2 | 1,75–1,98 µs | ≤ 4 µs | idem |
| inserir 1 char | S3 | 44–81 µs | ≤ 170 µs | O(runs do bloco): reconstruir 256 runs |
| apagar 1 char | S1 | 1,03 µs | ≤ 2,1 µs | custo fixo do step |
| apagar 1 KiB | S3 | 42,4 µs | ≤ 85 µs | O(runs do bloco) |
| `AddMark` 1 KiB | S1 | 4,32 µs | ≤ 8,6 µs | O(runs do bloco) |
| split/join | S1 | 3,1–3,2 µs | ≤ 6,4 µs | reconstrução do bloco |
| transação de 10 steps | S1 | 10,5 µs | ≤ 21 µs | uma cópia de bloco por step |
| mapear posição | S2 | 1,8 ns | ≤ 50 ns | alvo mantido |
| `from_json` | S2 | 12,8 ms | ≤ 15 ms | alvo mantido |
| `check_json` | S2 | 16,5 ms | ≤ 33 ms | sem alvo anterior; calibrado com 2× de folga |
| `to_json_string` | S2 | 2,75 ms | ≤ 6 ms | alvo mantido |
| `Doc::validate` | S2 | 3,72 ms | ≤ 10 ms | alvo mantido |
| `validate_a11y` | S2 | 2,34 ms | ≤ 10 ms | alvo mantido |

O alvo de 100 ns para inserir 1 char em S0/S1 não é atingível sem remover as alocações fixas do
step (texto, `Vec` de itens, `Arc<Block>`, chunk, `ChangeSet`, grupo de histórico) — o que exigiria
arena ou `unsafe`, ambos proibidos no core. O gate real é o par instruções + teto de alocação: uma
regressão de 5% em `Ir` falha o CI, e `tests/alloc.rs` falha se o número de alocações subir.

Se a calibração mostrar que um alvo é impossível sem `unsafe`, a resposta é **mudar o desenho**
(ex.: chunking, gap buffer, runs compartilhados), não relaxar `forbid(unsafe_code)`.
