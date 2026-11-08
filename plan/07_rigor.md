# 07 — Rigor: toolchain, lints, testes e CI

> **Regra.** Rigor é configuração executável, não intenção. Cada regra abaixo tem um lugar, um
> nível e um comando que falha. Se uma regra não pode ser executada pelo compilador, pelo clippy,
> por um teste ou pelo CI, ela não entra.

---

## 1. Dois números, quatro lugares

| Número | Onde | Valor | Se divergir |
|---|---|---|---|
| **Pin de desenvolvimento** | `rust-toolchain.toml` → `channel` | **1.99** + `clippy` + `rustfmt` | máquina e CI compilam com compiladores diferentes no dia a dia |
| **MSRV** | `Cargo.toml` → `rust-version` | **1.92** | a MSRV declarada é ficção |
| **MSRV** | `clippy.toml` → `msrv` | **1.92** | o clippy sugere API acima da MSRV |
| **MSRV** | job de CI `msrv` | **1.92** | a MSRV não é testada de verdade |

Regra: **o pin é local ao repositório; a MSRV é o contrato.** O pin pode subir com a stable; a MSRV
só sobe em major (ou em decisão explícita registrada). `resolver = "3"` faz a resolução respeitar
`rust-version`.

### 1.1 `rust-toolchain.toml`

```toml
[toolchain]
channel = "1.99"
components = ["clippy", "rustfmt"]
targets = ["wasm32-unknown-unknown"]
profile = "minimal"
```

O target WASM entra no pin porque o crate `rich-editor-auy-wasm` é entregável oficial (F31), não
um experimento. O job de MSRV instala o target separadamente quando testar o artefato.

### 1.2 APIs acima da MSRV — denylist inicial

O clippy 1.99 conhece várias APIs estabilizadas depois de 1.92, mas a lista dele não é exaustiva.
A denylist é uma lista de APIs que **não podem aparecer** no código, verificada por revisão + CI na
MSRV:

| API | Estabilizada em | Por que evitar |
|---|---|---|
| `format_into` / `NumBuffer` | 1.98 | formatação sem alocação; reimplementar com buffer `[u8; 20]` se necessário |
| `f64::algebraic_*` / `f32::algebraic_*` | 1.98 | não determinístico; proibido de qualquer forma |
| `VaList` | 1.99 | FFI; fora do escopo |
| `Box::into_non_null` | 1.96 (candidato) | não precisamos |
| `Vec::into_parts` | 1.93 (candidato) | não precisamos |
| `String::from_utf8_lossy_owned` | 1.96 (candidato) | não precisamos |

Os itens marcados "candidato" são verificados no primeiro build em 1.92; se compilarem, saem da
denylist com uma nota. A prova final é o job `msrv`, não a tabela.

## 2. `rustfmt.toml`

```toml
edition = "2024"
style_edition = "2024"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
hex_literal_case = "Lower"
use_field_init_shorthand = true
use_try_shorthand = true
```

Só opções **estáveis**. Opção unstable em toolchain estável é ignorada com aviso e dá a falsa
impressão de padrão configurado. Agrupar imports é manual (a opção é unstable); a ordem é a do
rustfmt.

## 3. `Cargo.toml` — workspace, lints e perfis

