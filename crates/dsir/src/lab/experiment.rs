//! Experiment authoring, runs, comparison, and promotion.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::DynPredictor;
use crate::data::example::Example as RawExample;
use crate::evaluate::{FieldExactMatch, average_score, evaluate_dyn_trainset};
use crate::graph::{GraphModule, GraphState, StrategyFactory, StrategyKind, StructuralOptimizer};
use crate::optimizer::{BootstrapFewShot, COPRO, GEPA, MIPROv2};
use crate::predictors::{DynModule, DynPredict};
use crate::trace;
use crate::{CallMetadata, DynSignature, Predicted};

use super::{LocalRegistry, RunRecord};

/// Optimizer choice for a lab run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabOptimizer {
    LabeledFewShot {
        k: usize,
    },
    BootstrapFewShot {
        max_bootstrapped_demos: usize,
        max_labeled_demos: usize,
        metric_threshold: f32,
    },
    Copro {
        breadth: usize,
        depth: usize,
    },
    Mipro,
    Gepa {
        num_iterations: usize,
    },
    Structural,
}

/// Offline-friendly Facet module: optimizers mutate `predictor` state, but
/// `forward` echoes prompt/question → answer without calling an LM.
#[derive(facet::Facet)]
#[facet(crate = facet)]
pub struct EchoModule {
    pub predictor: DynPredict,
}

impl EchoModule {
    pub fn from_signature(signature: &DynSignature, strategy: StrategyKind) -> Result<Self> {
        Ok(Self {
            predictor: StrategyFactory::create_predict_for_kind(signature, strategy)?,
        })
    }
}

impl DynModule for EchoModule {
    async fn forward(
        &self,
        input: RawExample,
    ) -> Result<Predicted<RawExample>, crate::PredictError> {
        let mut data = input.data.clone();
        if let Some(prompt) = data.get("prompt").cloned() {
            data.insert("answer".into(), prompt);
        } else if let Some(q) = data.get("question").cloned() {
            data.insert("answer".into(), q);
        }
        Ok(Predicted::new(
            RawExample::new(data, input.input_keys, vec!["answer".into()]),
            CallMetadata::default(),
        ))
    }
}

enum ProgramSlot {
    Live(GraphModule),
    Echo(EchoModule),
}

impl ProgramSlot {
    fn predictor_mut(&mut self) -> &mut DynPredict {
        match self {
            Self::Live(m) => &mut m.predictor,
            Self::Echo(m) => &mut m.predictor,
        }
    }
}

/// In-memory + filesystem-backed experiment harness.
pub struct Lab {
    pub registry: LocalRegistry,
    programs: BTreeMap<String, ProgramSlot>,
    signatures: BTreeMap<String, DynSignature>,
    datasets: BTreeMap<String, Vec<RawExample>>,
    promoted: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareRow {
    pub run_id: String,
    pub optimizer: String,
    pub avg_train: f32,
    pub avg_val: f32,
}

impl Lab {
    pub fn open(workdir: impl AsRef<Path>) -> Result<Self> {
        let registry = LocalRegistry::open(workdir)?;
        Ok(Self {
            registry,
            programs: BTreeMap::new(),
            signatures: BTreeMap::new(),
            datasets: BTreeMap::new(),
            promoted: None,
        })
    }

    pub fn author(
        &mut self,
        program_id: impl Into<String>,
        signature: DynSignature,
        strategy: StrategyKind,
    ) -> Result<String> {
        let program_id = program_id.into();
        let module = GraphModule::from_signature(&signature, strategy)?;
        self.signatures.insert(program_id.clone(), signature);
        self.programs
            .insert(program_id.clone(), ProgramSlot::Live(module));
        self.registry.write_json(
            &format!("programs/{program_id}.meta.json"),
            &serde_json::json!({
                "id": program_id,
                "strategy": strategy,
            }),
        )?;
        Ok(program_id)
    }

    /// Author an offline echo program (no LM required for optimize/execute).
    pub fn author_echo(
        &mut self,
        program_id: impl Into<String>,
        signature: DynSignature,
        strategy: StrategyKind,
    ) -> Result<String> {
        let program_id = program_id.into();
        let module = EchoModule::from_signature(&signature, strategy)?;
        self.signatures.insert(program_id.clone(), signature);
        self.programs
            .insert(program_id.clone(), ProgramSlot::Echo(module));
        self.registry.write_json(
            &format!("programs/{program_id}.meta.json"),
            &serde_json::json!({
                "id": program_id,
                "strategy": strategy,
                "echo": true,
            }),
        )?;
        Ok(program_id)
    }

