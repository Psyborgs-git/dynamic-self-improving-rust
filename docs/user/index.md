# dsir

**Program—don't prompt—language models in Rust.**

`dsir` is a DSPy-shaped library for building composable LM pipelines in Rust. Declare typed or runtime signatures, compose modules (`Predict`, `ChainOfThought`, `ReAct`, agents, dynamic graphs), evaluate with metrics, and compile programs so instructions and few-shot demos improve automatically.

## Why dsir

- **Typed by default:** `#[derive(Signature)]` enforces input/output contracts at compile time.
- **Runtime programs:** `DynSignature`, `ProgramGraph`, and the self-improvement lab for JSON/DSL-authored pipelines.
- **Self-improving prompts:** Bootstrap few-shot, instruction search (COPRO, MIPROv2, GEPA), and experiment compare/promote workflows.
- **Rust-native:** Async-first, Facet-based optimizer discovery, no runtime dependency on external DSPy crates.

## Quickstart

New to dsir? Start with the [Quickstart](getting-started/quickstart.md) guide, then run:

```bash
cargo run -p dsir --example 01_predict   # requires OPENAI_API_KEY
```

## Learn more

### Building blocks

- [Signatures](building-blocks/signature.md) — typed task contracts
- [Dynamic signatures](building-blocks/dyn-signatures.md) — JSON schema and string DSL
- [Predictors](building-blocks/predictors.md) — LM leaf modules
- [Modules](building-blocks/modules.md) — composable pipelines
- [Strategies](building-blocks/strategies.md) — CoT, ReAct, Agent, BestOfN, Refine
- [Adapter](building-blocks/adapter.md) — prompt format and parse
- [Language models](building-blocks/language-models.md) — providers and configuration

### Data and evaluation

- [Examples](data/examples.md) and [predictions](data/prediction.md)
- [DataLoader](data/dataloader.md)
- [Metrics](evaluation/metrics.md) and [running evaluation](evaluation/running-evaluation.md)

### Optimizers

- [Overview](optimizers/overview.md) — when to use each optimizer
- [Bootstrap few-shot](optimizers/bootstrap-few-shot.md)
- [COPRO](optimizers/copro.md), [MIPROv2](optimizers/miprov2.md), [GEPA](optimizers/gepa.md)
- [compile_dyn](optimizers/compile-dyn.md) — optimize runtime programs

### Dynamic programs and lab

- [Program graph](dynamic-programs/program-graph.md)
- [Persistence](dynamic-programs/persistence.md)
- [Lab overview](lab/overview.md) — author → optimize → compare → promote

### Guides

- [Examples index](guides/examples.md)
- [Tracing and debugging](guides/tracing-and-debugging.md)

> Inspired by [DSPy](https://github.com/stanfordnlp/dspy). Core LM programming pieces are adapted in-tree; see [vendor notes](../internal/VENDOR.md) for provenance.