```toml
[workspace]
resolver = "3"
members = [
    "crates/rich-editor-auy",
    "crates/rich-editor-auy-macros",
    "crates/rich-editor-auy-wasm",
    "crates/rich-editor-auy-testkit",
]
default-members = ["crates/rich-editor-auy"]
exclude = ["fuzz", "bench"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.92"
license = "MIT OR Apache-2.0"
repository = "https://github.com/st-all-one/rich-editor-auy"
homepage = "https://github.com/st-all-one/rich-editor-auy"
authors = ["st-all-one"]
# `publish` não é herdado: cada crate publicável fica no default (`publish = true`) e cada crate
# interno declara `publish = false`.

[workspace.dependencies]
blake3 = { version = "1.8", default-features = false, features = ["pure"] }
wasm-bindgen = "0.2"
proptest = "1"          # dev
serde_json = "1"        # dev, só para teste diferencial do parser
trybuild = "1"          # dev, macro `schema!`

[workspace.lints.rust]
# `deny` no workspace permite o glue do wasm/testkit com `#[allow]` + reason; o crate do core
# declara `#![forbid(unsafe_code)]` e não pode ser revertido localmente.
unsafe_code = "deny"
missing_docs = "deny"
unused_qualifications = "deny"
unused_lifetimes = "deny"
elided_lifetimes_in_paths = "deny"
single_use_lifetimes = "deny"
trivial_casts = "deny"
trivial_numeric_casts = "deny"
unreachable_pub = "deny"
unsafe_op_in_unsafe_fn = "deny"
let_underscore_drop = "deny"
macro_use_extern_crate = "deny"
meta_variable_misuse = "deny"
non_ascii_idents = "deny"
noop_method_call = "deny"
unused_import_braces = "deny"
variant_size_differences = "warn"

[workspace.lints.rustdoc]
broken_intra_doc_links = "deny"
unused_footnote_definition = "deny"

[workspace.lints.clippy]
# Grupos
all = { level = "deny", priority = -1 }
correctness = { level = "deny", priority = -1 }
suspicious = { level = "deny", priority = -1 }
style = { level = "deny", priority = -1 }
complexity = { level = "deny", priority = -1 }
perf = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
cargo = { level = "warn", priority = -1 }

# Restriction: o que não pode existir
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
unreachable = "deny"
dbg_macro = "deny"
print_stdout = "deny"
print_stderr = "deny"
exit = "deny"
mem_forget = "deny"
indexing_slicing = "deny"
arithmetic_side_effects = "deny"
as_conversions = "deny"
cast_possible_truncation = "deny"
cast_sign_loss = "deny"
cast_possible_wrap = "deny"
cast_precision_loss = "deny"
undocumented_unsafe_blocks = "deny"
multiple_unsafe_ops_per_block = "deny"
let_underscore_must_use = "deny"
let_underscore_untyped = "warn"
absolute_paths = "warn"
wildcard_imports = "deny"
fn_params_excessive_bools = "deny"
struct_excessive_bools = "deny"
disallowed_fields = "deny"
disallowed_macros = "deny"
disallowed_methods = "deny"
disallowed_names = "deny"
disallowed_types = "deny"
await_holding_lock = "deny"
await_holding_refcell_ref = "deny"
allow_attributes = "allow"
allow_attributes_without_reason = "deny"
large_enum_variant = "deny"
large_stack_arrays = "deny"
result_large_err = "warn"
large_types_passed_by_value = "warn"
large_stack_frames = "warn"
large_futures = "warn"

# Ruidosas: allow documentado
module_name_repetitions = "allow"
must_use_candidate = "allow"
implicit_hasher = "allow"
min_ident_chars = "allow"
many_single_char_names = "allow"
missing_docs_in_private_items = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
option_if_let_else = "allow"
significant_drop_tightening = "allow"
missing_const_for_fn = "allow"
# `excessive_nesting` NÃO é desligado: o limiar de `clippy.toml` é o gate; exceção local com reason.
assert_is_empty = "allow"   # o idioma do projeto em teste é assert!(x.is_empty())

[profile.release]
overflow-checks = true
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"

[profile.dev]
overflow-checks = true
debug-assertions = true
```

**Cada crate precisa de `[lints] workspace = true`**, senão herda o default e o gate vira ficção.
O core publicado é `#![no_std]` + `extern crate alloc`; `rich-editor-auy-wasm` pode usar `std` e o
glue do `wasm-bindgen`, mas continua separado do core. O crate de teste pode usar `std`.

### 3.1 Exceção de instrumentação (não é o core)

