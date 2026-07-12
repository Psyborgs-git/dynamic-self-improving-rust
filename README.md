# dsir

**Program—don't prompt—language models in Rust.**

`dsir` is a DSPy-shaped library for building composable LM pipelines: typed signatures, modules (`Predict`, `ChainOfThought`, `ReAct`, agents), evaluation, and self-improving prompts (few-shot bootstrap + instruction search). Runtime programs use `DynSignature`, `ProgramGraph`, and the self-improvement lab.

## Install

```toml
[dependencies]
dsir = { path = "crates/dsir" }  # or crates.io when published
tokio = { version = "1", features = ["full"] }
```

Requires **latest stable Rust** (see `rust-toolchain.toml`).

## Quickstart

```rust
use dsir::{configure, ChatAdapter, LM, Predict, Signature};

/// Answer questions accurately and concisely.
#[derive(Signature, Clone, Debug)]
struct QA {
    #[input]
    question: String,
    #[output]
    answer: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    configure(
        LM::builder()
            .model("openai:gpt-4o-mini".to_string())
            .build()
            .await?,
        ChatAdapter,
    );

    let predict = Predict::<QA>::new();
    let out = predict
        .call(QAInput {
            question: "What is the capital of France?".into(),
        })
        .await?;
    println!("{}", out.answer);
    Ok(())
}
```

```bash
cargo run -p dsir --example 01_predict   # requires OPENAI_API_KEY
```

## Library surface

| Area | Types |
|------|--------|
| Signatures | `Signature`, `#[derive(Signature)]`, `DynSignature` (JSON / string DSL) |
| Modules | `Predict`, `DynPredict`, `ChainOfThought`, `ReAct`, `Agent`, `BestOfN`, `Refine` |
| Graphs | `ProgramGraph`, `StrategyFactory`, `StructuralOptimizer` |
| Optimize | `LabeledFewShot`, `BootstrapFewShot`, `COPRO`, `MIPROv2`, `GEPA` (+ `compile_dyn`) |
| Evaluate | `TypedMetric`, `DynMetric`, `Evaluate`, `evaluate_trainset` |
| Lab | `Lab`, local/remote registry, promote/compare |
| Persist | `save_program` / `load_program`, `GraphState` |

## Optimizers at a glance

| Optimizer | Best for |
|-----------|----------|
| `LabeledFewShot` | Fast demo assignment from labeled data |
| `BootstrapFewShot` | Self-improvement from successful traces |
| `COPRO` | Quick instruction search, limited compute |
| `MIPROv2` | Best instruction quality, 15+ examples |
| `GEPA` | Tasks needing textual feedback |

See [optimizers overview](docs/user/optimizers/overview.md).

## Examples

| Example | Command | Notes |
|---------|---------|-------|
| Predict | `cargo run -p dsir --example 01_predict` | Live LM |
| Chain of thought | `cargo run -p dsir --example 02_cot` | Live LM |
| Bootstrap | `cargo run -p dsir --example 04_bootstrap` | Offline |
| Lab | `cargo run -p dsir --example 05_lab` | Offline |
| Program graph | `cargo run -p dsir --example 06_graph` | Offline |

Companion binaries: `dsir-registry` (HTTP run registry), `dsir-lab-ui` (experiment dashboard).

## Documentation

**[Documentation hub](docs/README.md)** — user guides and architecture notes.

Quick links:

- [Quickstart](docs/user/getting-started/quickstart.md)
- [Signatures](docs/user/building-blocks/signature.md)
- [Dynamic signatures](docs/user/building-blocks/dyn-signatures.md)
- [Optimizers](docs/user/optimizers/overview.md)
- [Lab](docs/user/lab/overview.md)
- [Examples index](docs/user/guides/examples.md)

Maintainer docs: [goals](docs/internal/GOALS.md), [architecture](docs/internal/ARCHITECTURE.md), [roadmap](docs/internal/ROADMAP.md).

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE). Core LM pieces adapted from [DSRs](https://github.com/krypticmouse/DSRs); see [vendor notes](docs/internal/VENDOR.md).
