# Examples index

Runnable examples in `crates/dsir/examples/`.

| Example | Command | API key? | Demonstrates |
|---------|---------|----------|--------------|
| `01_predict` | `cargo run -p dsir --example 01_predict` | Yes (`OPENAI_API_KEY`) | Signature → Predict |
| `02_cot` | `cargo run -p dsir --example 02_cot` | Yes | ChainOfThought |
| `03_react` | `cargo run -p dsir --example 03_react` | No (construction only) | ReAct + Agent setup |
| `04_bootstrap` | `cargo run -p dsir --example 04_bootstrap` | No (offline) | BootstrapFewShot + save/load |
| `05_lab` | `cargo run -p dsir --example 05_lab` | No (offline) | Lab author → optimize → promote |
| `06_graph` | `cargo run -p dsir --example 06_graph` | No (offline) | ProgramGraph + DynSignature |

## By topic

### Getting started
- [01_predict](../../../crates/dsir/examples/01_predict.rs) — [Quickstart](../getting-started/quickstart.md)

### Strategies
- [02_cot](../../../crates/dsir/examples/02_cot.rs) — [Chain of thought](../building-blocks/strategies.md)
- [03_react](../../../crates/dsir/examples/03_react.rs) — [ReAct / Agent](../building-blocks/strategies.md)

### Optimization
- [04_bootstrap](../../../crates/dsir/examples/04_bootstrap.rs) — [Bootstrap few-shot](../optimizers/bootstrap-few-shot.md)

### Dynamic programs and lab
- [05_lab](../../../crates/dsir/examples/05_lab.rs) — [Lab workflows](../lab/workflows.md)
- [06_graph](../../../crates/dsir/examples/06_graph.rs) — [Program graph](../dynamic-programs/program-graph.md)

## Planned ports

Upstream DSRs examples not yet ported to dsir:

- Tracing DAG (`12-tracing`)
- HotPotQA evaluation/optimization
- Live GEPA demos
- Custom LM client / batch providers

See [tutorials overview](../tutorials/overview.md).