`forbid(unsafe_code)` vale para o **core publicado**. O `rich-editor-auy-testkit` precisa de
`unsafe` para instalar um `#[global_allocator]` de contagem; ele não é publicado como parte do core
e tem configuração de lint própria (`deny(unsafe_code)` com um `#[allow(unsafe_code)]` local e
comentário `SAFETY`, ou uma dependência vetada que encapsula o alocador). `fuzz/` e `bench/` são
workspaces excluídos e também podem usar `unsafe` no harness. `crates/rich-editor-auy-wasm` é um
crate separado (não é o core publicado); se a expansão do `wasm-bindgen` exigir `unsafe`, ele fica
confinado ao glue gerado/`SAFETY`, nunca ao core. A regra que não muda: **nenhuma linha do crate
publicado pode conter `unsafe`**.

## 4. `clippy.toml` — valores

```toml
msrv = "1.92"

# ---------------------------------------------------------------- unsafe ----
accept-comment-above-attributes = false
accept-comment-above-statement = false

# ------------------------------------------------------- unwrap/expect ----
allow-expect-in-consts = false
allow-expect-in-tests = false
allow-unwrap-in-consts = false
allow-unwrap-in-tests = false
allow-panic-in-tests = false
allow-unwrap-types = []

# ---------------------------------------------------------------- testes ----
allow-dbg-in-tests = false
allow-print-in-tests = false
allow-useless-vec-in-tests = false
allow-indexing-slicing-in-tests = false
allow-large-stack-frames-in-tests = true

# ----------------------------------------------------------- estilo ----
allow-mixed-uninlined-format-args = false
allow-one-hash-in-raw-strings = false
unreadable-literal-lint-fractions = true
upper-case-acronyms-aggressive = true
enforce-iter-loop-reborrow = true
lint-commented-code = true
module-items-ordered-within-groupings = "all"
warn-on-all-wildcard-imports = true
allowed-wildcard-imports = []

# ------------------------------------------------------- complexidade ----
cognitive-complexity-threshold = 15
type-complexity-threshold = 150
too-many-lines-threshold = 60
too-many-arguments-threshold = 5
excessive-nesting-threshold = 3
max-trait-bounds = 3
max-suggested-slice-pattern-length = 3
single-char-binding-names-threshold = 2
min-ident-chars-threshold = 2
allowed-idents-below-min-chars = [
  "i", "j", "x", "y", "z", "n", "id", "ok", "err", "pos", "len", "..",
]

# -------------------------------------------------------------- tamanhos ----
max-fn-params-bools = 0
max-struct-bools = 1
enum-variant-size-threshold = 128
enum-variant-name-threshold = 3
struct-field-name-threshold = 3
large-error-threshold = 128
unnecessary-box-size = 128
pass-by-value-size-limit = 128
array-size-threshold = 4096
stack-size-threshold = 65536
future-size-threshold = 8192
max-include-file-size = 262144
vec-box-size-threshold = 4096
too-large-for-stack = 200

# ---------------------------------------------------------------- MSRV ----
avoid-breaking-exported-api = false
check-incompatible-msrv-in-tests = true
check-private-items = true
missing-docs-allow-unused = false
missing-docs-in-crate-items = false
inherent-impl-lint-scope = "crate"
recursive-self-in-type-definitions = true
enable-raw-pointer-heuristic-for-send = true
pub-underscore-fields-behavior = "PubliclyExported"

# ------------------------------------------------------- determinismo ----
await-holding-invalid-types = [
  "std::sync::MutexGuard",
  "std::sync::RwLockReadGuard",
  "std::sync::RwLockWriteGuard",
  "core::cell::Ref",
  "core::cell::RefMut",
  "alloc::cell::Ref",
  "alloc::cell::RefMut",
  "parking_lot::MutexGuard",
]

# ---------------------------------------------------------- aritmética ----
# Sem `f32`/`f64`: float não entra no modelo, no wire nem no hash (F14). Só operações explícitas
# de wrapping/saturating e atômicos (que não são caminho de documento).
arithmetic-side-effects-allowed = [
  "Wrapping", "Saturating",
  "AtomicU8", "AtomicU16", "AtomicU32", "AtomicU64", "AtomicUsize",
  "AtomicI8", "AtomicI16", "AtomicI32", "AtomicI64", "AtomicIsize",
]
arithmetic-side-effects-allowed-binary = []
arithmetic-side-effects-allowed-unary = []
suppress-restriction-lint-in-const = false
verbose-bit-mask-threshold = 1
const-literal-digits-threshold = 30
literal-representation-threshold = 16384

# ------------------------------------------------------- proibições ----
disallowed-names = ["foo", "baz", "quux", "tmp", "thing", "stuff", "misc", ".."]

disallowed-types = [
  { path = "alloc::rc::Rc", reason = "não é Send/Sync; use Arc (F12)" },
  { path = "std::rc::Rc", reason = "não é Send/Sync; use Arc (F12)" },
  { path = "alloc::rc::Weak", reason = "não é Send/Sync (F12)" },
  { path = "std::rc::Weak", reason = "não é Send/Sync (F12)" },
  { path = "core::cell::RefCell", reason = "mutabilidade interior; use valor imutável (F12)" },
  { path = "std::cell::RefCell", reason = "mutabilidade interior; use valor imutável (F12)" },
  { path = "core::cell::Cell", reason = "estado mutável compartilhado; use valor imutável (F12)" },
  { path = "std::cell::Cell", reason = "estado mutável compartilhado; use valor imutável (F12)" },
  { path = "alloc::collections::LinkedList", reason = "cache-hostil; use Vec" },
  { path = "std::collections::LinkedList", reason = "cache-hostil; use Vec" },
  { path = "std::collections::HashMap", reason = "iteração não determinística; use BTreeMap/Vec (F12)" },
  { path = "std::collections::HashSet", reason = "iteração não determinística; use BTreeSet/Vec (F12)" },
]

disallowed-methods = [
  { path = "std::mem::transmute", reason = "UB potencial; não há unsafe (F10)" },
  { path = "std::mem::forget", reason = "vazamento; use RAII (F12)" },
  { path = "std::mem::zeroed", reason = "valor inválido possível; use Default" },
  { path = "std::mem::uninitialized", reason = "deprecado e UB; use MaybeUninit" },
  { path = "std::slice::from_raw_parts", reason = "unsafe; proibido (F10)" },
  { path = "std::slice::from_raw_parts_mut", reason = "unsafe; proibido (F10)" },
  { path = "std::ptr::read", reason = "unsafe; proibido (F10)" },
  { path = "std::ptr::write", reason = "unsafe; proibido (F10)" },
  { path = "std::env::var", reason = "não determinístico; entre como dado (F13)" },
  { path = "std::env::var_os", reason = "não determinístico; entre como dado (F13)" },
  { path = "std::env::vars", reason = "não determinístico; entre como dado (F13)" },
  { path = "std::env::vars_os", reason = "não determinístico; entre como dado (F13)" },
  { path = "std::env::set_var", reason = "unsafe na Edition 2024; proibido (F13)" },
  { path = "std::env::remove_var", reason = "unsafe na Edition 2024; proibido (F13)" },
  { path = "std::time::SystemTime::now", reason = "não determinístico; entre como dado (F13)" },
  { path = "std::time::Instant::now", reason = "não determinístico; entre como dado (F13)" },
  { path = "std::process::exit", reason = "propague Result; o core não encerra processo (F13)" },
]

disallowed-macros = [
  { path = "std::dbg", reason = "remova antes do commit" },
  { path = "std::todo", reason = "inacabado; abra uma decisão" },
  { path = "std::unimplemented", reason = "inacabado; retorne erro" },
  { path = "std::unreachable", reason = "prove a impossibilidade sem panic (F11)" },
  { path = "std::panic", reason = "propague Result (F11)" },
]

disallowed-fields = []

# ------------------------------------------------------- documentação ----
doc-valid-idents = [
  "rich-editor-auy", "auy", "ProseMirror", "Yjs", "Loro", "CRDT", "OT",
  "UTF-8", "UTF-16", "ASCII", "Unicode", "NFC", "NFD", "MSRV", "API", "JSON",
  "HTML", "Markdown", "COW", "WASM", "IME", "ARIA", "DOM", "CSS", "WCAG",
  "BTreeMap", "BTreeSet", "Arc", "Pos", "Doc", "Slice", "StepMap", "Mapping",
  "ChangeSet", "MarkSet", "TextBlock", "InlineItem", "NodeSpec", "MarkSpec",
  "BLAKE3", "Ed25519", "JSR", "Deno", "TypeScript", "PHP", "Composer",
  "Packagist", "wasm32", "docs.rs", "rustdoc", "fuzz", "proptest",
  "..",
]

standard-macro-braces = [
  { name = "vec", brace = "[" },
  { name = "format", brace = "(" },
  { name = "write", brace = "(" },
  { name = "writeln", brace = "(" },
  { name = "assert", brace = "(" },
  { name = "assert_eq", brace = "(" },
  { name = "assert_ne", brace = "(" },
  { name = "debug_assert", brace = "(" },
  { name = "matches", brace = "(" },
]

warn-unsafe-macro-metavars-in-private-macros = true
absolute-paths-allowed-crates = ["std", "core", "alloc", "crate"]
absolute-paths-max-segments = 3
allow-comparison-to-zero = true
allow-exact-repetitions = true
allowed-dotfiles = []
allowed-duplicate-crates = []
allowed-prefixes = ["to", "as", "into", "from", "try_into", "try_from"]
allowed-scripts = ["Latin"]
```

