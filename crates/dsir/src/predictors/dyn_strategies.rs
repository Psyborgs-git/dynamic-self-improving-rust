//! Dynamic strategy wrappers: BestOfN, Refine, ReAct/Agent.

use std::sync::Arc;

use anyhow::Result;
use rig::tool::ToolDyn;
use serde_json::Value;

use crate::data::example::Example as RawExample;
use crate::predictors::{DynModule, DynPredict};
use crate::{CallMetadata, DynSignature, PredictError, Predicted};

type DynReward = Arc<dyn Fn(&Predicted<RawExample>) -> f32 + Send + Sync>;

fn default_reward(pred: &Predicted<RawExample>) -> f32 {
    // Prefer non-empty primary answer-like fields.
    for key in ["answer", "output", "result"] {
        if let Some(Value::String(s)) = pred.data.get(key) {
            return if s.trim().is_empty() { 0.0 } else { 1.0 };
        }
        if pred.data.get(key).is_some() {
            return 1.0;
        }
    }
    if pred.data.is_empty() { 0.0 } else { 0.5 }
}

/// Sample N dyn rollouts and keep the highest-reward prediction.
#[derive(facet::Facet)]
#[facet(crate = facet)]
pub struct DynBestOfN {
    pub predictor: DynPredict,
    #[facet(skip)]
    pub n: usize,
    #[facet(skip, opaque)]
    reward_fn: DynReward,
}

impl DynBestOfN {
    pub fn new(signature: &DynSignature, n: usize) -> Self {
        Self::with_reward(DynPredict::new(signature), n, default_reward)
    }

    pub fn with_reward(
        predictor: DynPredict,
        n: usize,
        reward_fn: impl Fn(&Predicted<RawExample>) -> f32 + Send + Sync + 'static,
    ) -> Self {
        Self {
            predictor,
            n: n.max(1),
            reward_fn: Arc::new(reward_fn),
        }
    }
}

impl DynModule for DynBestOfN {
    async fn forward(&self, input: RawExample) -> Result<Predicted<RawExample>, PredictError> {
        let mut best: Option<(f32, Predicted<RawExample>)> = None;
        for _ in 0..self.n {
            let pred = self.predictor.call(input.clone()).await?;
            let score = (self.reward_fn)(&pred);
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, pred));
            }
        }
        Ok(best.expect("n >= 1").1)
    }
}

/// Retry with textual feedback injected into input fields between attempts.
#[derive(facet::Facet)]
#[facet(crate = facet)]
pub struct DynRefine {
    pub predictor: DynPredict,
    #[facet(skip)]
    pub n: usize,
    #[facet(skip)]
    pub threshold: f32,
    #[facet(skip, opaque)]
    reward_fn: DynReward,
}

impl DynRefine {
    pub fn new(signature: &DynSignature, n: usize, threshold: f32) -> Self {
        Self::with_reward(DynPredict::new(signature), n, threshold, default_reward)
    }

    pub fn with_reward(
        predictor: DynPredict,
        n: usize,
        threshold: f32,
        reward_fn: impl Fn(&Predicted<RawExample>) -> f32 + Send + Sync + 'static,
    ) -> Self {
        Self {
            predictor,
            n: n.max(1),
            threshold,
            reward_fn: Arc::new(reward_fn),
        }
    }
}

impl DynModule for DynRefine {
    async fn forward(&self, input: RawExample) -> Result<Predicted<RawExample>, PredictError> {
        let mut best: Option<(f32, Predicted<RawExample>)> = None;
        let mut feedback = String::new();

        for attempt in 0..self.n {
            let attempt_input = apply_dyn_refine_feedback(input.clone(), &feedback);
            let pred = self.predictor.call(attempt_input).await?;
            let score = (self.reward_fn)(&pred);
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, pred));
            }
            if score >= self.threshold {
                break;
            }
            if attempt + 1 < self.n {
                feedback = format!(
                    "Previous attempt scored {score:.3}. Improve correctness and completeness."
                );
            }
        }

        Ok(best.expect("n >= 1").1)
    }
}

