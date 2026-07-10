# dsir

**Program—don't prompt—language models in Rust.**

`dsir` is a DSPy-shaped library for building composable LM pipelines: typed signatures, modules (`Predict`, `ChainOfThought`, `ReAct`, agents), evaluation, and self-improving prompts (few-shot bootstrap + instruction search).

Core LM programming pieces were adapted from [DSRs](https://github.com/krypticmouse/DSRs) (Apache-2.0) and are owned in-tree. There is **no** dependency on the crates.io `dspy-rs` package. See [docs/VENDOR.md](docs/VENDOR.md).

## Install

```toml
[dependencies]
dsir = { path = "crates/dsir" }  # or crates.io version when published
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

## Docs

- [Goals](docs/GOALS.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Modules](docs/MODULES.md)
- [Roadmap](docs/ROADMAP.md)
- [Vendor notes](docs/VENDOR.md)

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
