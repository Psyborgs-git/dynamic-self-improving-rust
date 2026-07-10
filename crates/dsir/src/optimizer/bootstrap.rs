//! Few-shot demo assignment and bootstrapping (DSPy-shaped teleprompters).

use anyhow::{Result, anyhow};
use bon::Builder;

use crate::core::DynPredictor;
use crate::data::example::Example as RawExample;
use crate::evaluate::{TypedMetric, average_score};
use crate::optimizer::{
    Optimizer, evaluate_module_with_metric, predictor_names, with_named_predictor,
};
use crate::predictors::Example;
use crate::{BamlType, Facet, Module, Signature};

/// Assign up to `k` labeled trainset examples as demos on every Predict leaf.
///
/// Demos are stored as type-erased [`RawExample`] rows built from the trainset signature.
/// Predictors whose field schema cannot accept a row skip that demo (best-effort).
#[derive(Builder)]
pub struct LabeledFewShot {
    #[builder(default = 4)]
    pub k: usize,
}

impl Default for LabeledFewShot {
    fn default() -> Self {
        Self { k: 4 }
    }
}

impl Optimizer for LabeledFewShot {
    type Report = usize;

    async fn compile<S, M, MT>(
        &self,
        module: &mut M,
        trainset: Vec<Example<S>>,
        _metric: &MT,
    ) -> Result<Self::Report>
    where
        S: Signature,
        S::Input: Clone,
        M: Module<Input = S::Input> + for<'a> Facet<'a>,
        MT: TypedMetric<S, M>,
    {
        let demos: Vec<RawExample> = trainset
            .iter()
            .take(self.k)
            .map(raw_from_typed_example)
            .collect::<Result<_>>()?;

        let names = predictor_names(module)?;
        if names.is_empty() {
            return Err(anyhow!("no optimizable predictors found"));
        }

        let mut assigned = 0usize;
        for name in names {
            with_named_predictor(module, &name, |predictor| {
                assign_demos_best_effort(predictor, demos.clone())?;
                assigned += 1;
                Ok(())
            })?;
        }
        Ok(assigned)
    }
}

/// Bootstrap few-shot demos by running a teacher program and keeping successful traces.
///
/// For each train example (until `max_bootstrapped_demos` successes):
/// 1. Run the student module (teacher override can be layered by the caller via LM settings)
/// 2. Score with the metric
/// 3. On success (`score >= metric_threshold`), append the labeled example as a demo
///
/// Also seeds up to `max_labeled_demos` labeled examples first (like DSPy).
#[derive(Builder)]
pub struct BootstrapFewShot {
    #[builder(default = 4)]
    pub max_bootstrapped_demos: usize,
    #[builder(default = 4)]
    pub max_labeled_demos: usize,
    #[builder(default = 1.0)]
    pub metric_threshold: f32,
}

impl Default for BootstrapFewShot {
    fn default() -> Self {
        Self {
            max_bootstrapped_demos: 4,
            max_labeled_demos: 4,
            metric_threshold: 1.0,
        }
    }
}

/// Report from [`BootstrapFewShot::compile`].
#[derive(Debug, Clone)]
pub struct BootstrapReport {
    pub labeled: usize,
    pub bootstrapped: usize,
    pub average_train_score: f32,
}

impl Optimizer for BootstrapFewShot {
    type Report = BootstrapReport;

    async fn compile<S, M, MT>(
        &self,
        module: &mut M,
        trainset: Vec<Example<S>>,
        metric: &MT,
    ) -> Result<Self::Report>
    where
        S: Signature,
        S::Input: Clone,
        M: Module<Input = S::Input> + for<'a> Facet<'a>,
        MT: TypedMetric<S, M>,
    {
        let names = predictor_names(module)?;
        if names.is_empty() {
            return Err(anyhow!("no optimizable predictors found"));
        }

        // Seed labeled demos.
        let labeled_raw: Vec<RawExample> = trainset
            .iter()
            .take(self.max_labeled_demos)
            .map(raw_from_typed_example)
            .collect::<Result<_>>()?;
        let labeled = labeled_raw.len();

        for name in &names {
            with_named_predictor(module, name, |predictor| {
                assign_demos_best_effort(predictor, labeled_raw.clone())
            })?;
        }

        let mut bootstrapped_raw: Vec<RawExample> = Vec::new();
        let mut scores = Vec::new();

        for example in &trainset {
            if bootstrapped_raw.len() >= self.max_bootstrapped_demos {
                break;
            }

            let predicted = module
                .call(example.input.clone())
                .await
                .map_err(|err| anyhow!("{err}"))?;
            let outcome = metric.evaluate(example, &predicted).await?;
            scores.push(outcome.score);

            if outcome.score >= self.metric_threshold {
                // Prefer gold labels for demos (stable); prediction success gates inclusion.
                bootstrapped_raw.push(raw_from_typed_example(example)?);
            }
        }

        // Merge labeled + bootstrapped (dedupe by JSON) onto each predictor.
        let mut merged = labeled_raw;
        for demo in bootstrapped_raw.iter() {
            if !merged.iter().any(|d| d.data == demo.data) {
                merged.push(demo.clone());
            }
        }

        for name in &names {
            with_named_predictor(module, name, |predictor| {
                assign_demos_best_effort(predictor, merged.clone())
            })?;
        }

        let average_train_score = if scores.is_empty() {
            evaluate_module_with_metric(&*module, &trainset, metric)
                .await
                .map(|o| average_score(&o))
                .unwrap_or(0.0)
        } else {
            scores.iter().sum::<f32>() / scores.len() as f32
        };

        Ok(BootstrapReport {
            labeled,
            bootstrapped: bootstrapped_raw.len(),
            average_train_score,
        })
    }
}

fn raw_from_typed_example<S: Signature>(example: &Example<S>) -> Result<RawExample>
where
    S::Input: BamlType,
    S::Output: BamlType,
{
    let input_value = serde_json::to_value(example.input.to_baml_value())?;
    let output_value = serde_json::to_value(example.output.to_baml_value())?;

    let input_map = input_value
        .as_object()
        .ok_or_else(|| anyhow!("expected object for signature input"))?
        .clone();
    let output_map = output_value
        .as_object()
        .ok_or_else(|| anyhow!("expected object for signature output"))?
        .clone();

    let input_keys: Vec<String> = input_map.keys().cloned().collect();
    let output_keys: Vec<String> = output_map.keys().cloned().collect();

    let mut data = std::collections::HashMap::new();
    data.extend(input_map);
    data.extend(output_map);

    Ok(RawExample::new(data, input_keys, output_keys))
}

fn assign_demos_best_effort(predictor: &mut dyn DynPredictor, demos: Vec<RawExample>) -> Result<()> {
    // Try full set; if schema mismatch, try one-by-one.
    if predictor.set_demos_from_examples(demos.clone()).is_ok() {
        return Ok(());
    }

    let mut accepted = Vec::new();
    for demo in demos {
        let mut trial = accepted.clone();
        trial.push(demo.clone());
        if predictor.set_demos_from_examples(trial).is_ok() {
            accepted.push(demo);
        }
    }
    predictor.set_demos_from_examples(accepted)?;
    Ok(())
}