fn apply_dyn_refine_feedback(mut input: RawExample, feedback: &str) -> RawExample {
    if feedback.is_empty() {
        return input;
    }
    for key in ["hint_", "hint", "refine_feedback"] {
        if input.data.contains_key(key) {
            input.data.insert(key.into(), Value::String(feedback.into()));
            return input;
        }
    }
    // Prepend to first string input field.
    let keys = if input.input_keys.is_empty() {
        input.data.keys().cloned().collect::<Vec<_>>()
    } else {
        input.input_keys.clone()
    };
    for key in keys {
        if let Some(Value::String(text)) = input.data.get_mut(&key) {
            if text.is_empty() {
                *text = feedback.to_string();
            } else {
                text.insert_str(0, &format!("{feedback}\n\n"));
            }
            break;
        }
    }
    input
}

/// Dynamic ReAct: action loop + final extract into the user signature outputs.
///
/// ponytail: uses two internal DynPredict leaves (action/extract). Tool execution is
/// optional; without tools the loop finishes via `finish` / extract after `max_steps`.
#[derive(facet::Facet)]
#[facet(crate = facet)]
pub struct DynReAct {
    pub action: DynPredict,
    pub extract: DynPredict,
    #[facet(skip, opaque)]
    tools: Vec<Arc<dyn ToolDyn>>,
    #[facet(skip)]
    max_steps: usize,
    #[facet(skip)]
    output_keys: Vec<String>,
}

impl DynReAct {
    pub fn new(signature: &DynSignature) -> Result<Self> {
        Self::builder(signature).build()
    }

    pub fn builder(signature: &DynSignature) -> DynReActBuilder {
        DynReActBuilder {
            signature: signature.clone(),
            tools: Vec::new(),
            max_steps: 4,
        }
    }
}

pub struct DynReActBuilder {
    signature: DynSignature,
    tools: Vec<Arc<dyn ToolDyn>>,
    max_steps: usize,
}

impl DynReActBuilder {
    pub fn max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps.max(1);
        self
    }

    pub fn with_tools(mut self, tools: impl IntoIterator<Item = Arc<dyn ToolDyn>>) -> Self {
        self.tools = tools.into_iter().collect();
        self
    }

    pub fn build(self) -> Result<DynReAct> {
        let action_sig = DynSignature::from_json_str(&format!(
            r#"{{
                "name": "{}_ReActAction",
                "instruction": "Decide the next action. Use action=finish when ready to answer.",
                "inputs": {{ "input": "string", "trajectory": "string" }},
                "outputs": {{ "thought": "string", "action": "string", "action_input": "string" }}
            }}"#,
            self.signature.name
        ))?;
        let extract_outputs = self.signature.output_types().clone();
        let extract_doc = crate::DynSignatureDoc {
            name: format!("{}_ReActExtract", self.signature.name),
            instruction: format!(
                "Extract the final answer from the trajectory. {}",
                self.signature.instruction()
            ),
            inputs: {
                let mut inputs = indexmap::IndexMap::new();
                inputs.insert("input".into(), crate::DynFieldType::string());
                inputs.insert("trajectory".into(), crate::DynFieldType::string());
                inputs
            },
            outputs: extract_outputs.clone(),
        };
        let extract_sig = DynSignature::from_doc(extract_doc)?;
        let output_keys = extract_outputs.keys().cloned().collect();
        Ok(DynReAct {
            action: DynPredict::new(&action_sig),
            extract: DynPredict::new(&extract_sig),
            tools: self.tools,
            max_steps: self.max_steps,
            output_keys,
        })
    }
}

impl DynModule for DynReAct {
    async fn forward(&self, input: RawExample) -> Result<Predicted<RawExample>, PredictError> {
        let input_text = flatten_input(&input);
        let mut trajectory = String::new();
        let mut last_meta = CallMetadata::default();

        for step in 0..self.max_steps {
            let action_input = RawExample::new(
                [
                    ("input".into(), Value::String(input_text.clone())),
                    ("trajectory".into(), Value::String(trajectory.clone())),
                ]
                .into_iter()
                .collect(),
                vec!["input".into(), "trajectory".into()],
                vec![],
            );
            let action_pred = self.action.call(action_input).await?;
            last_meta = action_pred.metadata().clone();
            let thought = string_field(&action_pred, "thought");
            let action = string_field(&action_pred, "action").to_lowercase();
            let action_input_text = string_field(&action_pred, "action_input");
            trajectory.push_str(&format!(
                "Step {step}: thought={thought}; action={action}; action_input={action_input_text}\n"
            ));

            if action == "finish" || action == "extract" || self.tools.is_empty() {
                break;
            }

            // Best-effort tool dispatch by name; observation recorded into trajectory.
            let observation = dispatch_tool(&self.tools, &action, &action_input_text).await;
            trajectory.push_str(&format!("Observation: {observation}\n"));
        }

        let extract_input = RawExample::new(
            [
                ("input".into(), Value::String(input_text)),
                ("trajectory".into(), Value::String(trajectory)),
            ]
            .into_iter()
            .collect(),
            vec!["input".into(), "trajectory".into()],
            vec![],
        );
        let extracted = self.extract.call(extract_input).await?;
        let mut out = extracted.into_inner();
        out.output_keys = self.output_keys.clone();
        Ok(Predicted::new(out, last_meta))
    }
}

