# Vendor notes

> **Audience:** contributors and migrators from upstream DSRs. End users should read the [user documentation](../README.md) instead.

## Source

- Upstream: [krypticmouse/DSRs](https://github.com/krypticmouse/DSRs) (crate historically published as `dspy-rs`)
- License: Apache-2.0
- Pinned commit: `5bb65ca514dfc8240955dd38c870fba77a0bd629` (shallow clone at vendor time)

## What was copied / hard-forked

| Upstream path | In-tree path |
|---------------|--------------|
| `crates/dspy-rs/` | `crates/dsir/` |
| `crates/dsrs-macros/` | `crates/dsir-macros/` |
| `crates/bamltype/` | `crates/bamltype/` |
| `crates/bamltype-derive/` | `crates/bamltype-derive/` |
| BAML runtime crates (formerly `vendor/baml/`) | `crates/psy-*` (owned crates.io namespace) |
| Facet reflection stack (formerly git pin) | `crates/psy-facet*` (owned crates.io namespace) |

## Renames

- Crate `dspy-rs` → `dsir`
- Crate `dsrs_macros` / `dsrs-macros` → `dsir_macros` / `dsir-macros`
- All `use dspy_rs::` / `extern crate` / docs → `dsir`
- BAML vendor crates → `psy-baml-ids`, `psy-baml-types`, `psy-bstd`, `psy-internal-baml-diagnostics`, `psy-internal-baml-jinja`, `psy-jsonish` (Rust lib names unchanged)
- Facet stack → `psy-facet`, `psy-facet-core`, `psy-facet-macros`, … (Rust lib names unchanged)

## Policy

- **No** Cargo dependency on `dspy-rs` or `dsrs`.
- **No** `path =`/`git =` dependencies on unpublished upstream vendor code in publishable crates.
- Prefer crates.io versions of transitive deps (`minijinja`, `rig-core`). Facet is owned in-tree as `psy-facet*` (generic-attr patches required for optimizer leaf discovery).
- Fix compiler/Edition breakage in-tree against `rust-toolchain.toml` (`stable`).
- New DSPy gaps (BootstrapFewShot, BestOfN, etc.) are authored under `dsir`, not upstream.

## Migration notes (rustc 1.97 / Edition 2024)

- Workspace builds on **stable 1.97** (`rust-toolchain.toml`).
- `minijinja` and `rig-core` use crates.io versions; Facet is the `psy-facet*` hard-fork.
- Facet attr grammar still uses the historical `dsrs::` namespace string inside `define_attr_grammar!` (future-incompat lint #52234); crate-level allow retained from upstream until a rename is intentional.
- OpenSSL system packages (`libssl-dev`) are required for `reqwest`/`hf-hub` native TLS on Linux.
- New dsir modules (BootstrapFewShot, BestOfN, Refine, Agent, Evaluate, persistence, LM instruction proposal) live under `crates/dsir/src/` and are not present upstream.

## Publish sequence (crates.io)

Leaf `psy-*` crates first, then `bamltype-derive` → `bamltype` → `dsir-macros` → `dsir`. See `.github/workflows/crates-io.yml`.

## See also

- [Migrating from DSRs](../user/guides/migrating-from-dsrs.md) (user-facing rename guide)
- [NOTICE](../../NOTICE)
