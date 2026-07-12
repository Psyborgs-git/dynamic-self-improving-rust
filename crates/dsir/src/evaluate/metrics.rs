//! Built-in metrics for typed and dynamic evaluation.

use anyhow::Result;
use serde_json::Value;

use crate::data::example::Example as RawExample;
use crate::predictors::DynModule;
use crate::{Predicted, evaluate::MetricOutcome};

use super::FeedbackMetric;

/// Score a dynamic module prediction against a gold [`RawExample`].
#[allow(async_fn_in_trait)]
pub trait DynMetric: Send + Sync {
    async fn evaluate(
        &self,
        example: &RawExample,
        prediction: &Predicted<RawExample>,
    ) -> Result<MetricOutcome>;
}

/// Exact match on a single output field (stringified JSON equality).
#[derive(Debug, Clone)]
pub struct FieldExactMatch {
    pub field: String,
    pub with_feedback: bool,
}

impl FieldExactMatch {
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            with_feedback: false,
        }
    }

    pub fn with_feedback(mut self) -> Self {
        self.with_feedback = true;
        self
    }
}

impl DynMetric for FieldExactMatch {
    async fn evaluate(
        &self,
        example: &RawExample,
        prediction: &Predicted<RawExample>,
    ) -> Result<MetricOutcome> {
        let gold = example.data.get(&self.field);
        let pred = prediction.data.get(&self.field);
        let score = match (gold, pred) {
            (Some(g), Some(p)) if values_equal(g, p) => 1.0,
            _ => 0.0,
        };
        if self.with_feedback {
            let feedback = FeedbackMetric::new(
                score,
                format!(
                    "field `{}`: gold={:?} pred={:?} score={score}",
                    self.field, gold, pred
                ),
            );
            Ok(MetricOutcome::with_feedback(score, feedback))
        } else {
            Ok(MetricOutcome::score(score))
        }
    }
}

/// Exact match across all gold output keys present on the example.
#[derive(Debug, Clone, Default)]
pub struct ExactMatchAll {
    pub with_feedback: bool,
}

impl ExactMatchAll {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_feedback(mut self) -> Self {
        self.with_feedback = true;
        self
    }
}

impl DynMetric for ExactMatchAll {
    async fn evaluate(
        &self,
        example: &RawExample,
        prediction: &Predicted<RawExample>,
    ) -> Result<MetricOutcome> {
        let keys = if example.output_keys.is_empty() {
            example
                .data
                .keys()
                .filter(|k| !example.input_keys.contains(k))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            example.output_keys.clone()
        };
        if keys.is_empty() {
            return Ok(MetricOutcome::score(0.0));
        }
        let mut hits = 0usize;
        for key in &keys {
            let gold = example.data.get(key);
            let pred = prediction.data.get(key);
            if matches!((gold, pred), (Some(g), Some(p)) if values_equal(g, p)) {
                hits += 1;
            }
        }
        let score = hits as f32 / keys.len() as f32;
        if self.with_feedback {
            Ok(MetricOutcome::with_feedback(
                score,
                FeedbackMetric::new(score, format!("matched {hits}/{}", keys.len())),
            ))
        } else {
            Ok(MetricOutcome::score(score))
        }
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(x), Value::String(y)) => x.trim() == y.trim(),
        _ => a == b,
    }
}

/// Evaluate a [`DynModule`] on a raw trainset.
pub async fn evaluate_dyn_trainset<M, MT>(
    module: &M,
    trainset: &[RawExample],
    metric: &MT,
) -> Result<Vec<MetricOutcome>>
where
    M: DynModule,
    MT: DynMetric,
{
    let mut outcomes = Vec::with_capacity(trainset.len());
    for example in trainset {
        let input = input_only(example);
        let predicted = module
            .call(input)
            .await
            .map_err(|err| anyhow::anyhow!("{err}"))?;
        outcomes.push(metric.evaluate(example, &predicted).await?);
    }
    Ok(outcomes)
}

pub fn input_only(example: &RawExample) -> RawExample {
    let keys = if example.input_keys.is_empty() {
        example
            .data
            .keys()
            .filter(|k| !example.output_keys.contains(k))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        example.input_keys.clone()
    };
    let mut data = std::collections::HashMap::new();
    for key in &keys {
        if let Some(v) = example.data.get(key) {
            data.insert(key.clone(), v.clone());
        }
    }
    RawExample::new(data, keys, Vec::new())
}
