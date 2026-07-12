//! Shared helpers for dynamic (RawExample) optimizer compilation.

use anyhow::{Result, anyhow};

use crate::core::DynPredictor;
use crate::data::example::Example as RawExample;
use crate::evaluate::{
    DynMetric, MetricOutcome, average_score, evaluate_dyn_trainset, input_only,
};
use crate::optimizer::{predictor_names, with_named_predictor};
use crate::predictors::DynModule;
use crate::Facet;

pub(crate) async fn evaluate_dyn_module_with_metric<M, MT>(
    module: &M,
    trainset: &[RawExample],
    metric: &MT,
) -> Result<Vec<MetricOutcome>>
where
    M: DynModule,
    MT: DynMetric,
{
    evaluate_dyn_trainset(module, trainset, metric).await
}

pub(crate) fn assign_demos_best_effort(
    predictor: &mut dyn DynPredictor,
    demos: Vec<RawExample>,
) -> Result<()> {
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

pub(crate) async fn score_instruction_dyn<M, MT>(
    module: &mut M,
    predictor_name: &str,
    instruction: &str,
    trainset: &[RawExample],
    metric: &MT,
) -> Result<f32>
where
    M: DynModule + for<'a> Facet<'a>,
    MT: DynMetric,
{
    let original_state =
        with_named_predictor(module, predictor_name, |predictor| Ok(predictor.dump_state()))?;
    with_named_predictor(module, predictor_name, |predictor| {
        predictor.set_instruction(instruction.to_string());
        Ok(())
    })?;
    let evaluation = evaluate_dyn_module_with_metric(&*module, trainset, metric).await;
    match evaluation {
        Ok(outcomes) => {
            with_named_predictor(module, predictor_name, |predictor| {
                predictor.load_state(original_state)
            })?;
            Ok(average_score(&outcomes))
        }
        Err(eval_err) => {
            let _ = with_named_predictor(module, predictor_name, |predictor| {
                predictor.load_state(original_state)
            });
            Err(eval_err)
        }
    }
}

pub(crate) fn ensure_predictors<M>(module: &mut M) -> Result<Vec<String>>
where
    M: for<'a> Facet<'a>,
{
    let names = predictor_names(module)?;
    if names.is_empty() {
        return Err(anyhow!("no optimizable predictors found"));
    }
    Ok(names)
}

pub(crate) fn merge_unique(mut base: Vec<RawExample>, extra: Vec<RawExample>) -> Vec<RawExample> {
    for demo in extra {
        if !base.iter().any(|d| d.data == demo.data) {
            base.push(demo);
        }
    }
    base
}

pub(crate) fn example_as_demo(example: &RawExample) -> RawExample {
    example.clone()
}

pub(crate) async fn run_and_score_dyn<M, MT>(
    module: &M,
    example: &RawExample,
    metric: &MT,
) -> Result<MetricOutcome>
where
    M: DynModule,
    MT: DynMetric,
{
    let predicted = module
        .call(input_only(example))
        .await
        .map_err(|err| anyhow!("{err}"))?;
    metric.evaluate(example, &predicted).await
}
