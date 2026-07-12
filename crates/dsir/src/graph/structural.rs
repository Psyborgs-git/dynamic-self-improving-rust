//! Discrete structural optimization over strategy choices.

use anyhow::Result;

use crate::core::DynPredictor;
use crate::data::example::Example as RawExample;
use crate::evaluate::{DynMetric, average_score, evaluate_dyn_trainset};
use crate::graph::{GraphModule, StrategyFactory, StrategyKind};
use crate::DynSignature;

/// Report from [`StructuralOptimizer`].
#[derive(Debug, Clone)]
pub struct StructuralReport {
    pub best_strategy: StrategyKind,
    pub baseline_score: f32,
    pub best_score: f32,
    pub trials: Vec<(StrategyKind, f32)>,
}

/// Try a small discrete set of strategies and keep the best on the trainset.
///
/// ponytail: O(|strategies| × |train|) exhaustive swap — not open-ended NAS.
/// Upgrade path: beam search over multi-node graphs / edge rewires.
#[derive(Debug, Clone, Default)]
pub struct StructuralOptimizer {
    pub strategies: Vec<StrategyKind>,
}

impl StructuralOptimizer {
    pub fn new() -> Self {
        Self {
            strategies: StrategyKind::structural_candidates().to_vec(),
        }
    }

    pub async fn compile(
        &self,
        signature: &DynSignature,
        module: &mut GraphModule,
        trainset: Vec<RawExample>,
        metric: &impl DynMetric,
    ) -> Result<StructuralReport> {
        let strategies = if self.strategies.is_empty() {
            StrategyKind::structural_candidates().to_vec()
        } else {
            self.strategies.clone()
        };

        let baseline = evaluate_dyn_trainset(module, &trainset, metric).await?;
        let baseline_score = average_score(&baseline);

        let mut trials = Vec::new();
        let mut best_strategy = StrategyKind::Predict;
        let mut best_score = f32::NEG_INFINITY;
        let mut best_state;

        for kind in strategies {
            let mut candidate = GraphModule::from_signature(signature, kind)?;
            // Carry over instruction/demos when schemas are compatible.
            let _ = DynPredictor::load_state(
                &mut candidate.predictor,
                DynPredictor::dump_state(&module.predictor),
            );
            let outcomes = evaluate_dyn_trainset(&candidate, &trainset, metric).await?;
            let score = average_score(&outcomes);
            trials.push((kind, score));
            if score > best_score {
                best_score = score;
                best_strategy = kind;
                best_state = DynPredictor::dump_state(&candidate.predictor);
                // Ensure leaf matches chosen strategy schema (esp. CoT reasoning field).
                module.predictor = StrategyFactory::create_predict_for_kind(signature, kind)?;
                module.strategy = kind;
                DynPredictor::load_state(&mut module.predictor, best_state.clone())?;
            }
        }

        Ok(StructuralReport {
            best_strategy,
            baseline_score,
            best_score,
            trials,
        })
    }
}
