# Migrating from DSRs

If you used upstream [DSRs](https://github.com/krypticmouse/DSRs) (`dspy-rs` on crates.io), this guide maps the renames and new features in `dsir`.

## Crate and import renames

| Upstream | dsir |
|----------|------|
| `dspy-rs` / `dsrs` crate | `dsir` |
| `use dspy_rs::` | `use dsir::` |
| `dsrs-macros` | `dsir-macros` |

```toml
# Before
dsrs = { package = "dspy-rs", version = "0.7" }

# After
dsir = "0.1.1"
```

## Model strings

dsir uses provider prefixes:

```rust
// Before
.model("gpt-4o-mini".to_string())

// After
.model("openai:gpt-4o-mini".to_string())
```

## What stayed the same

- `#[derive(Signature)]` with `#[input]` / `#[output]`
- `Predict`, `ChainOfThought`, `ReAct`, `ChatAdapter`, `LM`
- `Example`, `Predicted`, `TypedMetric`, `evaluate_trainset`
- `COPRO`, `MIPROv2`, `GEPA` optimizer APIs
- `DataLoader` typed loaders

## What's new in dsir

| Feature | Guide |
|---------|-------|
| `DynSignature` (JSON/DSL) | [Dynamic signatures](../building-blocks/dyn-signatures.md) |
| `ProgramGraph` | [Program graph](../dynamic-programs/program-graph.md) |
| `BootstrapFewShot` | [Bootstrap few-shot](../optimizers/bootstrap-few-shot.md) |
| `LabeledFewShot` | [Labeled few-shot](../optimizers/labeled-few-shot.md) |
| `Lab` (author/optimize/promote) | [Lab overview](../lab/overview.md) |
| `compile_dyn` | [compile_dyn](../optimizers/compile-dyn.md) |
| `BestOfN`, `Refine`, `Agent` | [Strategies](../building-blocks/strategies.md) |
| `save_program` / `load_program` | [Persistence](../dynamic-programs/persistence.md) |

## Dependency policy

dsir has **no** runtime Cargo dependency on `dspy-rs` or `dsrs`. Core code is vendored in-tree.

For contributor-level provenance and pinned upstream commit, see [vendor notes](../../internal/VENDOR.md).

## Documentation

Upstream hosted docs at dsrs.herumbshandilya.com are replaced by this repo's [user documentation](../index.md).
