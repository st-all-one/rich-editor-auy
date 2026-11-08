# Fuzz targets

Run from this directory with nightly and `cargo-fuzz`:

```sh
cargo +nightly fuzz run json_parse -- -max_total_time=60
```

The target exercises the canonical parser, the accessibility projection and its JSON writer.