    pub fn author_from_json(
        &mut self,
        program_id: impl Into<String>,
        json: &str,
        strategy: StrategyKind,
    ) -> Result<String> {
        let signature = DynSignature::from_json_str(json)?;
        self.author(program_id, signature, strategy)
    }

    pub fn author_from_dsl(
        &mut self,
        program_id: impl Into<String>,
        dsl: &str,
        strategy: StrategyKind,
    ) -> Result<String> {
        let signature = DynSignature::from_dsl(dsl)?;
        self.author(program_id, signature, strategy)
    }

    pub fn load_dataset_json(
        &mut self,
        dataset_id: impl Into<String>,
        rows: Vec<Value>,
        input_keys: Vec<String>,
        output_keys: Vec<String>,
    ) -> Result<String> {
        let dataset_id = dataset_id.into();
        let mut examples = Vec::new();
        for row in rows {
            let obj = row
                .as_object()
                .ok_or_else(|| anyhow!("dataset row must be a JSON object"))?;
            let data = obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            examples.push(RawExample::new(
                data,
                input_keys.clone(),
                output_keys.clone(),
            ));
        }
        self.registry
            .write_json(&format!("datasets/{dataset_id}.json"), &examples)?;
        self.datasets.insert(dataset_id.clone(), examples);
        Ok(dataset_id)
    }

    pub fn split_dataset(
        &self,
        dataset_id: &str,
        train_n: usize,
    ) -> Result<(Vec<RawExample>, Vec<RawExample>)> {
        let data = self
            .datasets
            .get(dataset_id)
            .ok_or_else(|| anyhow!("unknown dataset `{dataset_id}`"))?;
        let train = data.iter().take(train_n).cloned().collect::<Vec<_>>();
        let val = data.iter().skip(train_n).cloned().collect::<Vec<_>>();
        Ok((train, val))
    }

    pub async fn execute(
        &self,
        program_id: &str,
        input: RawExample,
    ) -> Result<Predicted<RawExample>> {
        match self
            .programs
            .get(program_id)
            .ok_or_else(|| anyhow!("unknown program `{program_id}`"))?
        {
            ProgramSlot::Live(m) => m.call(input).await.map_err(|e| anyhow!("{e}")),
            ProgramSlot::Echo(m) => m.call(input).await.map_err(|e| anyhow!("{e}")),
        }
    }

    pub async fn execute_traced(
        &self,
        program_id: &str,
        input: RawExample,
    ) -> Result<(Predicted<RawExample>, trace::Graph)> {
        match self
            .programs
            .get(program_id)
            .ok_or_else(|| anyhow!("unknown program `{program_id}`"))?
        {
            ProgramSlot::Live(m) => {
                let (result, graph) = trace::trace(|| async { m.call(input).await }).await;
                Ok((result.map_err(|e| anyhow!("{e}"))?, graph))
            }
            ProgramSlot::Echo(m) => {
                let (result, graph) = trace::trace(|| async { m.call(input).await }).await;
                Ok((result.map_err(|e| anyhow!("{e}"))?, graph))
            }
        }
    }

