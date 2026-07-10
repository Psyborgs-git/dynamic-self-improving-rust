//! Sample N rollouts and keep the best by a reward function.

use std::sync::Arc;

use crate::core::Module;
use crate::{BamlType, Facet, PredictError, Predicted};

/// Runs an inner module `n` times and returns the prediction with the highest reward.
///
/// Does not change the prompt — only selection. Bookkeeping stays on [`CallMetadata`](crate::CallMetadata).
pub struct BestOfN<M>
where
    M: Module,
{
    pub module: M,
    pub n: usize,
    reward_fn: Arc<dyn Fn(&Predicted<M::Output>) -> f32 + Send + Sync>,
}

impl<M> BestOfN<M>
where
    M: Module,
    M::Input: Clone,
{
    pub fn new(
        module: M,
        n: usize,
        reward_fn: impl Fn(&Predicted<M::Output>) -> f32 + Send + Sync + 'static,
    ) -> Self {
        Self {
            module,
            n: n.max(1),
            reward_fn: Arc::new(reward_fn),
        }
    }
}

impl<M> Module for BestOfN<M>
where
    M: Module,
    M::Input: Clone + BamlType + for<'a> Facet<'a> + Send + Sync,
    M::Output: BamlType + for<'a> Facet<'a> + Send + Sync,
{
    type Input = M::Input;
    type Output = M::Output;

    async fn forward(&self, input: Self::Input) -> Result<Predicted<Self::Output>, PredictError> {
        let mut best: Option<(f32, Predicted<Self::Output>)> = None;

        for _ in 0..self.n {
            let pred = self.module.call(input.clone()).await?;
            let score = (self.reward_fn)(&pred);
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, pred));
            }
        }

        Ok(best
            .expect("n >= 1 guarantees at least one successful candidate")
            .1)
    }
}
