# Último relatório de desempenho

- Data: 2026-11-08
- Host: linux 7.2.15-300.fc45.x86_64, x86_64
- Toolchain: 1.99 (ver `rustc --version`)
- Build: `cargo bench --offline`, perfil `release` do workspace `bench/`
- Cenários: S1 (100 blocos) e S2 (10.000 blocos, 100 bytes por bloco, ~1 MiB de texto)

| Medida | Resultado | Alvo (`plan/05`) | Estado |
|---|---:|---:|---|
| `apply` + inserir 1 char (S2) | 2,89 µs | ≤ 10 µs | **ok** |
| `apply` + inserir 1 char (S1) | 1,73 µs | ≤ 100 ns | **acima** |
| `to_json_string` | 1,12 ms | ≤ 6 ms | ok |
| `from_json` (S2) | 20,87 ms | ≤ 15 ms | **acima** |
| `from_json` (S1) | 344,0 µs | ≤ 15 ms (S2) | informativo |
| `a11y_tree` | 10,50 ms | orçamento de render | informativo |
| `validate_a11y` | 11,04 ms | ≤ 10 ms | **acima** |

## Artefato WASM

`scripts/build_wasm.sh` gerou `rich_editor_auy_wasm_bg.wasm` com 335,6 KB no pacote JSR. `wasm-opt`
não estava instalado; o valor é o artefato sem otimização adicional. O checksum BLAKE3 não foi
gerado porque `b3sum` também não está instalado.

## Evidência

O comando foi executado no host acima; os intervalos completos ficam no log da execução. O índice
de blocos (`Doc::block_at`) e o teste `tests/visits.rs` provam que a consulta local deixou de varrer
o documento. O teste `tests/memory.rs` cabe na fórmula de memória após `TextBlock::new` reservar
exatamente a capacidade necessária.

## Gate determinístico

`iai-callgrind` não está instalado neste host; o baseline de instruções e o gate de regressão de 5%
permanecem abertos em `DIVERGENCES.md` §F/G. Benchmarks de parede são informativos e não falham o
CI por variação de máquina.

## Análise do caminho quente

Perfil instrumentado em S2 (10.000 blocos, 100 bytes por bloco):

| Etapa | Tempo | Alocações | Bytes |
|---|---:|---:|---:|
| `transaction.clone()` | 55 ns | 2 | 97 B |
| `step.invert()` | 362 ns | 10 | 1.264 B |
| `apply_step()` antes dos chunks | 104,8 µs | 11 | 120.927 B |
| `ChangeSet::from_steps()` | 77 ns | 3 | 81 B |
| `state.apply()` antes dos chunks | 106,5 µs | 32 | 123.325 B |
| `state.apply()` com chunks | 2,89 µs | 35 | ~5,9 KiB |
| `state.apply_owned()` | 3,65 µs | 25 | ~4,5 KiB |

`apply_step` escalava linearmente com o número de blocos (~10 ns/bloco em 10k), porque copiava o
vetor de 10.000 `Arc<Block>` e o índice de inícios. Um protótipo isolado mostrou:

| Estratégia | Tempo | Bytes |
|---|---:|---:|
| Substituir em vetor flat de 10k | 102,7 µs | 80.000 B |
| Substituir em chunks de 128 | 2,8 µs | 3.328 B |

A implementação adotou chunks de 128 com diretório de `Arc<Chunk>` e índice de inícios por chunk.
O caminho quente agora copia um chunk e o diretório, não o documento, e reutiliza o `starts` do
chunk ajustando apenas o sufixo. O custo restante é dominado pelo texto do bloco, inversão e pelas
35 alocações pequenas. S1 (100 blocos) usa o chunk flat e ainda paga texto/inversão, por isso 1,73 µs. `apply_owned` elimina 10 alocações ao mutar o `DocInner`/chunk em
 vez de copiar, mas o caminho de uma etapa ainda fica próximo do imutável; a inversão agora
 usa um único `slice_text`.

## A/B com Serde

S2, mesmo JSON canônico de 10.000 blocos. O parser AUY é o caminho `check_json` (parse estrito,
descarta a árvore); o Serde parseia para `serde_json::Value`. A serialização Serde parte de um
`Value` pré-construído; a AUY parte do `Doc` e escreve o formato canônico.

| Operação | AUY | Serde | Leitura |
|---|---:|---:|---|
| Parse S2 | 8,89 ms | 19,77 ms | AUY ~2,2× mais rápido |
| Serialize S2 | 1,12 ms | 3,95 ms | AUY ~3,5× mais rápido |

O parse é praticamente empatado; a diferença está no modelo de árvore e na alocação de strings.
Na escrita, o writer incremental da AUY evita a árvore intermediária e a maquinaria de `Serialize`
do `Value`. O benchmark não mede a decodificação semântica (`from_json`) contra Serde, porque
Serde não tem um decodificador de schema equivalente no projeto.