## 5. `deny.toml`

```toml
[advisories]
version = 2
yanked = "deny"
ignore = []

[licenses]
version = 2
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "Zlib",
    "BSL-1.0",
    "CC0-1.0",
    "MPL-2.0",
]
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
wildcards = "deny"
allow-wildcard-paths = true
# O core é no_std + alloc e não faz I/O; runtime/framework HTTP não pode entrar. RNG e crypto
# nativa também ficam fora: o hash é BLAKE3 (`pure`) e a assinatura é Ed25519 em crate separado.
deny = [
    "tokio",
    "reqwest",
    "hyper",
    "axum",
    "rand",
    "getrandom",
    "openssl",
    "ring",
    "native-tls",
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

## 6. `Makefile` — o gate único

```make
CARGO ?= cargo
DENO ?= deno
COMPOSER ?= composer

.PHONY: check check-ecosystem fmt clippy test doc file-length links zero-style a11y package \
        wasm js php msrv deny audit machete miri fuzz bench ci

check: fmt clippy test doc file-length links zero-style a11y package
	@echo "check: verde"

check-ecosystem: wasm js php
	@echo "check-ecosystem: verde"

fmt:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

test:
	$(CARGO) test --workspace --locked

doc:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --no-deps
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --no-deps --all-features

file-length:
	./scripts/check_file_length.sh

