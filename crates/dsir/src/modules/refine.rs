//! Retry with LM-generated feedback between attempts (BestOfN + refine).

use std::sync::Arc;

use crate::core::lm::chat::{Chat, Message};
use crate::core::settings::GLOBAL_SETTINGS;
use crate::core::Module;
use crate::{BamlType, Facet, PredictError, Predicted};

/// Like [`BestOfN`](crate::BestOfN), but after a failed attempt asks the LM for feedback
/// used to decide early stopping when `reward_fn` returns a score ≥ `threshold`.
pub struct Refine<M>
where
    M: Module,
{
    pub module: M,
    pub n: usize,
    pub threshold: f32,
    reward_fn: Arc<dyn Fn(&Predicted<M::Output>) -> f32 + Send + Sync>,
}

impl<M> Refine<M>
where
    M: Module,
    M::Input: Clone,
{
    pub fn new(
        module: M,
        n: usize,
        threshold: f32,
        reward_fn: impl Fn(&Predicted<M::Output>) -> f32 + Send + Sync + 'static,
    ) -> Self {
        Self {
            module,
            n: n.max(1),
            threshold,
            reward_fn: Arc::new(reward_fn),
        }
    }
}

impl<M> Module for Refine<M>
where
    M: Module,
    M::Input: Clone + BamlType + for<'a> Facet<'a> + Send + Sync,
    M::Output: BamlType + for<'a> Facet<'a> + Send + Sync,
{
    type Input = M::Input;
    type Output = M::Output;

    async fn forward(&self, input: Self::Input) -> Result<Predicted<Self::Output>, PredictError> {
        let mut best: Option<(f32, Predicted<Self::Output>)> = None;
        let mut feedback = String::new();

        for attempt in 0..self.n {
            let pred = self.module.call(input.clone()).await?;
            let score = (self.reward_fn)(&pred);

            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, pred));
            }

            if score >= self.threshold {
                break;
            }

            if attempt + 1 < self.n {
                feedback = propose_feedback(&feedback, score).await;
                let _ = &feedback;
            }
        }

        Ok(best
            .expect("n >= 1 guarantees at least one successful candidate")
            .1)
    }
}

async fn propose_feedback(prior: &str, score: f32) -> String {
    let lm = GLOBAL_SETTINGS
        .read()
        .ok()
        .and_then(|g| g.as_ref().map(|s| (*s.lm).clone()));

    let Some(lm) = lm else {
        return format!(
            "Previous attempt scored {score:.3}; improve correctness and completeness."
        );
    };

    let mut chat = Chat::new(vec![Message::system(
        "You write short critique for improving an LM program attempt. Reply with one paragraph.",
    )]);
    chat.push_message(Message::user(format!(
        "Prior feedback:\n{prior}\n\nLatest score: {score:.3}\nWrite concise feedback for the next attempt."
    )));

    match lm.call(chat, vec![]).await {
        Ok(response) => response.output.content().to_string(),
        Err(_) => format!(
            "Previous attempt scored {score:.3}; improve correctness and completeness."
        ),
    }
}
