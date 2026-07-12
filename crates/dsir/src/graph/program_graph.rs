//! Runtime program graph over dynamic predictors.

use std::collections::BTreeMap;
use std::ops::ControlFlow;

use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{DynPredictor, PredictState, visit_named_predictors_mut};
use crate::data::example::Example as RawExample;
use crate::graph::{StrategyFactory, StrategyKind};
use crate::predictors::{DynModule, DynPredict};
use crate::{CallMetadata, DynSignature, Facet, PredictError, Predicted};

/// Edge that copies fields from the workspace (or a prior node prefix) into the next node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    /// Source field in the shared workspace.
    pub from: String,
    /// Destination field name for the next step's input.
    pub to: String,
}

/// One named node in a [`ProgramGraph`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeSpec {
    pub name: String,
    pub strategy: StrategyKind,
    pub signature: DynSignatureDocSerde,
    #[serde(default)]
    pub edges_in: Vec<GraphEdge>,
}

/// Serializable signature document embedded in graph topology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynSignatureDocSerde {
    pub name: String,
    #[serde(default)]
    pub instruction: String,
    #[serde(default)]
    pub inputs: indexmap::IndexMap<String, crate::DynFieldType>,
    #[serde(default)]
    pub outputs: indexmap::IndexMap<String, crate::DynFieldType>,
}

impl From<&DynSignature> for DynSignatureDocSerde {
    fn from(sig: &DynSignature) -> Self {
        Self {
            name: sig.name.clone(),
            instruction: sig.instruction().to_string(),
            inputs: sig.input_types().clone(),
            outputs: sig.output_types().clone(),
        }
    }
}

impl DynSignatureDocSerde {
    pub fn to_signature(&self) -> Result<DynSignature> {
        DynSignature::from_doc(crate::DynSignatureDoc {
            name: self.name.clone(),
            instruction: self.instruction.clone(),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
        })
    }
}

/// Serializable topology + predictor state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphState {
    pub nodes: Vec<GraphNodeSpec>,
    pub predictors: BTreeMap<String, PredictState>,
}

/// A linear/DAG-ordered program of [`DynPredict`] leaves with field wiring.
pub struct ProgramGraph {
    nodes: Vec<GraphNodeRuntime>,
}

struct GraphNodeRuntime {
    name: String,
    strategy: StrategyKind,
    signature: DynSignature,
    edges_in: Vec<GraphEdge>,
    predictor: DynPredict,
}

// Manual Facet walk: expose predictors as named fields via a companion container.
// ProgramGraph stores predictors in a Vec, so we implement leaf discovery through
// an explicit registry method rather than relying solely on Facet struct fields.

impl ProgramGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn from_single(name: impl Into<String>, signature: &DynSignature, kind: StrategyKind) -> Result<Self> {
        let mut graph = Self::new();
        graph.add_node(name, signature, kind, Vec::new())?;
        Ok(graph)
    }

    pub fn add_node(
        &mut self,
        name: impl Into<String>,
        signature: &DynSignature,
        kind: StrategyKind,
        edges_in: Vec<GraphEdge>,
    ) -> Result<()> {
        let name = name.into();
        if self.nodes.iter().any(|n| n.name == name) {
            bail!("duplicate graph node `{name}`");
        }
        let predictor = StrategyFactory::create_predict_for_kind(signature, kind)?;
        self.nodes.push(GraphNodeRuntime {
            name,
            strategy: kind,
            signature: signature.clone(),
            edges_in,
            predictor,
        });
        Ok(())
    }

    pub fn node_names(&self) -> Vec<String> {
        self.nodes.iter().map(|n| n.name.clone()).collect()
    }

    pub fn predictors_mut(&mut self) -> Vec<(&str, &mut DynPredict)> {
        self.nodes
            .iter_mut()
            .map(|n| (n.name.as_str(), &mut n.predictor))
            .collect()
    }

    pub fn dump_state(&mut self) -> GraphState {
        let mut predictors = BTreeMap::new();
        for (name, predictor) in self.predictors_mut() {
            predictors.insert(name.to_string(), DynPredictor::dump_state(predictor));
        }
        let nodes = self
            .nodes
            .iter()
            .map(|n| GraphNodeSpec {
                name: n.name.clone(),
                strategy: n.strategy,
                signature: DynSignatureDocSerde::from(&n.signature),
                edges_in: n.edges_in.clone(),
            })
            .collect();
        GraphState { nodes, predictors }
    }

    pub fn load_state(&mut self, state: GraphState) -> Result<()> {
        // Rebuild topology if provided.
        if !state.nodes.is_empty() {
            let mut rebuilt = ProgramGraph::new();
            for spec in &state.nodes {
                let sig = spec.signature.to_signature()?;
                rebuilt.add_node(&spec.name, &sig, spec.strategy, spec.edges_in.clone())?;
            }
            *self = rebuilt;
        }
        for (name, pred_state) in state.predictors {
            let Some(node) = self.nodes.iter_mut().find(|n| n.name == name) else {
                continue;
            };
            DynPredictor::load_state(&mut node.predictor, pred_state)?;
        }
        Ok(())
    }

    pub fn save_json(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let state = self.dump_state();
        let json = serde_json::to_string_pretty(&state)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_json(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let state: GraphState = serde_json::from_slice(&bytes)?;
        let mut graph = Self::new();
        graph.load_state(state)?;
        Ok(graph)
    }

    /// Apply predictor state via Facet-compatible wrapper when graph is embedded in a struct.
    pub fn visit_predictors_mut<F>(&mut self, mut visitor: F) -> Result<()>
    where
        F: FnMut(&str, &mut DynPredict) -> ControlFlow<()>,
    {
        for node in &mut self.nodes {
            if visitor(&node.name, &mut node.predictor).is_break() {
                break;
            }
        }
        Ok(())
    }
}

