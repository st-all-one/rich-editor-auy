# Exemplos executados — `calc-auy`

Tudo aqui foi **rodado neste ambiente** e as saídas estão coladas como saíram.
Onde algo não foi executado, está escrito `não executado` com o motivo.

## 0. Ambiente e uma armadilha de toolchain

```
$ rustup toolchain list
stable-x86_64-unknown-linux-gnu (default)
nightly-x86_64-unknown-linux-gnu
1.92-x86_64-unknown-linux-gnu
1.98-x86_64-unknown-linux-gnu (active)
1.99-x86_64-unknown-linux-gnu
```

`rust-toolchain.toml:2` (em `ref/calc-auy/`) fixa `channel = "1.98"`. Neste
ambiente a toolchain 1.98 está instalada **sem `cargo`**:

```
$ cd ref/calc-auy && cargo metadata --offline --no-deps
error: the 'cargo' binary, normally provided by the 'cargo' component, is not applicable to the '1.98-x86_64-unknown-linux-gnu' toolchain
```

Portanto **todo** comando abaixo usa
`RUSTUP_TOOLCHAIN=1.99-x86_64-unknown-linux-gnu` e
`CARGO_TARGET_DIR=/tmp/dossie-calc-target`. `ref/` não foi modificado.

Armadilha registrada: a primeira tentativa de `cargo test --workspace`
travou por muito tempo e terminou em erro de **linker**; a causa foi
`Disk quota exceeded` no `/tmp` (tmpfs de 8,7 GB, compartilhado com outros
builds). Depois de apagar artefatos meus (`debug/examples`, `tests/`) a
execução completa passou. Isso é ambiente, não bug do projeto.

## 1. Metadados

```
$ cargo metadata --offline --no-deps --format-version 1
exit=0
packages: 13
calc-auy-core 0.1.0 MPL-2.0 8 targets [['lib'], ['example'], ['example'], ['test'], ['bench'], ['bench'], ['bench'], ['bench']]
calc-auy-macros 0.1.0 MPL-2.0 1 targets [['proc-macro']]
calc-auy-parser 0.1.0 MPL-2.0 3 targets [['lib'], ['test'], ['bench']]
calc-auy-builder 0.1.0 MPL-2.0 4 targets [['lib'], ['test'], ['test'], ['bench']]
calc-auy-output 0.1.0 MPL-2.0 7 targets [['lib'], ['test'], ['test'], ['test'], ['test'], ['test'], ['test']]
calc-auy-nlg 0.1.0 MPL-2.0 1 targets [['lib']]
calc-auy-render 0.1.0 MPL-2.0 2 targets [['lib'], ['test']]
calc-auy-i18n 0.1.0 MPL-2.0 2 targets [['lib'], ['test']]
calc-auy-processors 0.1.0 MPL-2.0 5 targets [['lib'], ['test'], ['test'], ['test'], ['custom-build']]
calc-auy-api 0.1.0 MPL-2.0 4 targets [['lib'], ['example'], ['example'], ['test']]
calc-auy-wasm 0.1.0 MPL-2.0 1 targets [['cdylib', 'rlib']]
calc-auy 0.1.0 MPL-2.0 2 targets [['bin'], ['test']]
xtask 0.1.0 MPL-2.0 1 targets [['bin']]
```

## 2. Testes

### 2.1 Execução completa

```
$ RUSTUP_TOOLCHAIN=1.99-x86_64-unknown-linux-gnu \
  CARGO_TARGET_DIR=/tmp/dossie-calc-target \
  cargo test --workspace --locked --offline
...
exit=0
passed=322 failed=0 ignored=0
41 linhas "test result:" (30 alvos de unidade/integração + 11 alvos de doctest)
Wall time: 6.61s   (tudo em cache; a primeira compilação levou minutos)
```

Não houve `warning:` nem `error:` na saída. Os **311** testes
`#[test]`/`proptest` mais **11** doctests somam 322.

