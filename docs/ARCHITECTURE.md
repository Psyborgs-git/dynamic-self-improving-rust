# Architecture

## Mental model (DSPy-shaped)

Three layers:

| Layer | Concept | Key types |
|-------|---------|-----------|
| **Signatures** | Task contract: inputs → outputs + instructions | `Signature`, `#[derive(Signature)]` |
| **Modules** | Prompting / control-flow strategies | `Module`, `Predict`, `ChainOfThought`, `ReAct`, `Agent`, `BestOfN`, `Refine` |
| **Optimization** | Compile programs against metrics | `Optimizer`, `LabeledFewShot`, `BootstrapFewShot`, COPRO/MIPROv2 |

`Predict` is the only leaf that calls an LM. Higher modules transform signatures or wrap control flow. Optimizers discover `Predict` leaves and mutate **instructions** and **demos**.

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
- **New in dsir:** BootstrapFewShot, LabeledFewShot, stronger COPRO/MIPRO, BestOfN, Refine, Agent facade, richer Evaluate, program persistence, docs.

There is **no** Cargo dependency on `dspy-rs`.

## Toolchain

- `rust-toolchain.toml` pins `stable` (1.97+).
- Async-first (`tokio`).
- Heavy deps (parquet/HF) are feature-gated when present.
