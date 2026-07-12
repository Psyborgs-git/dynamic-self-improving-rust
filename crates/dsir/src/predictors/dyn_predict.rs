//! Dynamic (runtime-schema) predictors and modules.

use std::ops::ControlFlow;
use std::sync::Arc;

use anyhow::Result;
use rig::tool::ToolDyn;
use tracing::trace;

use crate as dsrs;
use crate::core::settings::GLOBAL_SETTINGS;
use crate::core::{DynPredictor, PredictAccessorFns, PredictState, SignatureSchema};
use crate::data::example::Example as RawExample;
use crate::{CallMetadata, Chat, DynSignature, LmError, PredictError, Predicted};

/// Type-erased module: `RawExample` in, `RawExample` out.
#[allow(async_fn_in_trait)]
pub trait DynModule: Send + Sync {
    async fn forward(&self, input: RawExample) -> Result<Predicted<RawExample>, PredictError>;

    async fn call(&self, input: RawExample) -> Result<Predicted<RawExample>, PredictError> {
        self.forward(input).await
    }
}

fn visit_dyn_predict_mut(
    this: *mut (),
    visitor: &mut dyn FnMut(&mut dyn DynPredictor) -> ControlFlow<()>,
) -> ControlFlow<()> {
    // SAFETY: pointer originates from Facet walk over `DynPredict`.
    let predictor = unsafe { &mut *(this as *mut DynPredict) };
    visitor(predictor)
}

trait DynPredictAccessorProvider {
    const VISIT_MUT: fn(
        *mut (),
        &mut dyn FnMut(&mut dyn DynPredictor) -> ControlFlow<()>,
    ) -> ControlFlow<()>;
}

impl DynPredictAccessorProvider for DynPredict {
    const VISIT_MUT: fn(
        *mut (),
        &mut dyn FnMut(&mut dyn DynPredictor) -> ControlFlow<()>,
    ) -> ControlFlow<()> = visit_dyn_predict_mut;
}

/// Runtime-schema LM leaf (dynamic counterpart of [`crate::Predict`]).
#[derive(facet::Facet)]
#[facet(crate = facet, opaque)]
#[facet(dsrs::predict_accessor = &PredictAccessorFns {
    visit_mut: <DynPredict as DynPredictAccessorProvider>::VISIT_MUT,
})]
pub struct DynPredict {
    #[facet(skip, opaque)]
    schema: Arc<SignatureSchema>,
    #[facet(skip, opaque)]
    signature_name: String,
    #[facet(skip, opaque)]
    default_instruction: String,
    #[facet(skip, opaque)]
    input_keys: Vec<String>,
    #[facet(skip, opaque)]
    output_keys: Vec<String>,
    #[facet(skip, opaque)]
    tools: Vec<Arc<dyn ToolDyn>>,
    #[facet(skip, opaque)]
    demos: Vec<RawExample>,
    instruction_override: Option<String>,
    #[facet(skip, opaque)]
    lm: Option<Arc<crate::core::LM>>,
}

impl DynPredict {
    pub fn new(signature: &DynSignature) -> Self {
        Self {
            schema: signature.schema_arc(),
            signature_name: signature.name.clone(),
            default_instruction: signature.instruction().to_string(),
            input_keys: signature.input_keys().to_vec(),
            output_keys: signature.output_keys().to_vec(),
            tools: Vec::new(),
            demos: Vec::new(),
            instruction_override: None,
            lm: None,
        }
    }

    pub fn with_instruction(mut self, instruction: impl Into<String>) -> Self {
        self.instruction_override = Some(instruction.into());
        self
    }

    pub fn with_demo(mut self, demo: RawExample) -> Self {
        self.demos.push(demo);
        self
    }

    pub fn with_lm(mut self, lm: Arc<crate::core::LM>) -> Self {
        self.lm = Some(lm);
        self
    }

    pub fn signature_name(&self) -> &str {
        &self.signature_name
    }

    pub fn input_keys(&self) -> &[String] {
        &self.input_keys
    }

    pub fn output_keys(&self) -> &[String] {
        &self.output_keys
    }

    pub fn demos(&self) -> &[RawExample] {
        &self.demos
    }

    pub fn set_demos(&mut self, demos: Vec<RawExample>) {
        self.demos = demos;
    }

    pub fn build_chat(&self, input: &RawExample) -> Result<Chat, PredictError> {
        let chat_adapter = crate::core::settings::chat_adapter();
        let system = chat_adapter
            .build_system(
                self.schema.as_ref(),
                self.instruction_override.as_deref(),
            )
            .map_err(|err| PredictError::Lm {
                source: LmError::Provider {
                    provider: "internal".to_string(),
                    message: err.to_string(),
                    source: None,
                },
            })?;

        let user = chat_adapter.format_input_raw(self.schema.as_ref(), input);
        let mut chat = Chat::new(vec![]);
        chat.push("system", &system);
        for demo in &self.demos {
            let demo_input = demo_as_input(demo, &self.input_keys);
            let demo_output = demo_as_output(demo, &self.output_keys);
            let demo_user = chat_adapter.format_input_raw(self.schema.as_ref(), &demo_input);
            let demo_assistant =
                chat_adapter.format_output_raw(self.schema.as_ref(), &demo_output);
            chat.push("user", &demo_user);
            chat.push("assistant", &demo_assistant);
        }
        chat.push("user", &user);
        trace!(message_count = chat.len(), "dyn chat constructed");
        Ok(chat)
    }

