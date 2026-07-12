//! ProgramGraph + StrategyFactory smoke (offline).

use anyhow::Result;
use dsir::{
    DynModule, DynSignature, EchoModule, ProgramGraph, StrategyKind, UntypedExample,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let sig = DynSignature::from_json_str(
        r#"{
            "name": "Echo",
            "instruction": "Echo the prompt.",
            "inputs": { "prompt": "string" },
            "outputs": { "answer": "string" }
        }"#,
    )?;

    let mut graph = ProgramGraph::from_single("main", &sig, StrategyKind::Predict)?;
    let path = std::env::temp_dir().join("dsir_graph_example.json");
    graph.save_json(&path)?;
    let loaded = ProgramGraph::load_json(&path)?;
    println!("graph nodes={:?}", loaded.node_names());

    let module = EchoModule::from_signature(&sig, StrategyKind::Predict)?;
    let predicted = module
        .call(UntypedExample::new(
            [("prompt".into(), json!("ping"))].into_iter().collect(),
            vec!["prompt".into()],
            vec![],
        ))
        .await?;
    println!("echo => {:?}", predicted.data.get("answer"));
    println!("saved {}", path.display());
    Ok(())
}