impl Default for ProgramGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DynModule for ProgramGraph {
    async fn forward(&self, input: RawExample) -> Result<Predicted<RawExample>, PredictError> {
        let mut workspace = input.data.clone();
        let mut last_output = RawExample::default();
        let mut metadata = CallMetadata::default();

        for node in &self.nodes {
            let mut step_input_data = std::collections::HashMap::new();
            if node.edges_in.is_empty() {
                for key in node.predictor.input_keys() {
                    if let Some(v) = workspace.get(key) {
                        step_input_data.insert(key.clone(), v.clone());
                    }
                }
            } else {
                for edge in &node.edges_in {
                    let value = workspace.get(&edge.from).cloned().unwrap_or(Value::Null);
                    step_input_data.insert(edge.to.clone(), value);
                }
            }
            let step_input = RawExample::new(
                step_input_data,
                node.predictor.input_keys().to_vec(),
                Vec::new(),
            );
            let predicted = node.predictor.call(step_input).await?;
            metadata = predicted.metadata().clone();
            for (k, v) in &predicted.data {
                workspace.insert(k.clone(), v.clone());
                // Also namespace by node for multi-step wiring: node.field
                workspace.insert(format!("{}.{}", node.name, k), v.clone());
            }
            last_output = predicted.into_inner();
        }

        // Final workspace projection: last node's outputs preferred, else full workspace.
        if last_output.data.is_empty() {
            last_output = RawExample::new(workspace, Vec::new(), Vec::new());
        }
        Ok(Predicted::new(last_output, metadata))
    }
}

/// Facet-friendly single-predictor module used when optimizers need `M: Facet`.
#[derive(facet::Facet)]
#[facet(crate = facet)]
pub struct GraphModule {
    pub predictor: DynPredict,
}

impl GraphModule {
    pub fn from_signature(signature: &DynSignature, kind: StrategyKind) -> Result<Self> {
        Ok(Self {
            predictor: StrategyFactory::create_predict_for_kind(signature, kind)?,
        })
    }
}

impl DynModule for GraphModule {
    async fn forward(&self, input: RawExample) -> Result<Predicted<RawExample>, PredictError> {
        self.predictor.forward(input).await
    }
}

/// Dump predictor state from any Facet module (typed or dyn leaves).
pub fn dump_facet_program_state<M>(module: &mut M) -> Result<BTreeMap<String, PredictState>>
where
    M: for<'a> Facet<'a>,
{
    let mut predictors = BTreeMap::new();
    visit_named_predictors_mut(module, |name, predictor| {
        predictors.insert(name.to_string(), predictor.dump_state());
        ControlFlow::Continue(())
    })
    .map_err(|e| anyhow!("{e}"))?;
    Ok(predictors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn single_node_graph_echo_via_stub() {
        let sig = DynSignature::from_dsl("prompt -> answer").unwrap();
        let mut graph = ProgramGraph::from_single("main", &sig, StrategyKind::Predict).unwrap();
        // Without LM, call fails — just check state roundtrip here.
        let state = graph.dump_state();
        assert_eq!(state.nodes.len(), 1);
        let mut g2 = ProgramGraph::new();
        g2.load_state(state).unwrap();
        assert_eq!(g2.node_names(), vec!["main".to_string()]);
        let _ = json!(null);
    }
}
