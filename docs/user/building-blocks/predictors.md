# Predictors

Call LMs with typed signatures

A `Predict` takes a [signature](../building-blocks/signature.md) and actually calls the LM. It's the bridge between your type definitions and real LLM inference. Under the hood, it uses an [adapter](../building-blocks/adapter.md) to format prompts and parse responses.

## Basic usage

```rust
use dsir::{Predict, Signature};

#[derive(Signature, Clone, Debug)]
/// Answer questions accurately.
struct QA {
    #[input]
    question: String,
    #[output]
    answer: String,
}

// Create a predictor for this signature
let predict = Predict::<QA>::new();

// Call it with typed input
let result = predict.call(QAInput {
    question: "What is the capital of France?".into(),
}).await?;

// Access typed output directly (Predicted<O> implements Deref<Target = O>)
println!("{}", result.answer);  // "Paris"
```

The turbofish `::` tells Rust which signature you're using. The macro generates `QAInput` from your `#[input]` fields.

## Creating predictors

### Simple

```rust
let predict = Predict::<QA>::new();
```

### With instruction override

```rust
let predict = Predict::<QA>::builder()
    .instruction("Answer like a pirate.")
    .build();
```

This overrides the docstring instruction on the signature.

### With demos (few-shot)

```rust
use dsir::Example;

let predict = Predict::<QA>::builder()
    .demo(Example::<QA>::new(
        QAInput { question: "What is 2+2?".into() },
        QAOutput { answer: "4".into() },
    ))
    .demo(Example::<QA>::new(
        QAInput { question: "What color is grass?".into() },
        QAOutput { answer: "Green".into() },
    ))
    .build();
```

Demos are `Example` — typed input/output pairs. They become few-shot examples in the prompt.

### With tools

```rust
let predict = Predict::<QA>::builder()
    .add_tool(my_tool)
    .build();
```

## Calling predictors

`.call()` returns `Result, PredictError>`.

`Predicted` wraps the output with call metadata and implements `Deref`, so you access fields directly:

```rust
let result = predict.call(QAInput {
    question: "Why is the sky blue?".into(),
}).await?;

// Direct field access via Deref
println!("{}", result.answer);
```

### Accessing metadata

For token usage, raw response text, or per-field parse details, use `.metadata()`:

```rust
let result = predict.call(input).await?;

// Token usage
let usage = &result.metadata().lm_usage;
println!("Tokens: {} in, {} out", usage.prompt_tokens, usage.completion_tokens);

// Raw LM response text
println!("Raw: {}", result.metadata().raw_response);

// Per-field parse details (raw text, constraint results, flags)
if let Some(field) = result.metadata().field_meta.get("answer") {
    println!("Raw text for answer: {}", field.raw_text);
    for check in &field.checks {
        println!("{}: {}", check.label, if check.passed { "ok" } else { "failed" });
    }
}
```

### `CallMetadata` fields

| Field | Type | Description |
|-------|------|-------------|
| `raw_response` | `String` | Raw LLM response text |
| `lm_usage` | `LmUsage` | Token counts |
| `tool_calls` | `Vec` | Tool calls the LM requested |
| `tool_executions` | `Vec` | Results from tool execution |
| `node_id` | `Option` | Trace node ID if tracing |
| `field_meta` | `IndexMap` | Per-field raw text, flags, constraint results |

## Error handling

```rust
use dsir::PredictError;

match predict.call(input).await {
    Ok(output) => println!("{}", output.answer),
    Err(PredictError::Lm { source }) => {
        // LLM call failed (network, rate limit, etc.)
        eprintln!("LLM error: {}", source);
    }
    Err(PredictError::Parse { source, raw_response, .. }) => {
        // Got a response but couldn't parse it
        eprintln!("Parse error: {}", source);
        eprintln!("Raw response was: {}", raw_response);
    }
    Err(PredictError::Conversion { source, .. }) => {
        // Parsed but couldn't convert to Rust types
        eprintln!("Conversion error: {}", source);
    }
}
```

## Predict implements Module

`Predict` implements the [`Module`](../building-blocks/modules.md) trait with typed associated types:

```rust
impl<S: Signature> Module for Predict<S> {
    type Input = S::Input;
    type Output = S::Output;

    async fn forward(&self, input: S::Input) -> Result<Predicted<S::Output>, PredictError>;
}
```

This means predictors work with optimizers and can be nested in custom modules.

## Multiple predictors in a pipeline

```rust
#[derive(Signature, Clone, Debug)]
struct Summarize {
    #[input] text: String,
    #[output] summary: String,
}

#[derive(Signature, Clone, Debug)]
struct Analyze {
    #[input] summary: String,
    #[output] sentiment: String,
    #[output] key_points: Vec<String>,
}

// Chain them
let summarizer = Predict::<Summarize>::new();
let analyzer = Predict::<Analyze>::new();

let summary = summarizer.call(SummarizeInput {
    text: long_text.into()
}).await?;

let analysis = analyzer.call(AnalyzeInput {
    summary: summary.summary
}).await?;

println!("Sentiment: {}", analysis.sentiment);
```

## Prompting strategies

Instead of manually adding fields for chain-of-thought reasoning, use library modules that augment any signature:

- **`ChainOfThought`** -- adds a `reasoning` field, accessible via `result.reasoning`
- **`ReAct`** -- adds tool-calling with an action/observation loop

See [Modules](../building-blocks/modules.md) for details.
