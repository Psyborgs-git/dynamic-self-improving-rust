# Tracing and debugging

Capture execution graphs for debugging and replay.

`dsir` includes a tracing system that records module execution as a directed acyclic graph (DAG).

## Basic usage

Wrap any execution in `trace::trace()` to capture the graph:

```rust
use dsir::trace;

let graph = trace::trace(|| async {
    module.call(input).await
}).await?;
```

## Inspect and replay

- Inspect nodes and edges in the captured graph
- Replay with new inputs via `trace::Executor`
- Modify graph structure for experimentation

## Bootstrap connection

Execution traces feed into BootstrapFewShot and MIPROv2 (dynamic `compile_dyn` path) for demo seeding.

## Status

A dedicated tracing example is planned (upstream had `12-tracing.rs`). Core tracing APIs are vendored in `crates/dsir/src/trace/`.

## See also

- [Bootstrap few-shot](../optimizers/bootstrap-few-shot.md)
- [MIPROv2](../optimizers/miprov2.md)