links:
	python3 scripts/check_links.py

zero-style:
	./scripts/check_zero_style.sh

a11y:
	$(CARGO) test -p rich-editor-auy --locked a11y

package:
	$(CARGO) package -p rich-editor-auy --locked
	$(CARGO) package -p rich-editor-auy-macros --locked
	$(CARGO) package -p rich-editor-auy-wasm --locked

wasm:
	$(CARGO) build -p rich-editor-auy-wasm --target wasm32-unknown-unknown --release --locked
	./scripts/build_wasm.sh

js:
	cd packages/ts && $(DENO) task check && $(DENO) task publish:dry

php:
	cd packages/php && $(COMPOSER) install --no-interaction --prefer-dist
	cd packages/php && $(COMPOSER) validate --strict && $(COMPOSER) audit
	cd packages/php && vendor/bin/phpunit && vendor/bin/phpstan analyse

msrv:
	RUSTUP_TOOLCHAIN=1.92 $(CARGO) build --workspace --locked
	RUSTUP_TOOLCHAIN=1.92 $(CARGO) test --workspace --lib --locked

deny:
	@if command -v cargo-deny >/dev/null 2>&1; then $(CARGO) deny check; else echo "cargo-deny ausente"; fi

audit:
	@if command -v cargo-audit >/dev/null 2>&1; then $(CARGO) audit; else echo "cargo-audit ausente"; fi

