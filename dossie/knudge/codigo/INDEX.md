# `codigo/` — origem, licença e por que cada arquivo está aqui

Todos os arquivos abaixo são **cópias byte-a-byte** de `ref/knudge/`, feitas com `cp`, sem
edição. O projeto é licenciado **`MIT OR Apache-2.0`** (`ref/knudge/Cargo.toml:12`), então a
cópia integral é permitida com atribuição; os textos das licenças estão em
`ref/knudge/LICENSE-MIT` e `ref/knudge/LICENSE-APACHE`.

Verificação de integridade: as 6 cópias foram conferidas com `cmp` contra a origem e são
**byte-idênticas** (comando sem saída, 6/6). Para reconferir:

```sh
cd /home/one/projects/one/rich-editor-auy
for f in Makefile clippy.toml Cargo.toml; do cmp "ref/knudge/$f" "dossie/knudge/codigo/$f"; done
cmp ref/knudge/.github/workflows/ci.yml   dossie/knudge/codigo/ci.yml
cmp ref/knudge/scripts/check_file_length.sh dossie/knudge/codigo/check_file_length.sh
cmp ref/knudge/crates/knudge-core/src/error.rs dossie/knudge/codigo/error.rs
```

| Arquivo local | Origem (`ref/knudge/...`) | Linhas | Por que foi copiado |
|---|---|---:|---|
| `Makefile` | `Makefile` | 154 | É o **gate**: `check: fmt clippy test file-length` (`:11`) e os alvos extras que pulam quando a ferramenta falta (`:64-128`). Nenhum resumo substitui ver que o alvo `deny`/`miri` é condicional. |
| `ci.yml` | `.github/workflows/ci.yml` | 92 | Mostra que o CI **chama o mesmo `make check`** (`:26`), o `RUSTFLAGS=-D warnings` global (`:12`) e os jobs de supply-chain, miri e fuzz com 30 s por alvo. |
| `check_file_length.sh` | `scripts/check_file_length.sh` | 21 | O gate de 300 linhas é 21 linhas de bash. Vale copiar inteiro: `limit=300` (`:6`), varredura só de `crates/**/src/*.rs` (`:15`), exit code correto (`:17-19`). |
| `clippy.toml` | `clippy.toml` | 194 | A camada de **valores** do rigor: `allow-unwrap-in-tests = false` (`:18`), thresholds de complexidade (`:43-45`), `disallowed-types` com `reason` (`:114-127`), `msrv` (`:74`). |
| `Cargo.toml` | `Cargo.toml` | 142 | A camada de **níveis** (`[workspace.lints.*]`, `:32-101`), o orçamento de dependências (`:14-30`) e os perfis de build (`:132-142`). Só faz sentido lendo junto com `clippy.toml`. |
| `error.rs` | `crates/knudge-core/src/error.rs` | 254 | Representante do crate: `ErrorKind` fechado (`:14`), `code()`/`exit_code()` (`:38`, `:56`), `enum Error` `#[non_exhaustive]` (`:74`), `lock_or_recover` (`:199`). É o padrão "erros como valores" de R30–R35. |

## O que estes seis arquivos, juntos, demonstram

1. **Gate único e reproduzível**: `Makefile:11` é a mesma coisa que `ci.yml:26` roda.
2. **Rigor em duas camadas**: níveis em `Cargo.toml`, valores em `clippy.toml` — e a doc
   `15_clippy_config.md:17-30` explica por que uma sem a outra não funciona.
3. **Limite objetivo**: o gate de tamanho é um script de 21 linhas, não um acordo social.
4. **Fronteira de erro explícita**: código de máquina separado de mensagem e de exit code.

## O que **não** foi copiado (e por quê)

- `crates/knudge-core/src/store/commit.rs` (19 linhas): exemplifica "nota antes do evento", mas
  é pequeno demais para ocupar uma vaga; o conceito está citado em
  [`../arquitetura.md` §7.1](../arquitetura.md#71-divergencesmd--o-artefato-mais-copiável) e em
  `ref/knudge/wiki/specs/DIVERGENCES.md:21`.
- `wiki/specs/DIVERGENCES.md` (145 linhas): é documento, não código; foi lido e resumido com
  citações de linha, e o texto integral é GPL-free (MIT/Apache), podendo ser copiado se o
  revisor quiser.
- `install.sh`, `scripts/bump-version.sh`, `release.yml`: analisados em
  [`../arquitetura.md` §9](../arquitetura.md#9-release-e-distribuição); copiar os três passaria do
  orçamento de 6 arquivos sem acrescentar padrão novo.
