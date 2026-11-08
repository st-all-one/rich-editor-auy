# Referências executáveis

Requer Node 22 e npm. Os pacotes não são dependências do workspace Rust.

```sh
cd bench/reference
npm install
node make_basic.mjs
node pm-basic.mjs /tmp/complex.json
node pm-all.mjs
node lexical.mjs
N=500 node quill.mjs
N=500 node tiptap.mjs
(cd taino && cargo run --release)
```

`N` controla as repetições do template (500 ≈ 3.000 blocos no caso básico). Quill e Tiptap
precisam de `happy-dom`; os pacotes estão em `package.json`.

`pm-all.mjs` lê `/tmp/complex-all.json`, gerado por:

```sh
cargo run --release --manifest-path bench/Cargo.toml --example all_types
```

Os números completos estão em [`../COMPLEX-COMPARISON.md`](../COMPLEX-COMPARISON.md).
