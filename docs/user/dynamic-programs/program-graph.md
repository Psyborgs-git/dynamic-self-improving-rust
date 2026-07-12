# Program graph

Compose runtime `DynPredict` nodes into multi-step pipelines.

`ProgramGraph` wires named nodes with field edges. Each node has a `DynSignature`, a `StrategyKind`, and persisted predictor state.

## Create a single-node graph

```rust
use dsir::{DynSignature, ProgramGraph, StrategyKind};

let sig = DynSignature::from_json_str(r#"{
    "name": "Echo",
    "instruction": "Echo the prompt.",
    "inputs": { "prompt": "string" },
    "outputs": { "answer": "string" }
}"#)?;

let graph = ProgramGraph::from_single("main", &sig, StrategyKind::Predict)?;
```

## Save and load

```rust
let path = "/tmp/graph.json";
graph.save_json(path)?;
let loaded = ProgramGraph::load_json(path)?;
println!("nodes: {:?}", loaded.node_names());
```

`GraphState` captures topology plus per-node predictor state (instructions and demos).

## Multi-node wiring

Nodes share a workspace. `GraphEdge` copies fields between steps:

```rust
// Edge: copy workspace field "summary" into next node's "context" input
GraphEdge { from: "summary".into(), to: "context".into() }
```

Build graphs programmatically or deserialize from JSON authored by the lab.

## Execute

Materialize a `DynModule` from a node and call with `UntypedExample`:

```rust
use dsir::{EchoModule, UntypedExample};
use serde_json::json;

let module = EchoModule::from_signature(&sig, StrategyKind::Predict)?;
let out = module.call(UntypedExample::new(
    [("prompt".into(), json!("ping"))].into_iter().collect(),
    vec!["prompt".into()],
    vec![],
)).await?;
```

## See also

- [Strategy factory](strategy-factory.md)
- [Structural optimization](structural-optimization.md)
- [Persistence](persistence.md)
- Example: `cargo run -p dsir --example 06_graph`