### 2.2 Distribuição por alvo (os maiores)

| Alvo | Testes |
|---|---:|
| `calc-auy-core` lib | 167 |
| `calc-auy` `tests/cli.rs` | 21 |
| `calc-auy-output` lib | 16 |
| `calc-auy-parser` lib | 15 |
| `calc-auy-render` lib | 12 |
| `calc-auy-wasm` lib | 10 |
| `calc-auy-processors` `tests/codecs.rs` | 8 |
| `calc-auy-i18n` `tests/realizers.rs` | 7 |
| `calc-auy-api` `tests/corpus.rs` | 6 |
| `calc-auy-builder` `tests/e2e.rs` | 5 |
| `calc-auy-output` `tests/roots.rs` / `snapshots.rs` | 5 + 5 |
| `calc-auy-render` `tests/render.rs` | 4 |
| `calc-auy-builder`/`calc-auy-core` `compile_fail.rs` (trybuild) | 1 + 1 |
| doctests (`api`, `builder`, `core`, `i18n`, `output`, `render`) | 1+1+1+1+3+4 |

### 2.3 Teste isolado (verificação cruzada)

```
$ cargo test --offline --locked -p calc-auy-core --lib
test result: ok. 167 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.64s
```

### 2.4 O doctest que falhou e depois passou

A primeira execução do workspace falhou em **um** doctest
(`crates/calc-auy-api/src/lib.rs - CalcAuy (line 34)`) com
`error: linking with 'cc' failed`. Repetindo isolado, a mensagem real era:

```
error: failed to write to `/tmp/dossie-calc-target/debug/deps/rmetaF3cqTF/full.rmeta`: Disk quota exceeded (os error 122)
```

Depois de liberar espaço, `cargo test -p calc-auy-api --doc`:

```
running 1 test
test crates/calc-auy-api/src/lib.rs - CalcAuy (line 34) ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Ou seja: **nenhuma falha de teste real**; a falha era de disco.

## 3. O binário, com saída real

Binário: `/tmp/dossie-calc-target/debug/calc-auy` (`--version` →
`calc-auy 0.1.0`).

```
$ calc-auy "1 + 2"
3.00
exit=0

$ calc-auy --params '{"mode":"monetary","locale":"pt-BR"}' "10000 * (1 + 14.75%) ^ 3"
R$ 15.109,78
exit=0

$ printf '1/3' | calc-auy --params '{"rounding":"half_up","decimals":4}'
0.3333
exit=0

$ calc-auy --params '{"mode":"unicode"}' "9^(1/2)"
roundₙᵦᵣ₋₅₈₉₁(√9, 2) = 3.00
exit=0

