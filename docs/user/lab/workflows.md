# Lab workflows

End-to-end author → optimize → compare → promote workflow.

This follows [`05_lab.rs`](../../../crates/dsir/examples/05_lab.rs).

## Setup

```rust
use dsir::{DynSignature, Lab, LabOptimizer, StrategyKind};
use serde_json::json;

let workdir = std::env::temp_dir().join("dsir_lab_example");
let mut lab = Lab::open(&workdir)?;
```

## Author and load data

```rust
let signature = DynSignature::from_dsl("Answer by echoing | prompt -> answer")?;
lab.author_echo("echo_qa", signature, StrategyKind::Predict)?;

lab.load_dataset_json(
    "toy",
    vec![
        json!({"prompt": "hello", "answer": "hello"}),
        json!({"prompt": "world", "answer": "world"}),
    ],
    vec!["prompt".into()],
    vec!["answer".into()],
)?;
```

## Optimize

### Bootstrap few-shot

```rust
let bootstrap = lab.optimize(
    "echo_qa",
    "toy",
    3,  // val split size
    LabOptimizer::BootstrapFewShot {
        max_bootstrapped_demos: 2,
        max_labeled_demos: 1,
        metric_threshold: 1.0,
    },
    "answer",  // output field to score
).await?;
```

### COPRO

```rust
let copro = lab.optimize(
    "echo_qa",
    "toy",
    3,
    LabOptimizer::Copro { breadth: 3, depth: 1 },
    "answer",
).await?;
```

## Compare and promote

```rust
let comparison = lab.compare(&[bootstrap.id.clone(), copro.id.clone()])?;
let best = comparison.first().map(|r| r.run_id.clone()).unwrap();
let promoted = lab.promote(&best, 0.0)?;
```

## Execute promoted program

```rust
use dsir::UntypedExample;

let out = lab.execute(
    "echo_qa",
    UntypedExample::new(
        [("prompt".into(), json!("test"))].into_iter().collect(),
        vec!["prompt".into()],
        vec![],
    ),
).await?;
```

## See also

- [Lab overview](overview.md)
- [Bootstrap few-shot](../optimizers/bootstrap-few-shot.md)
- [compile_dyn](../optimizers/compile-dyn.md)