machete:
	@if command -v cargo-machete >/dev/null 2>&1; then $(CARGO) machete; else echo "cargo-machete ausente"; fi

miri:
	@if command -v cargo-miri >/dev/null 2>&1; then \
	  PROPTEST_DISABLE_FAILURE_PERSISTENCE=1 \
	  MIRIFLAGS="-Zmiri-disable-isolation" \
	  $(CARGO) +nightly miri test -p rich-editor-auy --test miri_smoke; \
	else echo "miri ausente"; fi

fuzz:
	@if command -v cargo-fuzz >/dev/null 2>&1; then $(CARGO) +nightly fuzz build; else echo "cargo-fuzz ausente"; fi

bench:
	$(CARGO) bench -p rich-editor-auy --bench hot
	$(CARGO) bench -p rich-editor-auy --bench wire

ci: check check-ecosystem msrv deny audit machete miri fuzz bench
```

`make check` é a condição de "terminei" para o core; `make check-ecosystem` valida os subprojetos
oficiais (WASM/JSR/PHP). O release roda os dois. O CI repete o gate e adiciona MSRV, supply chain,
Miri, fuzz e os benches determinísticos. Nenhum alvo "conserta" o código; eles apenas verificam.

## 7. Gate de tamanho de arquivo

`scripts/check_file_length.sh`:

```sh
#!/usr/bin/env sh
set -eu
limit=300
status=0
for f in $(find crates -path '*/src/*' -name '*.rs' -type f); do
  lines=$(wc -l < "$f")
  if [ "$lines" -gt "$limit" ]; then
    echo "file-length: $f tem $lines linhas (limite $limit)" >&2
    status=1
  fi
done
exit "$status"
```

Exceções: tabelas geradas (se houver) e arquivos de teste não contam; a exceção é declarada no
cabeçalho do arquivo e justificada no PR. O limite é uma **força de modularização**, não uma meta
estética.

## 8. CI

```yaml
name: ci

on:
  push:
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -D warnings

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.99"
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets --all-features -- -D warnings
      - run: cargo test --workspace --locked
      - run: cargo build --workspace --no-default-features --locked
      - run: RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
      - run: RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --all-features
      - run: ./scripts/check_file_length.sh
      - run: python3 scripts/check_links.py
      - run: ./scripts/check_zero_style.sh
      - run: cargo test -p rich-editor-auy --locked a11y
      - run: cargo package -p rich-editor-auy --locked
      - run: cargo package -p rich-editor-auy-macros --locked
      - run: cargo package -p rich-editor-auy-wasm --locked

  msrv:
    runs-on: ubuntu-latest
    env:
      RUSTUP_TOOLCHAIN: "1.92"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.92"
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --workspace --locked
      - run: cargo test --workspace --lib --locked

  supply-chain:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.99"
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-deny,cargo-audit,cargo-machete
      - run: cargo deny check
      - run: cargo audit
      - run: cargo machete

  miri:
    runs-on: ubuntu-latest
    env:
      RUSTUP_TOOLCHAIN: nightly
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: miri
      - uses: Swatinem/rust-cache@v2
      - run: cargo miri test -p rich-editor-auy --test miri_smoke

  fuzz:
    runs-on: ubuntu-latest
    env:
      RUSTUP_TOOLCHAIN: nightly
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-fuzz
      - run: cargo fuzz run json_parse -- -max_total_time=30
      - run: cargo fuzz run apply_step -- -max_total_time=30
      - run: cargo fuzz run a11y_tree -- -max_total_time=30

  wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.99"
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with:
          tool: wasm-bindgen-cli,wasm-opt
      - run: cargo build -p rich-editor-auy-wasm --target wasm32-unknown-unknown --release --locked
      - run: ./scripts/build_wasm.sh

  js:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: packages/ts
    steps:
      - uses: actions/checkout@v4
      - uses: denoland/setup-deno@v2
        with:
          deno-version: "2.x"
      - run: deno task check
      - run: deno task publish:dry

  php:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: packages/php
    steps:
      - uses: actions/checkout@v4
      - uses: shivammathur/setup-php@v2
        with:
          php-version: "8.3"
          tools: composer, phpstan
      - run: composer install --no-interaction --prefer-dist
      - run: composer validate --strict
      - run: composer audit
      - run: vendor/bin/phpunit
      - run: vendor/bin/phpstan analyse

  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.99"
      - uses: taiki-e/install-action@v2
        with:
          tool: iai-callgrind
      - run: cargo bench -p rich-editor-auy --bench iai_hot
