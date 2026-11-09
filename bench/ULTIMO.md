# Último relatório de benchmark

Execução: `make bench` (`cargo bench --bench hot --bench wire`) em `x86_64`, toolchain 1.99,
perfil `release`, host de desenvolvimento. `criterion` mede parede e é **informativo**; o gate de
regressão é o `iai-callgrind` (`cargo bench --bench iai_hot`, `hard_limits` de 5% em `Ir`).

O gate `iai` foi executado em Docker (`rust:1.99-bookworm`, `--security-opt seccomp=unconfined`,
valgrind + runner 0.16.1): baseline `main` gravado e comparado no mesmo container, resultado
`Iai-Callgrind result: Ok. 10 without regressions; 0 regressed`.

Os documentos vêm de `bench/src/lib.rs` (`fixture(Scenario)`), gerados por função pura:

| Cenário | Blocos | Runs | Texto | JSON |
|---|---|---|---|---|
| S0 | 1 | 1 | 100 B | 185 B |
| S1 | 100 | 200 | 10 000 B | 21 KiB |
| S2 | 10 000 | 5 000 | 1 000 000 B | 1,5 MiB |
| S3 | 1 | 256 | 1 048 576 B | 1,0 MiB |
| S4 | 100 000 | 100 000 | 10 000 000 B | 15,9 MiB |

A coluna "mudança" do `criterion` não é comparável entre execuções cujo corpus mudou; os valores
absolutos abaixo são o registro válido desta execução.

## Caminho quente (parede)

| Operação | Cenário | Medido |
|---|---|---|
| inserir 1 char no início | S0 | 528 ns |
| inserir 1 char no meio | S0 | 510 ns |
| inserir 1 char no fim | S0 | 571 ns |
| inserir 1 char no início | S1 | 969 ns |
| inserir 1 char no meio | S1 | 1,10 µs |
| inserir 1 char no fim | S1 | 984 ns |
| inserir 1 char no início | S2 | 1,80 µs |
| inserir 1 char no meio | S2 | 1,98 µs |
| inserir 1 char no fim | S2 | 1,75 µs |
| inserir 1 char no início | S3 | 81,5 µs |
| inserir 1 char no meio | S3 | 77,9 µs |
| inserir 1 char no fim | S3 | 44,2 µs |
| inserir 1 char (`apply`, 100k blocos) | S4 | 8,76 µs |
| apagar 1 char | S1 | 1,03 µs |
| apagar 1 KiB | S3 | 42,4 µs |
| `AddMark` 1 KiB | S1 | 4,32 µs |
| split de bloco | S1 | 3,13 µs |
| join de bloco | S1 | 3,22 µs |
| transação de 10 steps | S1 | 10,5 µs |
| undo | S1 | 1,00 µs |
| redo | S1 | 709 ns |
| inserir 1 char (`apply_owned`) | S1 | 963 ns |
| inserir 1 char (`apply_owned`) | S2 | 2,12 µs |

S3 é O(runs do bloco): reconstruir o vetor de 256 runs custa as cópias dos runs não tocados, não
uma varredura do documento. Reduzir isso exigiria runs compartilhados (`Arc`) com custo de
publicação; o desenho atual troca essa constante por simplicidade e `forbid(unsafe_code)`.

## Wire e validação

| Operação | Cenário | Medido | Alvo |
|---|---|---|---|
| `to_json_into` | S2 | 1,80 ms | ≤ 5 ms |
| `to_json_string` | S2 | 2,75 ms | ≤ 6 ms |
| `check_json` | S2 | 16,5 ms | — |
| `from_json` | S2 | 12,8 ms | ≤ 15 ms |
| `Doc::validate` | S2 | 3,72 ms | ≤ 10 ms |
| `normalize` de bloco | S3 | 5,53 µs | ≤ 1 µs |
| `map_pos` | S2 | 1,8 ns | ≤ 50 ns |

`normalize` mede o bloco inteiro de S3 (256 runs); o alvo de 1 µs é por bloco local pequeno.

## Acessibilidade

| Operação | Cenário | Medido |
|---|---|---|
| `a11y_tree` | S2 | 2,15 ms |
| `validate_a11y` | S2 | 2,34 ms |

## Alocações (`tests/alloc.rs`, alocador de contagem)

| Operação | Alocações |
|---|---|
| inserir 1 char (`apply`, S0) | 15 |
| inserir 1 char (`apply_owned`, S0) | 10 |
| clonar estado com 8 grupos de undo | 1 |

O teto é o dobro de folga sobre o medido, como em `plan/05` §11: `≤ 16`, `≤ 11`, `≤ 2`.

## Gate de instruções

`bench/benches/iai_hot.rs` fixa `soft_limits` de 5% em `Ir` sobre a baseline
`target/iai/main.json`; uma regressão faz o runner sair com código 3 e quebra o job.
`scripts/bench_instructions.sh` e o job `bench` do CI criam a baseline quando ela não existe e
comparam em todas as execuções seguintes.

Verificação local da baseline e do gate (2026-11-09): executada em `docker run --rm
--security-opt seccomp=unconfined -v "$PWD":/src:ro rust:1.99-bookworm`, com `valgrind` instalado na
imagem. A primeira execução gravou `main.json` e a segunda comparou os 10 benchmarks: `Iai-Callgrind
result: Ok. 10 without regressions; 0 regressed` e saída 0. O perfil seccomp padrão bloqueia
`personality()`, que o `setarch` do runner usa para desligar ASLR; sem a flag o gate falha antes de
medir.
