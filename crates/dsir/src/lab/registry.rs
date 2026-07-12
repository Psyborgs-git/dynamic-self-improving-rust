//! Local filesystem experiment/run registry (+ remote client helpers).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::graph::GraphState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub id: String,
    pub program_id: String,
    pub dataset_id: String,
    pub optimizer: String,
    pub avg_train: f32,
    pub avg_val: f32,
    pub artifact: String,
}

#[derive(Debug, Clone)]
pub struct LocalRegistry {
    root: PathBuf,
    next_id: u64,
}

impl LocalRegistry {
    pub fn open(workdir: impl AsRef<Path>) -> Result<Self> {
        let root = workdir.as_ref().to_path_buf();
        for sub in [
            "experiments",
            "runs",
            "artifacts",
            "promoted",
            "programs",
            "datasets",
        ] {
            std::fs::create_dir_all(root.join(sub))?;
        }
        let next_id = std::fs::read_dir(root.join("runs"))?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .max()
            .unwrap_or(0)
            + 1;
        Ok(Self { root, next_id })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn write_json(&self, rel: &str, value: &impl Serialize) -> Result<PathBuf> {
        let path = self.root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(value)?;
        std::fs::write(&path, json)?;
        Ok(path)
    }

    pub fn record_run(&mut self, mut run: RunRecord) -> Result<RunRecord> {
        let id = format!("{:04}", self.next_id);
        self.next_id += 1;
        run.id = id.clone();
        run.artifact = format!("artifacts/{id}.json");
        self.write_json(&format!("runs/{id}.json"), &run)?;
        Ok(run)
    }

    pub fn get_run(&self, id: &str) -> Result<RunRecord> {
        let path = self.root.join(format!("runs/{id}.json"));
        let bytes = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn list_runs(&self) -> Result<Vec<RunRecord>> {
        let mut runs: Vec<RunRecord> = Vec::new();
        for entry in std::fs::read_dir(self.root.join("runs"))? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let bytes = std::fs::read(entry.path())?;
            runs.push(serde_json::from_slice(&bytes)?);
        }
        runs.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(runs)
    }

    pub fn save_artifact(&self, run_id: &str, state: &GraphState) -> Result<PathBuf> {
        self.write_json(&format!("artifacts/{run_id}.json"), state)
    }

    pub fn load_artifact(&self, run_id: &str) -> Result<GraphState> {
        let path = self.root.join(format!("artifacts/{run_id}.json"));
        let bytes = std::fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn promote(&self, run: &RunRecord) -> Result<PathBuf> {
        let state = self.load_artifact(&run.id)?;
        let path = self.write_json("promoted/program.json", &state)?;
        self.write_json(
            "promoted/meta.json",
            &serde_json::json!({
                "run_id": run.id,
                "program_id": run.program_id,
                "avg_val": run.avg_val,
                "optimizer": run.optimizer,
            }),
        )?;
        Ok(path)
    }
}

/// HTTP client for a remote run registry.
#[derive(Debug, Clone)]
pub struct RegistryClient {
    pub base_url: String,
    pub token: Option<String>,
}

impl RegistryClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: std::env::var("DSIR_REGISTRY_TOKEN").ok(),
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(token) = &self.token {
            if let Ok(value) =
                reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
            {
                headers.insert(reqwest::header::AUTHORIZATION, value);
            }
        }
        headers
    }

    pub async fn list_runs(&self) -> Result<Vec<RunRecord>> {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/v1/runs", self.base_url))
            .headers(self.headers())
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn upload_run(&self, run: &RunRecord, artifact: &GraphState) -> Result<RunRecord> {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "run": run,
            "artifact": artifact,
        });
        let resp = client
            .post(format!("{}/v1/runs", self.base_url))
            .headers(self.headers())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn get_promoted(&self) -> Result<Value> {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/v1/promoted", self.base_url))
            .headers(self.headers())
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }
}

/// Sync helper used by tests when async runtime isn't needed for local ops.
pub fn ensure_workdir(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref().to_path_buf();
    LocalRegistry::open(&path)?;
    Ok(path)
}

pub fn read_json_file<T: for<'de> Deserialize<'de>>(path: impl AsRef<Path>) -> Result<T> {
    let bytes = std::fs::read(path.as_ref())
        .with_context(|| format!("read {}", path.as_ref().display()))?;
    Ok(serde_json::from_slice(&bytes)?)
}
