# Metrics

Score module outputs for evaluation and optimization.

## TypedMetric

Implement `TypedMetric<S, M>` for typed programs:

```rust
use dsir::{Example, MetricOutcome, Predicted, TypedMetric};

struct ExactMatch;

impl TypedMetric<QA, MyModule> for ExactMatch {
    async fn evaluate(
        &self,
        example: &Example<QA>,
        prediction: &Predicted<QAOutput>,
    ) -> Result<MetricOutcome> {
        let score = if prediction.answer == example.output.answer { 1.0 } else { 0.0 };
        Ok(MetricOutcome::score(score))
    }
}
```

## Score-only vs feedback

| Return type | Used by |
|-------------|---------|
| `MetricOutcome::score(f32)` | COPRO, MIPROv2, BootstrapFewShot |
| `MetricOutcome::with_feedback(...)` | GEPA (textual feedback guides search) |

## DynMetric

For runtime programs, implement `DynMetric`:

```rust
use dsir::{DynMetric, MetricOutcome, UntypedExample};

struct EchoMatch;

impl DynMetric for EchoMatch {
    async fn evaluate(
        &self,
        example: &UntypedExample,
        prediction: &Predicted<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<MetricOutcome> {
        // compare example vs prediction fields
        Ok(MetricOutcome::score(1.0))
    }
}
```

## Built-in helpers

The `evaluate` module provides utilities for aggregation and common patterns. See [running evaluation](running-evaluation.md).

## See also

- [Optimizers overview](../optimizers/overview.md)
- [GEPA with LLM judge](../optimizers/gepa-llm-judge.md)