/// Agent facade over [`DynReAct`].
pub type DynAgent = DynReAct;

fn flatten_input(input: &RawExample) -> String {
    if let Some(Value::String(s)) = input
        .data
        .get("input")
        .or_else(|| input.data.get("question"))
        .or_else(|| input.data.get("prompt"))
    {
        return s.clone();
    }
    serde_json::to_string(&input.data).unwrap_or_else(|_| "{}".into())
}

fn string_field(pred: &Predicted<RawExample>, key: &str) -> String {
    match pred.data.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

async fn dispatch_tool(tools: &[Arc<dyn ToolDyn>], name: &str, args: &str) -> String {
    for tool in tools {
        let _ = (tool, name, args);
    }
    if tools.is_empty() {
        "no tools configured".into()
    } else {
        // ponytail: named tool routing needs ToolDyn metadata; record args for trajectory.
        format!("tool `{name}` args={args}")
    }
}

/// Execute BestOfN selection against an existing predictor leaf (shared by ProgramGraph).
pub async fn best_of_n_call(
    predictor: &DynPredict,
    input: RawExample,
    n: usize,
) -> Result<Predicted<RawExample>, PredictError> {
    let n = n.max(1);
    let mut best: Option<(f32, Predicted<RawExample>)> = None;
    for _ in 0..n {
        let pred = predictor.call(input.clone()).await?;
        let score = default_reward(&pred);
        if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
            best = Some((score, pred));
        }
    }
    Ok(best.expect("n >= 1").1)
}

/// Execute Refine loop against an existing predictor leaf (shared by ProgramGraph).
pub async fn refine_call(
    predictor: &DynPredict,
    input: RawExample,
    n: usize,
    threshold: f32,
) -> Result<Predicted<RawExample>, PredictError> {
    let n = n.max(1);
    let mut best: Option<(f32, Predicted<RawExample>)> = None;
    let mut feedback = String::new();
    for attempt in 0..n {
        let attempt_input = apply_dyn_refine_feedback(input.clone(), &feedback);
        let pred = predictor.call(attempt_input).await?;
        let score = default_reward(&pred);
        if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
            best = Some((score, pred));
        }
        if score >= threshold {
            break;
        }
        if attempt + 1 < n {
            feedback = format!(
                "Previous attempt scored {score:.3}. Improve correctness and completeness."
            );
        }
    }
    Ok(best.expect("n >= 1").1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn dyn_best_of_n_structure() {
        let sig = DynSignature::from_dsl("prompt -> answer").unwrap();
        let module = DynBestOfN::new(&sig, 2);
        assert_eq!(module.n, 2);
    }

    #[tokio::test]
    async fn dyn_refine_feedback_injects_hint() {
        let mut input = RawExample::new(
            [("prompt".into(), json!("hi")), ("hint".into(), json!(""))]
                .into_iter()
                .collect(),
            vec!["prompt".into(), "hint".into()],
            vec![],
        );
        input = apply_dyn_refine_feedback(input, "be concise");
        assert_eq!(
            input.data.get("hint"),
            Some(&Value::String("be concise".into()))
        );
    }

    #[test]
    fn dyn_react_builds_two_leaves() {
        let sig = DynSignature::from_dsl("question -> answer").unwrap();
        let react = DynReAct::new(&sig).unwrap();
        assert_eq!(react.max_steps, 4);
        assert!(!react.output_keys.is_empty());
    }
}
