# compile_dyn

Optimize runtime programs without typed signatures.

`compile_dyn` is the dynamic counterpart to `Optimizer::compile`. It operates on `DynModule` implementations with `UntypedExample` trainsets and `DynMetric` scorers.

## Supported optimizers

| Optimizer | `compile_dyn` |
|-----------|-----------------|
| `LabeledFewShot` | Yes |
| `BootstrapFewShot` | Yes |
| `COPRO` | Yes |
| `MIPROv2` | Yes (seeds demos from traces) |
| `GEPA` | Yes |

## Typed vs dynamic paths

| Path | API | Signatures | Trainset |
|------|-----|------------|----------|
| Typed | `optimizer.compile(...)` | `#[derive(Signature)]` | `Vec<Example<S>>` |
| Dynamic | `optimizer.compile_dyn(...)` | `DynSignature` | `Vec<UntypedExample>` |

## MIPROv2 note

Typed `MIPROv2::compile` is instruction-only on the static path (demo seeding from traces is tracked as a known gap). The **dynamic** `compile_dyn` path seeds demos from successful traces.

## Lab integration

`Lab::optimize` uses `compile_dyn` internally for runtime-authored programs. See [lab workflows](../lab/workflows.md).

## Example

```rust
use dsir::{BootstrapFewShot, DynMetric, EchoModule, StrategyKind};

let mut module = EchoModule::from_signature(&sig, StrategyKind::Predict)?;
let optimizer = BootstrapFewShot::default();
optimizer.compile_dyn(&mut module, trainset, &metric).await?;
```

## See also

- [Dynamic signatures](../building-blocks/dyn-signatures.md)
- [Bootstrap few-shot](bootstrap-few-shot.md)
- [MIPROv2](miprov2.md)