```

Cada job fixa `RUSTUP_TOOLCHAIN` quando precisa ignorar o pin: sem isso, o `rust-toolchain.toml`
(1.99) venceria o toolchain instalado e o job testaria o compilador errado.

## 9. Estratégia de testes

| Camada | O que prova | Onde |
|---|---|---|
| **Unidade** | funções puras, limites, erros | `src/**/tests.rs` |
| **Propriedade** | invariantes sobre documentos/operações geradas | `tests/prop_*.rs` |
| **Diferencial** | modelo ingênuo vs modelo real, mesmo resultado | `tests/diff.rs` + `testkit` |
| **Golden** | bytes canônicos, fingerprint, erros de parse | `tests/golden/*.json` |
| **A11y tree** | projeção determinística: role, nome, `alt`, `lang`/`dir`, ordem, ranges | `tests/a11y_*.rs` + goldens |
| **A11y validation** | um caso por `A11yCode`, severidade e `path`/`range` | `tests/a11y_*.rs` |
| **Fuzz** | parser e apply não entram em panic | `fuzz/fuzz_targets/*.rs` |
| **Miri** | ausência de UB (mesmo sem `unsafe` próprio) | `tests/miri_smoke.rs` |
| **Alocação** | zero/≤N alocações no caminho quente | `tests/alloc.rs` |
| **Complexidade** | visitas não crescem com o documento em operação local | `tests/visits.rs` |
| **Bench** | regressão de instruções | `bench/` + CI |
| **MSRV** | compila e testa em 1.92 | CI `msrv` |
| **Docs** | exemplos compilam; links não quebram | `cargo doc -D warnings` |
| **Pacote** | crate publicável, sem path/wildcard acidental | `cargo package` + `cargo publish --dry-run` |
| **WASM** | alvo `wasm32` compila e artefato é gerado | CI `wasm` + `scripts/build_wasm.sh` |
| **JSR/Deno** | tipos, imports, publish dry-run | `deno task check` + `deno task publish:dry` |
| **Composer** | pacote PHP válido, auditado e testado | `composer validate/audit`, PHPUnit, PHPStan |

### 9.1 Modelo ingênuo (diferencial)

O teste diferencial implementa uma versão lenta e óbvia do documento: uma lista de caracteres com
um `Vec<MarkSet>` por caractere e um `Vec<BlockKind>` por bloco. As operações de teste são geradas
por proptest; o resultado é comparado com o `Doc` canônico via `to_json`. Se os dois divergirem, o
teste mostra a sequência de operações e os dois JSONs.

O modelo ingênuo não precisa ser eficiente nem completo: precisa ser **evidentemente correto** para
o subconjunto que cobre. É a rede de segurança contra "plausível mas errado".

### 9.2 Fuzz

Alvos iniciais:

1. **`json_parse`**: qualquer `&str` → `Result<Doc, ParseError>`; nunca panic.
2. **`apply_step`**: doc válido gerado por uma fixture + bytes arbitrários decodificados como
   `Step` → `apply` pode falhar, mas não pode panic nem corromper invariantes.
3. **`round_trip`**: `Doc` → JSON → `Doc` → JSON; os bytes canônicos são idênticos.
4. **`slice`**: `Slice` extraído de um range arbitrário e inserido em outro; invariantes mantidas.
5. **`a11y_tree`**: `Doc` fixture + schema válido → `a11y_tree`/`validate_a11y` sem panic e
   determinístico; cobre o corpus de roles, mídia, tabela e marcas.

Pós-MVP: `html_import`, `markdown_import`, `docx_import`, `odf_import`.

### 9.3 Miri

O core é `forbid(unsafe_code)`, mas as dependências de teste (proptest, alocador de contagem) podem
ter `unsafe`. O alvo `miri_smoke` cobre o caminho de apply/normalize/serialize com uma fixture
pequena; não roda a suíte inteira sob interpretação (lento). `-Zmiri-disable-isolation` evita o
`getcwd` do proptest; os testes do core usam fixtures determinísticas.

### 9.4 Alocação e complexidade

Ver [`05_desempenho.md`](05_desempenho.md). O alocador de contagem e o contador de visitas vivem no
`testkit`, não no core; o core só expõe hooks `#[cfg(test)]`/feature `test-instrument`.

## 10. Política de exceção

1. Todo `#[allow]`/`#[expect]` exige `reason`.
2. Prefira `#[expect]`: quando a lint deixa de disparar, o `expect` vira erro — a supressão não
   fica zumbi.
3. Exceção é **local** (função/statement), nunca de workspace.
4. "Desligar para limpar o CI" é violação: ou o código se adequa, ou entra `#[expect(…, reason =
   "…")]` com justificativa revisável.
5. A exceção de `indexing_slicing` é permitida apenas em um módulo de texto auditado, com
   `debug_assert` de limites e teste de propriedade; não se espalha pelo core.
6. A exceção de `arithmetic_side_effects` é permitida apenas no módulo `math`, com operações
   checked/saturating e teste de borda.

## 11. Documentação

- Rustdoc em **inglês**; `#![deny(missing_docs)]`; exemplos compilam em doc-test.
- `README.md` público em inglês; `plan/`, `AGENTS.md` e `CHANGELOG.md` em português.
- Todo item público tem `# Errors`, `# Panics` (quando aplicável, mas o core não panic) e
  `# Examples` quando a API for não trivial.
- `cargo doc -D warnings` no gate.
- `A11yCode`, `A11yIssue` e `A11yReport` são API pública documentada; cada código tem teste e
  aparece no relatório do CLI. `make check` inclui `a11y`; release sem esse alvo verde é bloqueada.
- `llms.txt`/`SKILL.md` em inglês, gerados/atualizados quando a API estabilizar.
- `scripts/check_zero_style.sh` varre `packages/ts/` e `packages/php/` por `style=`, `class=`,
  `<style`, `insertRule` e arquivos `.css`; se os subprojetos ainda não existem (F0–F3), sai com
  sucesso e uma mensagem. O CI do F3.5 roda o gate sem skip.
- Link relativo quebrado é doc quebrada; um script `check_links.py` (ou equivalente) entra no CI
  quando houver `plan/` + `docs/` suficientes.

## 12. O que não fazer

1. Não adicionar `serde`, `regex`, `html5ever` ou `pulldown-cmark` ao core no MVP.
2. Não adicionar `build.rs` nem geração em tempo de build.
3. Não adicionar `unsafe`; se um dia for inevitável, é um crate separado com decisão e `SAFETY`.
4. Não usar `HashMap`/`HashSet` em caminho de bytes.
5. Não usar relógio, ambiente ou RNG no core.
6. Não usar `unwrap`/`expect`/`panic` em `src/` nem em testes.
7. Não usar `#[allow]` sem `reason`.
8. Não usar bench de parede como gate de CI.
9. Não usar `Debug` como serialização.
10. Não relaxar lint para "passar"; conserte o código ou registre a decisão.
