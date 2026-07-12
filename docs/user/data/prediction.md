# Prediction

Understand and work with prediction outputs from your LM pipelines.

## Predicted wrapper

`Predict` and other modules return `Predicted<O>` where `O` is your signature's output type:

```rust
let output: Predicted<QAOutput> = predict.call(input).await?;
println!("{}", output.answer);  // access output fields directly
```

`Predicted` wraps the output struct plus `CallMetadata` (token usage, latency, etc.).

## Augmented outputs

Strategies like `ChainOfThought` return augmented types:

```rust
let result: Predicted<WithReasoning<QAOutput>> = cot.call(input).await?;
println!("{}", result.reasoning);
println!("{}", result.answer);  // Deref to QAOutput
```

## Dynamic predictions

`DynModule::call` returns predictions with JSON field maps:

```rust
let predicted = module.call(untyped_example).await?;
let answer = predicted.data.get("answer");
```

## UntypedExample

Runtime programs use `UntypedExample` (alias `RawExample`) for input rows with JSON values instead of generated input structs.

## See also

- [Examples](examples.md)
- [Predictors](../building-blocks/predictors.md)
- [Dynamic signatures](../building-blocks/dyn-signatures.md)
