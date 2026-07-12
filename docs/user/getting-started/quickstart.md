# Quickstart

Build your first LM pipeline in minutes.

`dsir` lets you call language models with typed Rust structs. Define your inputs and outputs as a struct, and the library handles prompt formatting and response parsing.

## 1. Install

Add to your `Cargo.toml`:

```toml
[dependencies]
dsir = { path = "crates/dsir" }  # or crates.io when published
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

Or via cargo:

```bash
cargo add dsir tokio anyhow
```

Requires **latest stable Rust** (see `rust-toolchain.toml`).

## 2. Configure the LM

Tell dsir which model to use. This sets a global default that all predictors will use:

```rust
use dsir::{configure, init_tracing, ChatAdapter, LM};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;

    configure(
        LM::builder()
            .model("openai:gpt-4o-mini".to_string())
            .build()
            .await?,
        ChatAdapter,
    );

    Ok(())
}
```

Set `OPENAI_API_KEY` in your environment. For other providers, use the appropriate prefix (e.g. `anthropic:claude-3-haiku`).

## 3. Define a signature

A [signature](../building-blocks/signature.md) declares your task's inputs and outputs:

```rust
use dsir::Signature;

/// Answer questions accurately and concisely.
#[derive(Signature, Clone, Debug)]
struct QA {
    /// The question to answer
    #[input]
    question: String,

    /// A clear, direct answer
    #[output]
    answer: String,
}
```

The doc comments become:

- **Struct docstring** → instruction for the LM
- **Field docstrings** → field descriptions in the prompt

## 4. Call the LM

Create a [predictor](../building-blocks/predictors.md) and call it:

```rust
use dsir::Predict;

let predict = Predict::<QA>::new();

let output = predict.call(QAInput {
    question: "What is the capital of France?".into(),
}).await?;

println!("Answer: {}", output.answer);
```

The `#[derive(Signature)]` macro generates `QAInput` from your `#[input]` fields.

## Complete example

```rust
use dsir::{configure, init_tracing, ChatAdapter, LM, Predict, Signature};

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
    init_tracing()?;

    configure(
        LM::builder()
            .model("openai:gpt-4o-mini".to_string())
            .build()
            .await?,
        ChatAdapter,
    );

    let predict = Predict::<QA>::new();
    let output = predict.call(QAInput {
        question: "What is the capital of France?".into(),
    }).await?;

    println!("Answer: {}", output.answer);
    Ok(())
}
```

Run the bundled example:

```bash
cargo run -p dsir --example 01_predict
```

## Next steps

- [Signatures](../building-blocks/signature.md) — field attributes, demos, constraints
- [Modules](../building-blocks/modules.md) — multi-step pipelines
- [Dynamic signatures](../building-blocks/dyn-signatures.md) — runtime JSON/DSL programs
- [Optimizers overview](../optimizers/overview.md) — improve prompts automatically
- [Examples index](../guides/examples.md) — all runnable examples

## Adding complexity

### Few-shot demos

```rust
use dsir::Example;

let predict = Predict::<QA>::builder()
    .demo(Example::<QA>::new(
        QAInput { question: "What is 2+2?".into() },
        QAOutput { answer: "4".into() },
    ))
    .build();
```

### Chain of thought

```rust
use dsir::ChainOfThought;

let cot = ChainOfThought::<QA>::new();
let result = cot.call(QAInput { question: "What is 2+2?".into() }).await?;
println!("Reasoning: {}", result.reasoning);
println!("Answer: {}", result.answer);
```

See [Strategies](../building-blocks/strategies.md) for ReAct, Agent, BestOfN, and Refine.
