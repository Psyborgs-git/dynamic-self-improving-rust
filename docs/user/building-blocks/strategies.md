# Strategies

Prompting and control-flow strategies beyond bare `Predict`.

## Chain of thought

`ChainOfThought` prepends a `reasoning` output field and delegates to an inner `Predict`:

```rust
use dsir::{ChainOfThought, Signature};

#[derive(Signature, Clone, Debug)]
struct MathQA {
    #[input] question: String,
    #[output] answer: String,
}

let cot = ChainOfThought::<MathQA>::new();
let result = cot.call(MathQAInput { question: "What is 2+2?".into() }).await?;

println!("{}", result.reasoning);
println!("{}", result.answer);  // via Deref to MathQAOutput
```

Returns `Predicted<WithReasoning<MathQAOutput>>`. Access `.reasoning` directly; `.answer` through auto-deref.

Example: `cargo run -p dsir --example 02_cot`

## ReAct

`ReAct` runs a tool loop: the LM proposes actions, tools execute, results feed back until a final answer is extracted.

```rust
use dsir::ReAct;

let react = ReAct::<MySignature>::builder()
    .max_steps(5)
    .tools(my_tools)
    .build();
```

Requires `rig` tool definitions for production use. Example `03_react` shows construction; add tools for live runs.

## Agent

`Agent` is a DSPy-shaped facade over `ReAct` with a simpler builder API:

```rust
use dsir::Agent;

let agent = Agent::<MySignature>::builder()
    .max_steps(3)
    .tools(my_tools)
    .build();
```

## BestOfN

Sample `n` rollouts and keep the best by a reward function:

```rust
use dsir::BestOfN;

let best_of_n = BestOfN::<QA>::builder()
    .n(5)
    .reward_fn(|pred| score_prediction(pred))
    .build();
```

## Refine

`Refine` extends BestOfN with LM-generated feedback between attempts:

```rust
use dsir::Refine;

let refine = Refine::<QA>::builder()
    .n(3)
    .reward_fn(|pred| score_prediction(pred))
    .build();
```

## Dynamic strategies

For runtime programs, use `StrategyKind` with `DynPredict` and [StrategyFactory](../dynamic-programs/strategy-factory.md):

- `StrategyKind::Predict`
- `StrategyKind::ChainOfThought`
- `StrategyKind::ReAct`
- `StrategyKind::Agent`
- `StrategyKind::BestOfN`
- `StrategyKind::Refine`

## See also

- [Predictors](predictors.md)
- [Modules](modules.md)
- [Dynamic signatures](dyn-signatures.md)
