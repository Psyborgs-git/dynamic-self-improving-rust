# Persistence

Save and restore compiled predictor state across runs.

After optimization, predictor leaves hold improved instructions and few-shot demos. `save_program` / `load_program` persist that state to disk.

## Typed programs

```rust
use dsir::{dump_program_state, load_program_state, save_program, load_program};

// Save entire module state
save_program(&module, "program.json")?;

// Load into a fresh module
let mut module = MyModule::new();
load_program(&mut module, "program.json")?;
```

`ProgramState` is a map from dotted predictor path → `PredictState` (instruction + demos).

## Graph state

For dynamic programs, `ProgramGraph::save_json` / `load_json` persist both topology and per-node predictor state as `GraphState`.

## Lab integration

The [lab](../lab/overview.md) stores promoted programs under a workdir. `promote` writes the winning run's compiled state for later `execute` calls.

## Round-trip guarantees

`load_program_state` rolls back on partial failure — if any predictor fails to load, previously updated leaves are restored to their pre-load state.

## See also

- [Bootstrap few-shot](../optimizers/bootstrap-few-shot.md) — produces demos to persist
- Example: `cargo run -p dsir --example 04_bootstrap`
