//! Strategy factory for dynamic modules.

use anyhow::{Result, bail};

use crate::DynSignature;
use crate::predictors::{
    DynBestOfN, DynChainOfThought, DynPredict, DynReAct, DynRefine,
};

/// Which prompting strategy to instantiate for a dynamic signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StrategyKind {
    #[default]
    Predict,
    ChainOfThought,
    BestOfN,
    Refine,
    ReAct,
    Agent,
}

impl StrategyKind {
    pub fn all() -> &'static [StrategyKind] {
        &[
            StrategyKind::Predict,
            StrategyKind::ChainOfThought,
            StrategyKind::BestOfN,
            StrategyKind::Refine,
            StrategyKind::ReAct,
            StrategyKind::Agent,
        ]
    }

    /// Strategies that structural search can swap while keeping a single leaf schema.
    pub fn structural_candidates() -> &'static [StrategyKind] {
        &[
            StrategyKind::Predict,
            StrategyKind::ChainOfThought,
            StrategyKind::BestOfN,
            StrategyKind::Refine,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Predict => "predict",
            Self::ChainOfThought => "chain_of_thought",
            Self::BestOfN => "best_of_n",
            Self::Refine => "refine",
            Self::ReAct => "react",
            Self::Agent => "agent",
        }
    }
}

/// Builds dyn strategy modules from a [`DynSignature`].
#[derive(Debug, Clone, Default)]
pub struct StrategyFactory;

impl StrategyFactory {
    pub fn create_predict(signature: &DynSignature) -> DynPredict {
        DynPredict::new(signature)
    }

    pub fn create_chain_of_thought(signature: &DynSignature) -> Result<DynChainOfThought> {
        DynChainOfThought::new(signature)
    }

    pub fn create_best_of_n(signature: &DynSignature, n: usize) -> DynBestOfN {
        DynBestOfN::new(signature, n)
    }

    pub fn create_refine(signature: &DynSignature, n: usize, threshold: f32) -> DynRefine {
        DynRefine::new(signature, n, threshold)
    }

    pub fn create_react(signature: &DynSignature) -> Result<DynReAct> {
        DynReAct::new(signature)
    }

    pub fn create_agent(signature: &DynSignature) -> Result<DynReAct> {
        DynReAct::new(signature)
    }

    /// Create the optimizable [`DynPredict`] leaf used inside [`crate::ProgramGraph`] nodes.
    ///
    /// BestOfN/Refine keep the base signature leaf (selection happens at execute time).
    /// CoT augments the schema with `reasoning`. ReAct/Agent need two leaves — use
    /// [`create_react`] / [`create_agent`] for standalone modules.
    pub fn create_predict_for_kind(
        signature: &DynSignature,
        kind: StrategyKind,
    ) -> Result<DynPredict> {
        match kind {
            StrategyKind::Predict | StrategyKind::BestOfN | StrategyKind::Refine => {
                Ok(Self::create_predict(signature))
            }
            StrategyKind::ChainOfThought => {
                let cot = Self::create_chain_of_thought(signature)?;
                Ok(cot.predictor)
            }
            StrategyKind::ReAct | StrategyKind::Agent => bail!(
                "strategy `{kind:?}` needs multi-leaf DynReAct; use StrategyFactory::create_react \
                 (or create_agent) instead of a single ProgramGraph leaf"
            ),
        }
    }
}
