# Structural optimization

Search over strategy choices in dynamic program graphs.

`StructuralOptimizer` explores discrete topology/strategy variants — which `StrategyKind` each node uses — and scores them with a metric on a trainset.

## When to use

- You have a [program graph](program-graph.md) with multiple nodes
- You're unsure whether `Predict` vs `ChainOfThought` vs `BestOfN` works best per step
- You want automated strategy selection alongside instruction optimization

## Workflow

1. Author a base `ProgramGraph` with candidate strategy options
2. Define a `DynMetric` scoring function
3. Run `StructuralOptimizer` to search the strategy space
4. Promote the winning topology via the [lab](../lab/overview.md)

## Typed vs dynamic

Structural search operates on dynamic graphs. For typed Rust modules, swap strategies manually at compile time and use [COPRO](../optimizers/copro.md) or [Bootstrap few-shot](../optimizers/bootstrap-few-shot.md) on the chosen topology.

## See also

- [Strategy factory](strategy-factory.md)
- [compile_dyn](../optimizers/compile-dyn.md)
- [Lab workflows](../lab/workflows.md)
