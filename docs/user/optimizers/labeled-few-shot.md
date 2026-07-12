# Labeled few-shot

Assign labeled training examples as demos on every `Predict` leaf.

`LabeledFewShot` is the simplest optimizer: take up to `k` examples from your trainset and attach them as few-shot demos.

## Usage

```rust
use dsir::{LabeledFewShot, Optimizer};

let optimizer = LabeledFewShot::builder().k(4).build();
optimizer.compile(&mut module, trainset, &metric).await?;
```

## How it works

1. Walk the module graph to find all `Predict` / `DynPredict` leaves
2. Convert trainset rows to demos (best-effort per leaf schema)
3. Assign demos to each leaf

No LM calls required — this is pure demo assignment.

## Dynamic programs

`LabeledFewShot` also implements `compile_dyn` for runtime modules:

```rust
optimizer.compile_dyn(&mut dyn_module, raw_trainset, &dyn_metric).await?;
```

## When to use

- You already have good labeled examples
- You want a fast baseline before BootstrapFewShot or instruction search
- You need demos on a new student module copied from a teacher

## See also

- [Bootstrap few-shot](bootstrap-few-shot.md) — demos from successful traces
- [Optimizers overview](overview.md)
