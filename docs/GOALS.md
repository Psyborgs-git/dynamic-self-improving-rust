# Goals

## Product goals

`dsir` is a DSPy-shaped Rust library for **programming language models**: declare typed task signatures, compose modules (`Predict`, `ChainOfThought`, `ReAct`, agents), and **compile** programs against metrics so prompts and few-shot demos improve automatically.

Users should be able to:

1. Define a task with a typed `Signature` (inputs → outputs + instructions).
2. Pick a strategy (`Predict`, `ChainOfThought`, `ReAct`, `Agent`, `BestOfN`, `Refine`).
3. Compose multi-step pipelines as ordinary Rust modules.
4. Evaluate with a metric and optimize with `LabeledFewShot` / `BootstrapFewShot` / instruction search.
5. Persist compiled predictor state (`save` / `load`) and redeploy without re-optimizing.

## Non-goals (v0.1)

- Full Stanford DSPy parity (external GEPA engine, GRPO, weight finetuning, ProgramOfThought sandbox, ColBERT retrieve, streaming, multimodal).
- Depending on the crates.io `dspy-rs` / `dsrs` package at runtime.
- A string-signature DSL as the primary API (typed `#[derive(Signature)]` is the default; inline helpers may exist).

## Success criteria

- `cargo build` and core tests pass on **latest stable Rust** (pinned in `rust-toolchain.toml`).
- Users install with `cargo add dsir` and write DSPy-shaped pipelines without importing any DSRs crate.
- BootstrapFewShot can improve a small QA program on a toy trainset (demo + metric).
- Architecture and module responsibilities are documented under `docs/`.

## Provenance

Core LM programming pieces (signatures, adapters, predictors, CoT, ReAct, LM clients) are **vendored and adapted** from [DSRs](https://github.com/krypticmouse/DSRs) (Apache-2.0). See [VENDOR.md](VENDOR.md) and [NOTICE](../NOTICE). Gaps and weak optimizers are implemented in-tree as `dsir` modules.
