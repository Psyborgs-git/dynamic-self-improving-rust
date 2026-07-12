# Strategy factory

Build runtime modules from `DynSignature` + `StrategyKind`.

`StrategyFactory` materializes the correct `DynModule` implementation for a dynamic signature:

```rust
use dsir::{DynSignature, StrategyFactory, StrategyKind};

let sig = DynSignature::from_dsl("Answer | question -> answer")?;
let module = StrategyFactory::build(&sig, StrategyKind::ChainOfThought)?;
```

## Supported strategies

| `StrategyKind` | Module behavior |
|----------------|-----------------|
| `Predict` | Single LM call |
| `ChainOfThought` | Adds reasoning field |
| `ReAct` | Tool loop |
| `Agent` | Agent facade over ReAct |
| `BestOfN` | Sample N, pick best |
| `Refine` | BestOfN + feedback |

## Program graph integration

`ProgramGraph` uses `StrategyFactory` internally when hydrating nodes from `GraphState`. Change strategy per node without recompiling Rust.

## Structural search

[`StructuralOptimizer`](structural-optimization.md) varies `StrategyKind` choices across nodes to find better program topology.

## See also

- [Dynamic signatures](../building-blocks/dyn-signatures.md)
- [Program graph](program-graph.md)
