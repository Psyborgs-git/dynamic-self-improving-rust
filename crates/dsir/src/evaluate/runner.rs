//! Parallel evaluation runner with aggregation.

use anyhow::{Result, anyhow};
use futures::future::join_all;

use crate::core::Module;
use crate::evaluate::{MetricOutcome, TypedMetric, average_score};
use crate::predictors::Example;
use crate::Signature;

/// Batch evaluator with optional concurrency.
///
/// Closer to DSPy's `Evaluate` class: run a module on a devset, score with a metric,
/// return an aggregated report.
pub struct Evaluate {
    pub num_threads: usize,
}

impl Default for Evaluate {
    fn default() -> Self {
        Self { num_threads: 4 }
    }
}

impl Evaluate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_threads(mut self, n: usize) -> Self {
        self.num_threads = n.max(1);
        self
    }

    /// Run `module` on `devset` and score with `metric` (preserves order).
    pub async fn run<S, M, MT>(
        &self,
        module: &M,
        devset: &[Example<S>],
        metric: &MT,
    ) -> Result<EvaluateReport>
    where
        S: Signature,
        S::Input: Clone + Send + Sync,
        M: Module<Input = S::Input>,
        MT: TypedMetric<S, M>,
    {
        let chunk = self.num_threads;
        let mut outcomes = Vec::with_capacity(devset.len());

        for window in devset.chunks(chunk) {
            let futs = window.iter().enumerate().map(|(offset, example)| {
                let idx = outcomes.len() + offset;
                async move {
                    let predicted = module
                        .call(example.input.clone())
                        .await
                        .map_err(|err| anyhow!("example {idx}: {err}"))?;
                    metric
                        .evaluate(example, &predicted)
                        .await
                        .map_err(|err| anyhow!("metric {idx}: {err}"))
                }
            });
            let batch = join_all(futs).await;
            for outcome in batch {
                outcomes.push(outcome?);
            }
        }

        let avg = average_score(&outcomes);
        Ok(EvaluateReport {
            n: outcomes.len(),
            average: avg,
            outcomes,
        })
    }
}

/// Summary of an [`Evaluate::run`] call.
#[derive(Debug, Clone)]
pub struct EvaluateReport {
    pub outcomes: Vec<MetricOutcome>,
    pub average: f32,
    pub n: usize,
}

impl EvaluateReport {
    pub fn average_score(&self) -> f32 {
        self.average
    }
}
