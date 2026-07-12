# dsir documentation

**Program—don't prompt—language models in Rust.**

## Learn dsir

Start here if you are building LM pipelines with `dsir`.

### Getting started

- [Introduction](user/index.md)
- [Quickstart](user/getting-started/quickstart.md)

### Building blocks

- [Signatures](user/building-blocks/signature.md)
- [Dynamic signatures](user/building-blocks/dyn-signatures.md) (dsir-only)
- [Predictors](user/building-blocks/predictors.md)
- [Modules](user/building-blocks/modules.md)
- [Strategies](user/building-blocks/strategies.md) (CoT, ReAct, Agent, BestOfN, Refine)
- [Adapter](user/building-blocks/adapter.md)
- [Language models](user/building-blocks/language-models.md)
- [Custom types](user/building-blocks/types.md)
- [Constraints](user/building-blocks/constraints.md)

### Data

- [Examples](user/data/examples.md)
- [Predictions](user/data/prediction.md)
- [DataLoader](user/data/dataloader.md)

### Evaluation

- [Metrics](user/evaluation/metrics.md)
- [Running evaluation](user/evaluation/running-evaluation.md)

### Optimizers

- [Overview](user/optimizers/overview.md)
- [Labeled few-shot](user/optimizers/labeled-few-shot.md)
- [Bootstrap few-shot](user/optimizers/bootstrap-few-shot.md)
- [COPRO](user/optimizers/copro.md)
- [MIPROv2](user/optimizers/miprov2.md)
- [GEPA](user/optimizers/gepa.md)
- [GEPA with LLM judge](user/optimizers/gepa-llm-judge.md)
- [compile_dyn](user/optimizers/compile-dyn.md)

### Dynamic programs (dsir-only)

- [Program graph](user/dynamic-programs/program-graph.md)
- [Strategy factory](user/dynamic-programs/strategy-factory.md)
- [Structural optimization](user/dynamic-programs/structural-optimization.md)
- [Persistence](user/dynamic-programs/persistence.md)

### Self-improvement lab (dsir-only)

- [Lab overview](user/lab/overview.md)
- [Workflows](user/lab/workflows.md)
- [Registry](user/lab/registry.md)
- [Lab UI](user/lab/lab-ui.md)

### Guides

- [Examples index](user/guides/examples.md)
- [Tracing and debugging](user/guides/tracing-and-debugging.md)
- [Migrating from DSRs](user/guides/migrating-from-dsrs.md)

### Tutorials

- [Overview](user/tutorials/overview.md)

---

## Contributing / architecture

Maintainer-facing notes for contributors and migrators.

- [Goals](internal/GOALS.md)
- [Architecture](internal/ARCHITECTURE.md)
- [Module map](internal/MODULES.md)
- [Roadmap](internal/ROADMAP.md)
- [Vendor notes](internal/VENDOR.md) (provenance and migration policy)
