//! Strategy factory for dynamic modules.

use anyhow::Result;

use crate::DynSignature;
use crate::predictors::{DynChainOfThought, DynPredict};

/// Which prompting strategy to instantiate for a dynamic signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StrategyKind {
    #[default]
    Predict,
    ChainOfThought,
}

impl StrategyKind {
    pub fn all() -> &'static [StrategyKind] {
        &[StrategyKind::Predict, StrategyKind::ChainOfThought]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Predict => "predict",
            Self::ChainOfThought => "chain_of_thought",
        }
    }
}

/// Builds dyn strategy leaves from a [`DynSignature`].
#[derive(Debug, Clone, Default)]
pub struct StrategyFactory;

impl StrategyFactory {
    pub fn create_predict(signature: &DynSignature) -> DynPredict {
        DynPredict::new(signature)
    }

    pub fn create_chain_of_thought(signature: &DynSignature) -> Result<DynChainOfThought> {
        DynChainOfThought::new(signature)
    }

    pub fn create_predict_for_kind(
        signature: &DynSignature,
        kind: StrategyKind,
    ) -> Result<DynPredict> {
        match kind {
            StrategyKind::Predict => Ok(Self::create_predict(signature)),
            StrategyKind::ChainOfThought => {
                let cot = Self::create_chain_of_thought(signature)?;
                Ok(cot.predictor)
            }
        }
    }
}