    pub async fn call_and_parse(
        &self,
        chat: Chat,
    ) -> Result<(Predicted<RawExample>, Chat), PredictError> {
        let lm = match &self.lm {
            Some(lm) => Arc::clone(lm),
            None => {
                let guard = GLOBAL_SETTINGS.read().unwrap();
                let settings = guard.as_ref().ok_or_else(|| PredictError::Lm {
                    source: LmError::Provider {
                        provider: "settings".to_string(),
                        message: "LM not configured; call dsir::configure first".to_string(),
                        source: None,
                    },
                })?;
                Arc::clone(&settings.lm)
            }
        };

        let response = lm
            .call(chat, self.tools.clone())
            .await
            .map_err(|err| PredictError::Lm {
                source: LmError::Provider {
                    provider: lm.model.clone(),
                    message: err.to_string(),
                    source: None,
                },
            })?;

        let crate::core::lm::LMResponse {
            output,
            usage,
            chat,
            tool_calls,
            tool_executions,
        } = response;

        let node_id = if crate::trace::is_tracing() {
            crate::trace::record_node(
                crate::trace::NodeType::Predict {
                    signature_name: self.signature_name.clone(),
                },
                vec![],
                None,
            )
        } else {
            None
        };

        let chat_adapter = crate::core::settings::chat_adapter();
        let raw_response = output.content().to_string();
        let lm_usage = usage.clone();

        let (parsed, field_metas) =
            match chat_adapter.parse_output_raw(self.schema.as_ref(), &output) {
                Ok(parsed) => parsed,
                Err(err) => {
                    return Err(PredictError::Parse {
                        source: err,
                        raw_response,
                        lm_usage,
                    });
                }
            };

        let mut metadata = CallMetadata::new(
            raw_response,
            lm_usage,
            tool_calls,
            tool_executions,
            node_id,
            field_metas,
        );
        let _ = &mut metadata;

        Ok((Predicted::new(parsed, metadata), chat))
    }
}

impl DynModule for DynPredict {
    async fn forward(&self, input: RawExample) -> Result<Predicted<RawExample>, PredictError> {
        let chat = self.build_chat(&input)?;
        let (predicted, _) = self.call_and_parse(chat).await?;
        Ok(predicted)
    }
}

impl DynPredictor for DynPredict {
    fn schema(&self) -> &SignatureSchema {
        self.schema.as_ref()
    }

    fn instruction(&self) -> String {
        self.instruction_override
            .clone()
            .unwrap_or_else(|| self.default_instruction.clone())
    }

    fn set_instruction(&mut self, instruction: String) {
        self.instruction_override = Some(instruction);
    }

    fn demos_as_examples(&self) -> Vec<RawExample> {
        self.demos.clone()
    }

    fn set_demos_from_examples(&mut self, demos: Vec<RawExample>) -> Result<()> {
        self.demos = demos;
        Ok(())
    }

    fn dump_state(&self) -> PredictState {
        PredictState {
            demos: self.demos.clone(),
            instruction_override: self.instruction_override.clone(),
        }
    }

    fn load_state(&mut self, state: PredictState) -> Result<()> {
        self.demos = state.demos;
        self.instruction_override = state.instruction_override;
        Ok(())
    }
}

fn demo_as_input(demo: &RawExample, input_keys: &[String]) -> RawExample {
    let mut data = std::collections::HashMap::new();
    for key in input_keys {
        if let Some(v) = demo.data.get(key) {
            data.insert(key.clone(), v.clone());
        }
    }
    // Also keep keys marked as inputs on the example itself.
    for key in &demo.input_keys {
        if let Some(v) = demo.data.get(key) {
            data.entry(key.clone()).or_insert_with(|| v.clone());
        }
    }
    RawExample::new(data, input_keys.to_vec(), Vec::new())
}

fn demo_as_output(demo: &RawExample, output_keys: &[String]) -> RawExample {
    let mut data = std::collections::HashMap::new();
    for key in output_keys {
        if let Some(v) = demo.data.get(key) {
            data.insert(key.clone(), v.clone());
        }
    }
    for key in &demo.output_keys {
        if let Some(v) = demo.data.get(key) {
            data.entry(key.clone()).or_insert_with(|| v.clone());
        }
    }
    RawExample::new(data, Vec::new(), output_keys.to_vec())
}

/// Dyn Chain-of-Thought: adds a `reasoning` output field at runtime.
#[derive(facet::Facet)]
#[facet(crate = facet)]
pub struct DynChainOfThought {
    pub predictor: DynPredict,
}

impl DynChainOfThought {
    pub fn new(signature: &DynSignature) -> Result<Self, anyhow::Error> {
        let augmented =
            signature.with_extra_string_output("reasoning", "Think step by step before answering.")?;
        Ok(Self {
            predictor: DynPredict::new(&augmented),
        })
    }
}

impl DynModule for DynChainOfThought {
    async fn forward(&self, input: RawExample) -> Result<Predicted<RawExample>, PredictError> {
        self.predictor.forward(input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DynPredictor;
    use serde_json::json;

    #[test]
    fn dyn_predict_state_roundtrip() {
        let sig = DynSignature::from_dsl("question -> answer").unwrap();
        let mut predictor = DynPredict::new(&sig);
        DynPredictor::set_instruction(&mut predictor, "be brief".into());
        let demo = RawExample::new(
            [("question".into(), json!("hi")), ("answer".into(), json!("hello"))]
                .into_iter()
                .collect(),
            vec!["question".into()],
            vec!["answer".into()],
        );
        DynPredictor::set_demos_from_examples(&mut predictor, vec![demo]).unwrap();
        let state = DynPredictor::dump_state(&predictor);
        let mut other = DynPredict::new(&sig);
        DynPredictor::load_state(&mut other, state).unwrap();
        assert_eq!(DynPredictor::instruction(&other), "be brief");
        assert_eq!(DynPredictor::demos_as_examples(&other).len(), 1);
    }
}
