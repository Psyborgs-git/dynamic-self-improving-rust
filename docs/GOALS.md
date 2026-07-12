# Goals

## Product goals

`dsir` is a DSPy-shaped Rust library for **programming language models**: declare task signatures (typed or runtime/JSON), compose modules (`Predict`, `ChainOfThought`, `ReAct`, agents, dyn graphs), and **compile** programs against metrics so prompts and few-shot demos improve automatically.

Users should be able to:

1. Define a task with a typed `Signature` **or** a runtime [`DynSignature`](../crates/dsir/src/core/dyn_signature.rs) (JSON schema / string DSL, including nested object/list fields).
2. Pick a strategy (`Predict`, `ChainOfThought`, `ReAct`, `Agent`, `BestOfN`, `Refine`, or `StrategyFactory` for dyn).
3. Compose multi-step pipelines as Rust modules **or** a [`ProgramGraph`](../crates/dsir/src/graph/program_graph.rs).
4. Evaluate with metrics and optimize with `LabeledFewShot` / `BootstrapFewShot` / COPRO / MIPROv2 / GEPA / structural search (`compile` and `compile_dyn`).
5. Persist compiled predictor/graph state and promote winners via the self-improvement lab (local + remote registry, experiment UI).

## Non-goals (v0.1)

- Full Stanford DSPy parity (external GEPA engine, GRPO, weight finetuning, ProgramOfThought sandbox, ColBERT retrieve, streaming, multimodal).
- Depending on the crates.io `dspy-rs` / `dsrs` package at runtime.
- Replacing typed `#[derive(Signature)]` for static Rust programs (JSON/DSL is primary for **dynamic** modules only).

## Success criteria

- `cargo build` and core tests pass on **latest stable Rust** (pinned in `rust-toolchain.toml`).
- Users install with `cargo add dsir` and write DSPy-shaped pipelines without importing any DSRs crate.
- BootstrapFewShot can improve a small QA program on a toy trainset (demo + metric).
- Dyn lab example can author → optimize → compare → promote offline.
- Architecture and module responsibilities are documented under `docs/`.

## Provenance

Core LM programming pieces (signatures, adapters, predictors, CoT, ReAct, LM clients) are **vendored and adapted** from [DSRs](https://github.com/krypticmouse/DSRs) (Apache-2.0). See [VENDOR.md](VENDOR.md) and [NOTICE](../NOTICE). Gaps and weak optimizers are implemented in-tree as `dsir` modules.
