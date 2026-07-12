# Module responsibilities

## Public surface (`dsir`)

| Symbol | Responsibility | Origin | User guide |
|--------|----------------|--------|------------|
| `configure` / settings | Global LM + adapter defaults | Vendored | [Language models](../user/building-blocks/language-models.md) |
| `Signature` + derive | Typed I/O contract + instructions | Vendored | [Signatures](../user/building-blocks/signature.md) |
| `DynSignature` / JSON / DSL | Runtime signatures (nested object/list) | **New** | [Dynamic signatures](../user/building-blocks/dyn-signatures.md) |
| `Predict` | Atomic LM call; holds demos + instruction state | Vendored | [Predictors](../user/building-blocks/predictors.md) |
| `DynPredict` / `DynModule` | Runtime-schema LM leaf + trait | **New** | [Dynamic signatures](../user/building-blocks/dyn-signatures.md) |
| `ChainOfThought` | Prepend reasoning field; delegates to Predict | Vendored | [Strategies](../user/building-blocks/strategies.md) |
| `ReAct` | Tool loop + extract | Vendored | [Strategies](../user/building-blocks/strategies.md) |
| `Agent` | DSPy-shaped agent API over ReAct | **New** | [Strategies](../user/building-blocks/strategies.md) |
| `BestOfN` | Sample N rollouts; pick by reward | **New** | [Strategies](../user/building-blocks/strategies.md) |
| `Refine` | BestOfN + feedback between attempts | **New** | [Strategies](../user/building-blocks/strategies.md) |
| `ProgramGraph` / `StrategyFactory` / `StructuralOptimizer` | Dyn composition + strategy search | **New** | [Program graph](../user/dynamic-programs/program-graph.md) |
| `Module` | Composable program trait | Vendored | [Modules](../user/building-blocks/modules.md) |
| `LM` / `ChatAdapter` | Providers + prompt format/parse (typed + raw) | Vendored + **extended** | [Adapter](../user/building-blocks/adapter.md) |
| `Example` / `Predicted` | Train records and outputs | Vendored | [Examples](../user/data/examples.md) |
| `TypedMetric` / `DynMetric` / `Evaluate` | Scoring + batch evaluation | Vendored + **extended** | [Metrics](../user/evaluation/metrics.md) |
| `LabeledFewShot` | Assign labeled demos (`compile` + `compile_dyn`) | **New** | [Labeled few-shot](../user/optimizers/labeled-few-shot.md) |
| `BootstrapFewShot` | Teacher trace → demos on student | **New** | [Bootstrap few-shot](../user/optimizers/bootstrap-few-shot.md) |
| `COPRO` / `MIPROv2` / `GEPA` | Instruction search (+ dyn compile) | Vendored shell, **replaced/extended** | [Optimizers overview](../user/optimizers/overview.md) |
| `Lab` / registry | Author, optimize, compare, promote | **New** | [Lab](../user/lab/overview.md) |
| `save` / `load` / `GraphState` | Persist compiled predictor/graph state | **New** | [Persistence](../user/dynamic-programs/persistence.md) |

## Companion binaries

| Crate | Responsibility | User guide |
|-------|----------------|------------|
| `dsir-registry` | HTTP remote run registry | [Registry](../user/lab/registry.md) |
| `dsir-lab-ui` | Local experiment dashboard | [Lab UI](../user/lab/lab-ui.md) |

## Internal crates

| Crate | Responsibility |
|-------|----------------|
| `dsir` | Library facade and all runtime modules |
| `dsir-macros` | `#[derive(Signature)]`, augmentation derives |
| `bamltype` (+ derive) | Type IR, Jinja render, jsonish parse used by ChatAdapter |

## Optimizer discovery

Optimizers walk module graphs to find `Predict` / `DynPredict` leaves (Facet / `DynPredictor` walker). Bootstrap and instruction search mutate those leaves only.

## See also

- [Architecture](ARCHITECTURE.md)
- [User documentation hub](../README.md)
