# Example

Explore data currency that makes up dsir.

`Example` is the typed training/evaluation row for a signature `S`.

```rust
use dsir::{Example, Signature};

#[derive(Signature, Clone, Debug)]
struct QA {
    #[input]
    question: String,
    #[output]
    answer: String,
}

let row = Example::new(
    QAInput {
        question: "What is 2+2?".to_string(),
    },
    QAOutput {
        answer: "4".to_string(),
    },
);
```

Use `Vec<Example<S>>` for:
- `evaluate_trainset(...)`
- `optimizer.compile(...)`

For file/dataset ingestion, use [`DataLoader`](../data/dataloader.md).
