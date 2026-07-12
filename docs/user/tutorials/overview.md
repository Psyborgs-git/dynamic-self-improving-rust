# Tutorials overview

End-to-end examples for learning dsir.

## Available tutorials

| Tutorial | Example | Guide |
|----------|---------|-------|
| First LM call | `01_predict` | [Quickstart](../getting-started/quickstart.md) |
| Chain of thought | `02_cot` | [Strategies](../building-blocks/strategies.md) |
| Bootstrap optimization | `04_bootstrap` | [Bootstrap few-shot](../optimizers/bootstrap-few-shot.md) |
| Self-improvement lab | `05_lab` | [Lab workflows](../lab/workflows.md) |
| Dynamic program graph | `06_graph` | [Program graph](../dynamic-programs/program-graph.md) |

Run any example:

```bash
cargo run -p dsir --example 01_predict
```

See the [examples index](../guides/examples.md) for API key requirements.

## Planned tutorials

These existed in upstream DSRs but are not yet ported:

- **Tracing** — capture and replay execution DAGs
- **HotPotQA** — evaluation and optimization on a real dataset
- **GEPA live** — evolutionary optimization with LLM judge feedback
- **Custom LM client** — non-OpenAI providers and batch mode
- **Tool loops** — full ReAct with rig tools

Track progress in [roadmap](../../internal/ROADMAP.md).

## Learning path

1. [Quickstart](../getting-started/quickstart.md) → `01_predict`
2. [Signatures](../building-blocks/signature.md) + [Modules](../building-blocks/modules.md)
3. [Evaluation](../evaluation/running-evaluation.md) + [Optimizers overview](../optimizers/overview.md)
4. [Dynamic signatures](../building-blocks/dyn-signatures.md) → `06_graph`
5. [Lab workflows](../lab/workflows.md) → `05_lab`
