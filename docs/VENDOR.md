# Vendor notes

## Source

- Upstream: [krypticmouse/DSRs](https://github.com/krypticmouse/DSRs) (crate historically published as `dspy-rs`)
- License: Apache-2.0
- Pinned commit: `5bb65ca514dfc8240955dd38c870fba77a0bd629` (shallow clone at vendor time)

## What was copied

| Upstream path | In-tree path |
|---------------|--------------|
| `crates/dspy-rs/` | `crates/dsir/` |
| `crates/dsrs-macros/` | `crates/dsir-macros/` |
| `crates/bamltype/` | `crates/bamltype/` |
| `crates/bamltype-derive/` | `crates/bamltype-derive/` |
| `vendor/baml/` (as required) | `vendor/baml/` |

## Renames

- Crate `dspy-rs` → `dsir`
- Crate `dsrs_macros` / `dsrs-macros` → `dsir_macros` / `dsir-macros`
- All `use dspy_rs::` / `extern crate` / docs → `dsir`

## Policy

- **No** Cargo dependency on `dspy-rs` or `dsrs`.
- Prefer crates.io versions of transitive deps; keep git pins only when required for Facet/rig/minijinja compatibility, documented in workspace `Cargo.toml`.
- Fix compiler/Edition breakage in-tree against `rust-toolchain.toml` (`stable`).
- New DSPy gaps (BootstrapFewShot, BestOfN, etc.) are authored under `dsir`, not upstream.

## Migration notes (rustc 1.97 / Edition 2024)

- Workspace builds on **stable 1.97** (`rust-toolchain.toml`).
- Facet / rig-core / minijinja remain git-pinned (same revs as upstream DSRs) — required for optimizer leaf discovery and LM tool loops.
- Facet attr grammar still uses the historical `dsrs::` namespace string inside `define_attr_grammar!` (future-incompat lint #52234); crate-level allow retained from upstream until Facet ships a fix.
- OpenSSL system packages (`libssl-dev`) are required for `reqwest`/`hf-hub` native TLS on Linux.
- New dsir modules (BootstrapFewShot, BestOfN, Refine, Agent, Evaluate, persistence, LM instruction proposal) live under `crates/dsir/src/` and are not present upstream.
