# Bootstrap few-shot

Bootstrap few-shot demos by running the student and keeping successful traces.

`BootstrapFewShot` is a DSPy-shaped teleprompter: it runs your module on training examples, scores outputs, and adds successful predictions as few-shot demos.

## Usage

```rust
use dsir::{BootstrapFewShot, Optimizer};

let optimizer = BootstrapFewShot::builder()
    .max_bootstrapped_demos(4)
    .max_labeled_demos(2)
    .metric_threshold(0.8)
    .build();

optimizer.compile(&mut module, trainset, &metric).await?;
```

## Algorithm

For each train example (until `max_bootstrapped_demos` successes):

1. Seed up to `max_labeled_demos` labeled examples first
2. Run the student module on the example
3. Score with the metric
4. On success (`score >= metric_threshold`), append as a demo

## Offline demo

Example `04_bootstrap` uses a deterministic `EchoModule` teacher — no API key required:

```bash
cargo run -p dsir --example 04_bootstrap
```

## Persistence

After bootstrapping, save the compiled state:

```rust
use dsir::{save_program, load_program};

save_program(&module, "optimized.json")?;
load_program(&mut module, "optimized.json")?;
```

## Dynamic programs

Use `compile_dyn` for runtime modules in the lab:

```rust
optimizer.compile_dyn(&mut dyn_module, raw_trainset, &dyn_metric).await?;
```

## See also

- [Labeled few-shot](labeled-few-shot.md)
- [Persistence](../dynamic-programs/persistence.md)
- [Optimizers overview](overview.md)
