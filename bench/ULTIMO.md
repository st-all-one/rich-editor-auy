# Último relatório de desempenho

- Data: 2026-11-08
- Host: linux 7.2.15-300.fc45.x86_64, x86_64
- Toolchain: 1.99 (ver `rustc --version`)
- Build: `cargo bench --offline`, perfil `release` do workspace `bench/`
- Cenários: S1 (100 blocos) e S2 (10.000 blocos, 100 bytes por bloco, ~1 MiB de texto)

| Medida | Resultado | Alvo (`plan/05`) | Estado |
|---|---:|---:|---|
| `apply` + inserir 1 char (S2) | 107,5 µs | ≤ 10 µs | **acima** |
| `apply` + inserir 1 char (S1) | 2,04 µs | ≤ 100 ns | **acima** |
| `to_json_string` | 2,90 ms | ≤ 6 ms | ok |
| `from_json` (S2) | 35,57 ms | ≤ 15 ms | **acima** |
| `from_json` (S1) | 345,8 µs | ≤ 15 ms (S2) | informativo |
| `a11y_tree` | 11,30 ms | orçamento de render | informativo |
| `validate_a11y` | 11,41 ms | ≤ 10 ms | **acima** |

## Artefato WASM

`scripts/build_wasm.sh` gerou `rich_editor_auy_wasm_bg.wasm` com 322,0 KB no pacote JSR. `wasm-opt`
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
