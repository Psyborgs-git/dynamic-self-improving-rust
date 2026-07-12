# Architecture

## Mental model (DSPy-shaped)

Three layers (plus a parallel runtime/dyn path):

| Layer | Concept | Key types |
|-------|---------|-----------|
| **Signatures** | Task contract: inputs → outputs + instructions | `Signature`, `#[derive(Signature)]`, `DynSignature` (JSON/DSL) |
| **Modules** | Prompting / control-flow strategies | `Module`, `Predict`, `DynPredict`, `ChainOfThought`, `ReAct`, `Agent`, `BestOfN`, `Refine`, `ProgramGraph` |
| **Optimization** | Compile programs against metrics | `Optimizer`, `compile_dyn`, `LabeledFewShot`, `BootstrapFewShot`, COPRO/MIPROv2/GEPA, `StructuralOptimizer` |
| **Lab** | Author → run → compare → promote | `Lab`, local/remote registry, experiment UI |

`Predict` / `DynPredict` are the LM leaves. Optimizers discover leaves and mutate **instructions** and **demos**.

## Runtime data flow

```text
User Module
    → Predict.forward(inputs)
        → resolve LM + Adapter (settings / overrides)
        → Adapter.format(signature, demos, inputs) → messages
        → LM.complete(messages) → raw text
        → Adapter.parse(signature, text) → typed outputs
        → optional TraceStep for bootstrapping
```

## Crate layout

```text
crates/
  dsir/           # Main library (vendored DSRs core + dsir extensions)
  dsir-macros/    # Proc macros (Signature, Augmentation)
  bamltype*/      # Schema/jsonish bridge (vendored; required by ChatAdapter)
```

Internal module map inside `dsir`:

| Module | Responsibility |
|--------|----------------|
| `adapter` | Signature ↔ prompt/parse (`ChatAdapter`) |
| `core` | `Module`, `Signature`, `LM`, settings, errors |
| `predictors` | `Predict` leaf + typed `Example` |
| `modules` | CoT, ReAct, Agent, BestOfN, Refine |
| `evaluate` | Metrics + `Evaluate` runner |
| `optimizer` / `teleprompt` | Compilers (bootstrap, instruction search) |
| `data` | Examples, predictions, optional DataLoader |
| `persistence` | Program-level save/load of predictor state |
| `trace` | Execution recording for bootstrap |

## Vendored vs new

- **Vendored (adapted from DSRs):** signatures, macros, ChatAdapter, LM clients, Predict, ChainOfThought, ReAct, Module trait, thin eval, optimizer trait/walker, DataLoader/trace.
- **New in dsir:** BootstrapFewShot, LabeledFewShot, stronger COPRO/MIPRO, BestOfN, Refine, Agent facade, richer Evaluate, program persistence, dynamic programs, lab.

There is **no** Cargo dependency on `dspy-rs`.

## Toolchain

- `rust-toolchain.toml` pins `stable` (1.97+).
- Async-first (`tokio`).
- Heavy deps (parquet/HF) are feature-gated when present.

## See also

- [Signatures](../user/building-blocks/signature.md) and [dynamic signatures](../user/building-blocks/dyn-signatures.md)
- [Modules](../user/building-blocks/modules.md) and [strategies](../user/building-blocks/strategies.md)
- [Optimizers overview](../user/optimizers/overview.md)
- [Dynamic programs](../user/dynamic-programs/program-graph.md)
- [Lab](../user/lab/overview.md)
- [Module map](MODULES.md)