    pub async fn optimize(
        &mut self,
        program_id: &str,
        dataset_id: &str,
        train_n: usize,
        optimizer: LabOptimizer,
        metric_field: &str,
    ) -> Result<RunRecord> {
        let (train, val) = self.split_dataset(dataset_id, train_n)?;
        let metric = FieldExactMatch::new(metric_field);
        let feedback_metric = FieldExactMatch::new(metric_field).with_feedback();
        let optimizer_name = match &optimizer {
            LabOptimizer::LabeledFewShot { .. } => "labeled_fewshot",
            LabOptimizer::BootstrapFewShot { .. } => "bootstrap_fewshot",
            LabOptimizer::Copro { .. } => "copro",
            LabOptimizer::Mipro => "mipro",
            LabOptimizer::Gepa { .. } => "gepa",
            LabOptimizer::Structural => "structural",
        }
        .to_string();

        if matches!(optimizer, LabOptimizer::Structural) {
            let sig = self
                .signatures
                .get(program_id)
                .ok_or_else(|| anyhow!("missing signature for `{program_id}`"))?
                .clone();
            match self
                .programs
                .get_mut(program_id)
                .ok_or_else(|| anyhow!("unknown program `{program_id}`"))?
            {
                ProgramSlot::Live(module) => {
                    StructuralOptimizer::new()
                        .compile(&sig, module, train.clone(), &metric)
                        .await?;
                }
                ProgramSlot::Echo(_) => {
                    bail!("structural optimizer requires a live GraphModule program");
                }
            }
        }

        let (avg_train, avg_val) = match self
            .programs
            .get_mut(program_id)
            .ok_or_else(|| anyhow!("unknown program `{program_id}`"))?
        {
            ProgramSlot::Live(module) => {
                run_optimizer(&optimizer, module, train.clone(), &metric, &feedback_metric)
                    .await?;
                let avg_train =
                    average_score(&evaluate_dyn_trainset(module, &train, &metric).await?);
                let avg_val = if val.is_empty() {
                    0.0
                } else {
                    average_score(&evaluate_dyn_trainset(module, &val, &metric).await?)
                };
                (avg_train, avg_val)
            }
            ProgramSlot::Echo(module) => {
                run_optimizer(&optimizer, module, train.clone(), &metric, &feedback_metric)
                    .await?;
                let avg_train =
                    average_score(&evaluate_dyn_trainset(module, &train, &metric).await?);
                let avg_val = if val.is_empty() {
                    0.0
                } else {
                    average_score(&evaluate_dyn_trainset(module, &val, &metric).await?)
                };
                (avg_train, avg_val)
            }
        };

        let module_predictor_state = {
            let slot = self
                .programs
                .get_mut(program_id)
                .ok_or_else(|| anyhow!("unknown program `{program_id}`"))?;
            DynPredictor::dump_state(slot.predictor_mut())
        };

        let mut state = GraphState::default();
        state
            .predictors
            .insert("predictor".into(), module_predictor_state);
        let run = self.registry.record_run(RunRecord {
            id: String::new(),
            program_id: program_id.to_string(),
            dataset_id: dataset_id.to_string(),
            optimizer: optimizer_name,
            avg_train,
            avg_val,
            artifact: String::new(),
        })?;
        self.registry
            .save_artifact(&run.id, &state)
            .context("save run artifact")?;
        Ok(run)
    }

    pub fn compare(&self, run_ids: &[String]) -> Result<Vec<CompareRow>> {
        let mut rows = Vec::new();
        for id in run_ids {
            let run = self.registry.get_run(id)?;
            rows.push(CompareRow {
                run_id: run.id,
                optimizer: run.optimizer,
                avg_train: run.avg_train,
                avg_val: run.avg_val,
            });
        }
        rows.sort_by(|a, b| {
            b.avg_val
                .partial_cmp(&a.avg_val)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(rows)
    }

    pub fn promote(&mut self, run_id: &str, min_val: f32) -> Result<PathBuf> {
        let run = self.registry.get_run(run_id)?;
        if run.avg_val < min_val {
            bail!(
                "run `{run_id}` val score {:.3} below promotion threshold {min_val}",
                run.avg_val
            );
        }
        let path = self.registry.promote(&run)?;
        let state: GraphState = self.registry.load_artifact(&run.id)?;
        if let Some(slot) = self.programs.get_mut(&run.program_id) {
            if let Some(pred_state) = state.predictors.get("predictor") {
                DynPredictor::load_state(slot.predictor_mut(), pred_state.clone())?;
            }
        }
        self.promoted = Some(run.program_id);
        Ok(path)
    }

    pub fn promoted_program_id(&self) -> Option<&str> {
        self.promoted.as_deref()
    }
}

async fn run_optimizer<M>(
    optimizer: &LabOptimizer,
    module: &mut M,
    train: Vec<RawExample>,
    metric: &FieldExactMatch,
    feedback_metric: &FieldExactMatch,
) -> Result<()>
where
    M: DynModule + for<'a> crate::Facet<'a>,
{
    match optimizer {
        LabOptimizer::LabeledFewShot { k } => {
            crate::LabeledFewShot { k: *k }
                .compile_dyn(module, train)
                .await?;
        }
        LabOptimizer::BootstrapFewShot {
            max_bootstrapped_demos,
            max_labeled_demos,
            metric_threshold,
        } => {
            BootstrapFewShot {
                max_bootstrapped_demos: *max_bootstrapped_demos,
                max_labeled_demos: *max_labeled_demos,
                metric_threshold: *metric_threshold,
            }
            .compile_dyn(module, train, metric)
            .await?;
        }
        LabOptimizer::Copro { breadth, depth } => {
            COPRO::builder()
                .breadth(*breadth)
                .depth(*depth)
                .build()
                .compile_dyn(module, train, metric)
                .await?;
        }
        LabOptimizer::Mipro => {
            MIPROv2::builder()
                .build()
                .compile_dyn(module, train, metric)
                .await?;
        }
        LabOptimizer::Gepa { num_iterations } => {
            GEPA::builder()
                .num_iterations(*num_iterations)
                .build()
                .compile_dyn(module, train, feedback_metric)
                .await?;
        }
        LabOptimizer::Structural => {}
    }
    Ok(())
}
