# Optimizers overview

Choose the right optimizer for your task, data, and compute budget.

Optimizers discover `Predict` / `DynPredict` leaves in your module graph and mutate their **instructions** and **few-shot demos**.

## Comparison table

| Optimizer | Approach | Few-shot demos | Instruction search | Feedback | Best for |
|-----------|----------|----------------|-------------------|----------|----------|
| **LabeledFewShot** | Assign labeled demos | Yes (labeled) | No | Score only | Fast baseline, you have good labels |
| **BootstrapFewShot** | Trace → demo on success | Yes (bootstrapped) | No | Score only | Teacher/student, self-improvement |
| **COPRO** | Iterative refinement | No | Yes | Score only | Quick iteration, simple tasks |
| **MIPROv2** | LLM-guided generation | Yes (dyn path) | Yes | Traces + tips | Complex reasoning, 15+ examples |
| **GEPA** | Evolutionary + feedback | No | Yes | Textual feedback | Subtle failures, LLM judge |
| **StructuralOptimizer** | Strategy topology search | — | — | Metric | Dynamic graph strategy choice |

## When to use each

### LabeledFewShot

- You have labeled training data ready to paste as demos
- No LM budget for bootstrapping or instruction search
- [Guide](labeled-few-shot.md)

### BootstrapFewShot

- A teacher (or the module itself) can produce good traces on your trainset
- Self-improvement workflows in the [lab](../lab/overview.md)
- [Guide](bootstrap-few-shot.md) — example `04_bootstrap`

### COPRO

- Fast iteration needed
- Simple tasks with clear metrics
- Limited compute budget
- [Guide](copro.md)

### MIPROv2

- Best possible instruction quality
- Complex reasoning tasks
- 15+ training examples recommended
- Use `compile_dyn` for demo seeding from traces
- [Guide](miprov2.md)

### GEPA

- Score alone doesn't explain failures
- Need actionable textual feedback
- Pareto exploration of instruction space
- [Guide](gepa.md) and [GEPA with LLM judge](gepa-llm-judge.md)

## Typed vs dynamic

- **Typed programs:** `optimizer.compile(&mut module, trainset, &metric)`
- **Runtime programs:** `optimizer.compile_dyn(&mut dyn_module, trainset, &metric)`

See [compile_dyn](compile-dyn.md).

## Optimizer discovery

Optimizers walk your module via Facet reflection. Any `Predict<S>` field inside a `#[derive(Facet)]` struct is discovered automatically:

```rust
#[derive(facet::Facet)]
#[facet(crate = facet)]
struct Pipeline {
    analyzer: Predict<Analyze>,
    summarizer: Predict<Summarize>,
}
```

Both leaves are optimizer-visible. Non-predict fields are ignored.

## See also

- [Running evaluation](../evaluation/running-evaluation.md)
- [Examples index](../guides/examples.md)
