# Running evaluation

Benchmark modules before and after optimization.

## evaluate_trainset

Run a module on every example and collect metric outcomes:

```rust
use dsir::{evaluate_trainset, average_score};

let outcomes = evaluate_trainset(&module, &test_examples, &metric).await?;
let score = average_score(&outcomes);
println!("Average score: {score}");
```

## Evaluate runner

`Evaluate` provides parallel evaluation with aggregation for larger trainsets. Optimizers use this internally; call it directly to benchmark.

## Workflow

```text
1. Build module (before optimization)
2. evaluate_trainset → baseline score
3. optimizer.compile → improved module
4. evaluate_trainset → after score
5. Compare
```

## Dynamic programs

For `DynModule`, use `DynMetric` with `UntypedExample` rows. The lab's `optimize` method runs evaluation automatically during compare.

## See also

- [Metrics](metrics.md)
- [DataLoader](../data/dataloader.md)
- [Optimizers overview](../optimizers/overview.md)
