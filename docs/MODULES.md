# Module responsibilities

## Public surface (`dsir`)

| Symbol | Responsibility | Origin |
|--------|----------------|--------|
| `configure` / settings | Global LM + adapter defaults | Vendored |
| `Signature` + derive | Typed I/O contract + instructions | Vendored |
| `DynSignature` / JSON / DSL | Runtime signatures (nested object/list) | **New** |
| `Predict` | Atomic LM call; holds demos + instruction state | Vendored |
| `DynPredict` / `DynModule` | Runtime-schema LM leaf + trait | **New** |
| `ChainOfThought` | Prepend reasoning field; delegates to Predict | Vendored |
| `ReAct` | Tool loop + extract | Vendored |
| `Agent` | DSPy-shaped agent API over ReAct | **New** |
| `BestOfN` | Sample N rollouts; pick by reward | **New** |
| `Refine` | BestOfN + feedback between attempts | **New** |
| `ProgramGraph` / `StrategyFactory` / `StructuralOptimizer` | Dyn composition + strategy search | **New** |
| `Module` | Composable program trait | Vendored |
| `LM` / `ChatAdapter` | Providers + prompt format/parse (typed + raw) | Vendored + **extended** |
| `Example` / `Predicted` | Train records and outputs | Vendored |
| `TypedMetric` / `DynMetric` / `Evaluate` | Scoring + batch evaluation | Vendored + **extended** |
| `LabeledFewShot` | Assign labeled demos (`compile` + `compile_dyn`) | **New** |
| `BootstrapFewShot` | Teacher trace → demos on student | **New** |
| `COPRO` / `MIPROv2` / `GEPA` | Instruction search (+ dyn compile) | Vendored shell, **replaced/extended** |
| `Lab` / registry | Author, optimize, compare, promote | **New** |
| `save` / `load` / `GraphState` | Persist compiled predictor/graph state | **New** |

## Companion binaries

| Crate | Responsibility |
|-------|----------------|
| `dsir-registry` | HTTP remote run registry |
| `dsir-lab-ui` | Local experiment dashboard |

## Internal crates

| Crate | Responsibility |
|-------|----------------|
| `dsir` | Library facade and all runtime modules |
| `dsir-macros` | `#[derive(Signature)]`, augmentation derives |
| `bamltype` (+ derive) | Type IR, Jinja render, jsonish parse used by ChatAdapter |

## Optimizer discovery

Optimizers walk module graphs to find `Predict` / `DynPredict` leaves (Facet / `DynPredictor` walker). Bootstrap and instruction search mutate those leaves only.