$ calc-auy --params '{"mode":"verbal","locale":"ja-JP"}' "1 + 2"
式 一 足す 二 は 三 です。丸め：NBR-5891、小数第 2 位。
exit=0
```

Observação que vale registrar: **todo** formato textual (menos o decimal
"cru") envolve o resultado na anotação de arredondamento
`round_{...}(expr, casas) = valor`; o modo `decimal` imprime só o número.

Nota de fidelidade de bytes: o monetário separa `R$` do número com **U+00A0**
(NBSP), não espaço ASCII (`od -c` → `R $ 302 240 1 5 . 1 0 9 , 7 8`). Nos
blocos deste arquivo o NBSP aparece como espaço comum — inclusive no `result`
do lote (§3.5) e nas saídas do harness e do `json` (§4).

### 3.1 JSON multi-formato

```
$ calc-auy --params '{"mode":"json"}' "1 + 2" | wc -c
1175
$ calc-auy --params '{"mode":"json"}' "1 + 2" | python3 -c 'import json,sys;print(sorted(json.load(sys.stdin)))'
['auditTrace', 'decimal', 'latex', 'mermaid', 'monetary', 'scaled', 'signature', 'unicode', 'verbal']
```

Primeiros 330 caracteres (truncado de propósito; 1175 bytes no total):

```
{"auditTrace":"{\"payload\":{\"ast\":{\"kind\":\"operation\",\"op_type\":\"add\",\"operands\":[{\"kind\":\"literal\",\"value\":{\"n\":\"1\",\"d\":\"1\"},\"original_input\":\"1\"},{\"kind\":\"literal\",\"value\":{\"n\":\"2\",\"d\":\"1\"},\"original_input\":\"2\"}]},\"final_result\":{\"n\":\"3\",\"d\":\"1\"},\"round_strategy\":\"n
```

### 3.2 Assinatura, traço e verificação

```
$ calc-auy --params '{"op":"sign"}' "7 * 6"
82b1836d6e6ae436485fefe8a83b3c365d55b414347fb22326d6f14aa7c2724d
exit=0

$ calc-auy --params '{"op":"commit"}' "1 + 2"
{"payload":{"ast":{"kind":"operation","op_type":"add","operands":[{"kind":"literal","value":{"n":"1","d":"1"},"original_input":"1"},{"kind":"literal","value":{"n":"2","d":"1"},"original_input":"2"}]},"final_result":{"n":"3","d":"1"},"round_strategy":"nbr5891","trace_version":1},"signature":"3b051ce99eb6f435f15d4b6e3917df3e05a95322e3ea3b7d2eaaf938041191c5"}
exit=0

$ T=$(calc-auy --params '{"op":"commit"}' "1 + 2")
$ echo "$T" | calc-auy verify
true
exit=0

$ echo "$T" | sed 's/"n":"3"/"n":"4"/' | calc-auy verify
false
erro: violação de integridade: esperado 3b051ce99eb6f435f15d4b6e3917df3e05a95322e3ea3b7d2eaaf938041191c5, recebido 656e2f73a2860e20048bfb27f85ce66d8a5ca345b02ef94e5634cc1ed754c349
exit=6
```

Dois `sign` seguidos da mesma expressão produzem o **mesmo** digest (o teste
`params_op_sign_is_deterministic` faz exatamente isso, `tests/cli.rs:134-140`).

### 3.3 Códigos de saída observados

| Comando | `stderr`/`stdout` | exit |
|---|---|---:|
| `calc-auy "1/0"` | `erro: divisão por zero` | 5 |
| `calc-auy "1 +"` | `erro: parse error: operando esperado (byte 3..3)` | 3 |
| `calc-auy "2^(1/2)"` | `erro: resultado irracional: 2^1/2` | 7 |
| `calc-auy "2^100000000"` | `erro: overflow matemático: 200000000 bits excede limite de 1000000` | 4 |
| traço adulterado + `verify` | `false` + erro de integridade | 6 |
| `--params '{}' --batch` | `erro: --params e --batch são mutuamente exclusivos` | 2 |
| `--params '{"foo":1}'` | erro citando `foo` | 2 |

### 3.4 `ProblemDetails` reprodutível

```
$ calc-auy --params '{"error_format":"json","deterministic":true}' "1/0"
{"type":"https://calc-auy.dev/errors/division-by-zero","title":"Divisão por zero","status":400,"detail":"A operação tentou dividir por zero.","instance":"00000000-0000-0700-0000-000000000000"}
exit=5
$ (mesmo comando de novo)
{"type":"...","instance":"00000000-0000-0700-0000-000000000000"}
exit=5
```

O `instance` é idêntico porque a CLI troca `SystemClock` por `MockClock`
quando `deterministic: true` (`crates/calc-auy/src/main.rs:65-72`).

### 3.5 Lote NDJSON (`lote.ndjson` deste diretório)

```
$ calc-auy --batch < lote.ndjson
{"id":1,"ok":true,"result":"2.00"}
{"id":2,"ok":false,"problem":{"detail":"A operação tentou dividir por zero.","instance":"00000000-0000-0700-0000-000000000000","status":400,"title":"Divisão por zero","type":"https://calc-auy.dev/errors/division-by-zero"}}
{"id":3,"ok":true,"result":"R$ 15.109,78"}
{"id":4,"ok":true,"result":"roundₙᵦᵣ₋₅₈₉₁(√9, 2) = 3.00"}
exit=0
```

Ordem preservada, linhas com erro não interrompem o lote e o exit é 0
(o erro viaja no corpo).

### 3.6 Arquivo de configuração (`calc-auy.conf`, análogo ao `.akrc`)

```
$ CALC_AUY_CONFIG=./calc-auy.conf calc-auy "1/3"
0.3333          # decimals = 4, rounding = half_up

$ CALC_AUY_CONFIG=./calc-auy.conf CALC_AUY_DECIMALS=2 calc-auy "1/3"
0.33            # ambiente vence o arquivo

$ CALC_AUY_CONFIG=./calc-auy.conf CALC_AUY_DECIMALS=2 calc-auy --params '{"decimals":6}' "1/3"
0.333333        # --params vence o ambiente
```

Isso confirma a precedência documentada em `crates/calc-auy/src/config.rs:1-5`.

### 3.7 `hydrate`, `schema`, `man`

```
$ echo "$AST_JSON" | calc-auy hydrate
{"kind":"operation","op_type":"add","operands":[{"kind":"literal","value":{"n":"1","d":"1"},"original_input":"1"},{"kind":"literal","value":{"n":"2","d":"1"},"original_input":"2"}]}
exit=0

$ calc-auy schema
{ "name": "calc-auy", "version": "0.1.0", "trace_version": 1, "payload": "SignaturePayload",
  "operations": ["format","commit","sign"],
  "modes": ["decimal","monetary","latex","unicode","mermaid","scaled","verbal"],
  "locales": ["pt-BR","en-US","en-EU","fr-FR","de-DE","es-ES","ru-RU","zh-CN","ja-JP"],
  "rounding": ["nbr5891","half_up","half_even","truncate","ceil","none"],
  "error_format": ["human","json"],
  "subcommands": ["verify","hydrate","repl","schema","man"] }
exit=0
```

O campo `modes` do `schema` **omite** `json`, `html` e `svg`, embora o binário
os execute (§3.8). É introspecção incompleta (`commands.rs:316-331`).

```
$ calc-auy man | head -2
.TH CALC-AUY 1 "" "calc-auy"
.SH NAME
$ calc-auy man --completions bash | head -2
# completions para bash: eval "$(calc-auy man --completions bash)"
_calc_auy() {
```

### 3.8 HTML e SVG

```
$ calc-auy --params '{"mode":"html"}' "1/2 + 3" | head -c 300
<div class="calc-auy-result" aria-label="um dividido por dois mais três é igual a três vírgula cinco. Arredondamento: NBR-5891 para 2 casas decimais."><style>/* KaTeX v0.16.45 stylesheet with woff2 fonts embedded as data URIs.
 * Copyright (c) 2026 Khan Academy and other contributors. MIT Licens
```

O `aria-label` é a **descrição verbal real** produzida pelo realizer pt-BR —
é a evidência mais direta de acessibilidade no projeto.

```
$ calc-auy --params '{"mode":"svg"}' "1/2 + 3" | head -c 300
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="152.5196pt" height="23.028pt" viewBox="0 0 152.5196 23.028">
<g fill="#000000">
<path d="M 183.0 482.0 L 159.0 482.0 L 35.0 452.0 L 35.0 424.0 C 35.0 424.0 46.0 426.0 66.0 426.0 C 102.0 426.0 102.0 408.0 102.0 372.
```

O SVG **não** tem `<title>`, `<desc>`, `role` nem `aria-label`: nenhuma
informação textual para leitor de tela. O contraste com o HTML é o achado de
a11y do dossiê.

## 4. Harness Rust da API (`src/main.rs`)

```
$ cd dossie/calc-auy/exemplos
$ RUSTUP_TOOLCHAIN=1.99-x86_64-unknown-linux-gnu \
  CARGO_TARGET_DIR=/tmp/dossie-calc-target cargo run --offline
   Compiling calc-auy-core v0.1.0 (...)   # … 8 crates …
   Compiling calc-auy-dossie-exemplo v0.0.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.73s
     Running `/tmp/dossie-calc-target/debug/calc-auy-dossie-exemplo`
expressao    = 10000 * (1 + 14.75%) ^ 3
decimal      = 15109.78
monetario    = R$ 15.109,78
latex        = \text{round}_{\text{NBR-5891}}(10000 \times \left( 1 + 14.75 \times \frac{1}{100} \right)^{3}, 2) = 15109.78
assinatura   = 70671996017674ecca1131f4d434fd7adc0d11f89b01f7c9711c13727510c831
determinista = true
rascunho     = {"kind":"operation","op_type":"add","operands":[{"kind":"operation","op_type":"div","operands":[{"kind":"literal","value":{"n":"1","d":"1"},"original_input":"1"},{"kind":"literal","value":{"n":"3","d":"1"},"original_input":"3"}]},{"kind":"literal","value":{"n":"1","d":"1"},"original_input":"1"}]}
reidratado   = 1.33
json         = {"decimal":"15109.78","monetary":"R$ 15.109,78","signature":"70671996017674ecca1131f4d434fd7adc0d11f89b01f7c9711c13727510c831","verbal":"dez mil multiplicado por abre parênteses um mais quatorze vírgula setenta e cinco multiplicado por um sobre cem fecha parênteses elevado a três é igual a quinze mil cento e nove vírgula setenta e oito. Arredondamento: NBR-5891 para 2 casas decimais."}
erro.type    = https://calc-auy.dev/errors/division-by-zero
erro.status  = 400
erro.detail  = A operação tentou dividir por zero.
```

O harness cobre, em 76 linhas: `parse` → `commit`, determinismo de assinatura,
`hibernate`/`hydrate` de rascunho, `to_json_formats` com realizer e conversão
de erro em `ProblemDetails` via `MockClock`.

Repetido depois com `--locked`: compilou e produziu a **mesma** assinatura
`70671996017674ecca1131f4d434fd7adc0d11f89b01f7c9711c13727510c831` — é a
verificação de determinismo entre processos, não só dentro do mesmo processo
(o teste `params_op_sign_is_deterministic` faz o mesmo via CLI).

## 5. O que **não** foi executado

| Item | Estado | Motivo |
|---|---|---|
| `cargo clippy --workspace -- -D warnings` | não executado | custo de rebuild completo; lints já são `deny` no manifesto (`Cargo.toml:100-160`) |
| `cargo fmt --check` | não executado | irrelevante para o dossiê |
| `cargo doc` / `RUSTDOCFLAGS=-D warnings` | não executado | idem; coberto pelo CI declarado |
| `cargo deny` / `cargo audit` | não executado | ferramentas não instaladas |
| `cargo bench` / `cargo run -p xtask -- bench-report` | não executado | release build + tempo; números do projeto estão em `plan/evidence/bench/` e são citados **como do projeto** |
| fuzz (`cargo fuzz run parse`) | não executado | `cargo-fuzz` indisponível; `Makefile:45-48` já prevê o fallback |
| Miri | não executado | exige nightly + `-Zmiri-disable-isolation`; job é `continue-on-error` no CI |
| build `--target wasm32-unknown-unknown` | não executado | target não instalado neste ambiente |
| MSRV 1.98 | **não verificável aqui** | a toolchain 1.98 local não tem `cargo` (§0) |
| `--features instrument` (diag) | não executado | feature opcional; código lido em `diag/mod.rs` |

O que essas lacunas custam: performance **medida**, MSRV real e fuzz ficam
como afirmações do projeto, não observações minhas. Tudo o mais neste dossiê
tem execução ou `caminho:linha`.
