# Module responsibilities

## Public surface (`dsir`)

| Symbol | Responsibility | Origin |
|--------|----------------|--------|
| `configure` / settings | Global LM + adapter defaults | Vendored |
| `Signature` + derive | Typed I/O contract + instructions | Vendored |
| `Predict` | Atomic LM call; holds demos + instruction state | Vendored |
| `ChainOfThought` | Prepend reasoning field; delegates to Predict | Vendored |
| `ReAct` | Tool loop + extract | Vendored |
| `Agent` | DSPy-shaped agent API over ReAct | **New** |
| `BestOfN` | Sample N rollouts; pick by reward | **New** |
| `Refine` | BestOfN + feedback between attempts | **New** |
| `Module` | Composable program trait | Vendored |
| `LM` / `ChatAdapter` | Providers + prompt format/parse | Vendored |
| `Example` / `Predicted` | Train records and outputs | Vendored |
| `TypedMetric` / `Evaluate` | Scoring + batch evaluation | Vendored + **extended** |
| `LabeledFewShot` | Assign labeled demos | **New** |
| `BootstrapFewShot` | Teacher trace → demos on student | **New** |
| `COPRO` / `MIPROv2` | Instruction search (LM-proposed) | Vendored shell, **replaced bodies** |
| `save` / `load` | Persist compiled predictor state | **New** |

## Internal crates

| Crate | Responsibility |
|-------|----------------|
| `dsir` | Library facade and all runtime modules |
| `dsir-macros` | `#[derive(Signature)]`, augmentation derives |
| `bamltype` (+ derive) | Type IR, Jinja render, jsonish parse used by ChatAdapter |

## Optimizer discovery

Optimizers walk module graphs to find `Predict` leaves (Facet / `DynPredictor` walker, vendored). Bootstrap and instruction search mutate those leaves only.
