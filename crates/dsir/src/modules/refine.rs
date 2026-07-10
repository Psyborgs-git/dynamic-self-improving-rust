//! Retry with LM-generated feedback between attempts (BestOfN + refine).

use std::sync::Arc;

use crate::core::lm::chat::{Chat, Message};
use crate::core::settings::GLOBAL_SETTINGS;
use crate::core::{ConversionError, Module};
use crate::{BamlType, BamlValue, Facet, PredictError, Predicted};

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
            let attempt_input = apply_refine_feedback(input.clone(), &feedback)?;
            let pred = self.module.call(attempt_input).await?;
            let score = (self.reward_fn)(&pred);

            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, pred));
            }

            if score >= self.threshold {
                break;
            }

            if attempt + 1 < self.n {
                feedback = propose_feedback(&feedback, score).await;
            }
        }

        Ok(best
            .expect("n >= 1 guarantees at least one successful candidate")
            .1)
    }
}

/// Injects `feedback` into a module input for the next refine attempt.
///
/// When the input serializes to a map, a `hint_` / `hint` / `refine_feedback` field is
/// updated when present; otherwise feedback is prepended to the first string field.
fn apply_refine_feedback<T: BamlType>(input: T, feedback: &str) -> Result<T, PredictError> {
    if feedback.is_empty() {
        return Ok(input);
    }

    let mut value = input.to_baml_value();
    let map = match &mut value {
        BamlValue::Map(map) | BamlValue::Class(_, map) => map,
        _ => return Ok(input),
    };

    for key in ["hint_", "hint", "refine_feedback"] {
        if map.contains_key(key) {
            map.insert(key.to_string(), BamlValue::String(feedback.to_string()));
            return T::try_from_baml_value(value.clone()).map_err(|source| PredictError::Conversion {
                source: source.into(),
                parsed: value,
            });
        }
    }

    for field in map.values_mut() {
        if let BamlValue::String(text) = field {
            if !text.is_empty() {
                text.insert_str(0, &format!("{feedback}\n\n"));
            } else {
                *text = feedback.to_string();
            }
            return T::try_from_baml_value(value.clone()).map_err(|source| PredictError::Conversion {
                source: source.into(),
                parsed: value,
            });
        }
    }

    Err(PredictError::Conversion {
        source: ConversionError::TypeMismatch {
            expected: "map input with a hint or string field",
            actual: format!("{value:?}"),
        },
        parsed: value,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use dsir_macros::Signature;

    #[derive(Signature, Clone, Debug)]
    #[allow(dead_code)]
    struct WithHint {
        #[input]
        question: String,

        #[input]
        hint_: String,

        #[output]
        answer: String,
    }

    #[derive(Signature, Clone, Debug)]
    struct Plain {
        #[input]
        prompt: String,

        #[output]
        answer: String,
    }

    #[test]
    fn refine_feedback_updates_hint_field() {
        let input = WithHintInput {
            question: "q".into(),
            hint_: String::new(),
        };
        let updated =
            apply_refine_feedback(input, "try again").expect("hint field should accept feedback");
        assert_eq!(updated.hint_, "try again");
        assert_eq!(updated.question, "q");
    }

    #[test]
    fn refine_feedback_prepends_to_first_string_field() {
        let input = PlainInput {
            prompt: "original".into(),
        };
        let updated =
            apply_refine_feedback(input, "critique").expect("string field should accept feedback");
        assert_eq!(updated.prompt, "critique\n\noriginal");
    }
}
