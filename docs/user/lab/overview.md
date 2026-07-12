# Lab overview

The self-improvement lab authors, optimizes, compares, and promotes dynamic LM programs offline.

`Lab` is a filesystem-backed experiment runner. It manages programs, datasets, optimization runs, and promoted winners — without requiring live LM calls for smoke workflows.

## Core workflow

```text
author → optimize → compare → promote → execute
```

1. **Author** a program with `DynSignature` + `StrategyKind`
2. **Load** a JSON dataset
3. **Optimize** with BootstrapFewShot, COPRO, or other `LabOptimizer` variants
4. **Compare** runs by train/val scores
5. **Promote** the best run
6. **Execute** the promoted program on new inputs

## Open a lab

```rust
use dsir::Lab;

let workdir = std::env::temp_dir().join("my_lab");
let mut lab = Lab::open(&workdir)?;
```

## Author a program

```rust
use dsir::{DynSignature, StrategyKind};

let sig = DynSignature::from_dsl("Answer by echoing | prompt -> answer")?;
lab.author_echo("echo_qa", sig, StrategyKind::Predict)?;
```

`author_echo` installs a deterministic echo module for offline testing. Replace with live modules for production.

## See also

- [Workflows](workflows.md) — full optimize/compare/promote example
- [Registry](registry.md) — remote run storage
- [Lab UI](lab-ui.md) — experiment dashboard
- Example: `cargo run -p dsir --example 05_lab`
