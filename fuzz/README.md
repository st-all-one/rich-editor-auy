# Fuzz targets

Run from this directory with nightly, a C++ toolchain (for libFuzzer) and `cargo-fuzz`:

```sh
cargo +nightly fuzz run --fuzz-dir . json_parse -- -max_total_time=60
cargo +nightly fuzz run --fuzz-dir . a11y_tree -- -max_total_time=60
cargo +nightly fuzz run --fuzz-dir . apply_step -- -max_total_time=60
cargo +nightly fuzz run --fuzz-dir . round_trip -- -max_total_time=60
cargo +nightly fuzz run --fuzz-dir . fragment -- -max_total_time=60
```

`--fuzz-dir .` is required because this directory is already the fuzz crate; without it
`cargo fuzz` looks for `./fuzz/Cargo.toml`. `make fuzz` runs all five targets for 30 s each, and the
CI job runs them for 60 s.

| Target | Invariant |
|---|---|
| `json_parse` | Any UTF-8 input either parses into a validated document or returns a located error; the a11y projection and `validate_a11y` never panic |
| `a11y_tree` | The semantic tree is projected and serialized for every parsed document |
| `apply_step` | Delete and insert transactions either apply or fail atomically |
| `round_trip` | Canonical bytes re-parse to the same bytes, and a full `Doc::slice` survives `Slice` wire round-trip with the same size |
| `fragment` | `from_fragment_json`/`from_slice_json` accept or reject deterministically, and accepted payloads re-serialize to the same canonical bytes |

Every target uses `Schema::full`, so the corpus covers tables, media, `raw_code` and marks.
Seeds live in `corpus/<target>/`; libFuzzer also writes new entries next to them.

Last verified local run (2026-11-09, `rustlang/rust:nightly-bookworm` + `cargo-fuzz`, zero
crashes): `json_parse` 75 840 runs, `a11y_tree` 68 130, `apply_step` 61 873, `round_trip` 77 422
(60 s each) and `fragment` 23 727 runs (20 s).

On a container without the `personality()` syscall (Docker's default seccomp profile), the runner
fails before fuzzing. Use `--security-opt seccomp=unconfined` when the image is the only place with
a C++ toolchain.
