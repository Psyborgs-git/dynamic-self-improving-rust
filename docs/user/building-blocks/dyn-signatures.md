# Dynamic signatures

Runtime-configurable task contracts for JSON-authored and lab-driven programs.

Typed `#[derive(Signature)]` is the primary API for static Rust programs. Use `DynSignature` when signatures are defined at runtime — from JSON, a string DSL, or the self-improvement lab.

## When to use

- Author programs in the [lab](../lab/overview.md) without recompiling Rust
- Load signature schemas from config files or a registry
- Build [program graphs](../dynamic-programs/program-graph.md) with runtime topology
- Optimize with [`compile_dyn`](../optimizers/compile-dyn.md)

## JSON schema

```rust
use dsir::DynSignature;

let sig = DynSignature::from_json_str(
    r#"{
        "name": "Echo",
        "instruction": "Echo the prompt.",
        "inputs": { "prompt": "string" },
        "outputs": { "answer": "string" }
    }"#,
)?;
```

Supported primitive shorthands: `"string"`, `"int"`, `"float"`, `"bool"`.

Nested object and list fields:

```json
{
  "name": "Extract",
  "instruction": "Extract structured items.",
  "inputs": { "text": "string" },
  "outputs": {
    "items": {
      "type": "list",
      "items": {
        "type": "object",
        "fields": {
          "label": "string",
          "score": "float"
        }
      }
    }
  }
}
```

## String DSL

Compact authoring for lab workflows:

```rust
let sig = DynSignature::from_dsl("Answer by echoing | prompt -> answer")?;
```

Format: `instruction | input1, input2 -> output1, output2`

## Builder API

```rust
use dsir::{DynFieldType, DynSignature};
use indexmap::indexmap;

let sig = DynSignature::builder()
    .name("QA")
    .instruction("Answer concisely.")
    .input("question", DynFieldType::string())
    .output("answer", DynFieldType::string())
    .build()?;
```

## DynPredict and DynModule

`DynPredict` is the runtime LM leaf — it holds instruction state and demos for a `DynSignature`:

```rust
use dsir::{DynPredict, DynSignature, StrategyKind};

let predict = DynPredict::from_signature(&sig, StrategyKind::Predict)?;
```

`DynModule` is the trait for runtime modules (including `EchoModule` for offline demos and graph nodes).

## Raw adapter I/O

`ChatAdapter` supports both typed signatures and dynamic schemas. Dynamic programs use `UntypedExample` rows with JSON field values instead of generated input structs.

## See also

- [Program graph](../dynamic-programs/program-graph.md)
- [Lab overview](../lab/overview.md)
- [Typed signatures](signature.md) — when you don't need runtime schemas
- Example: `cargo run -p dsir --example 06_graph`
