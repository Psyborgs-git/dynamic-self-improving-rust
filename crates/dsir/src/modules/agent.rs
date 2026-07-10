//! DSPy-shaped agent facade over [`ReAct`](crate::ReAct).

use std::sync::Arc;

use facet::Facet;
use rig::tool::ToolDyn;

use crate::core::{Module, Signature};
use crate::modules::react::ReAct;
use crate::{BamlType, PredictError, Predicted};

/// Tool-using agent with a clear DSPy-like constructor.
///
/// Thin facade over [`ReAct`]: configure tools and `max_iters`, then `call` with the
/// signature input. Prefer this when you want an "agent" API; use `ReAct` directly when
/// you need builder-level control over action/extract predictors.
#[derive(Facet)]
#[facet(crate = facet)]
pub struct Agent<S>
where
    S: Signature,
    S::Input: BamlType + Clone,
    S::Output: BamlType,
{
    react: ReAct<S>,
}

impl<S> Agent<S>
where
    S: Signature,
    S::Input: BamlType + Clone,
    S::Output: BamlType,
{
    pub fn new(tools: Vec<Arc<dyn ToolDyn>>, max_iters: usize) -> Self {
        let react = ReAct::<S>::builder()
            .with_tools(tools)
            .max_steps(max_iters.max(1))
            .build();
        Self { react }
    }

    pub fn from_react(react: ReAct<S>) -> Self {
        Self { react }
    }

    pub fn into_react(self) -> ReAct<S> {
        self.react
    }
}

impl<S> Module for Agent<S>
where
    S: Signature,
    S::Input: BamlType + Clone + for<'a> Facet<'a> + Send + Sync,
    S::Output: BamlType + for<'a> Facet<'a> + Send + Sync,
{
    type Input = S::Input;
    type Output = S::Output;

    async fn forward(&self, input: Self::Input) -> Result<Predicted<Self::Output>, PredictError> {
        self.react.forward(input).await
    }
}
