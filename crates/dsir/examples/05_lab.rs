//! Offline self-improvement lab: author → bootstrap/COPRO → compare → promote.

use anyhow::Result;
use dsir::{DynSignature, Lab, LabOptimizer, StrategyKind, UntypedExample};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let workdir = std::env::temp_dir().join("dsir_lab_example");
    let _ = std::fs::remove_dir_all(&workdir);
    let mut lab = Lab::open(&workdir)?;

    let signature = DynSignature::from_dsl("Answer by echoing | prompt -> answer")?;
    lab.author_echo("echo_qa", signature, StrategyKind::Predict)?;

    lab.load_dataset_json(
        "toy",
        vec![
            json!({"prompt": "hello", "answer": "hello"}),
            json!({"prompt": "world", "answer": "world"}),
            json!({"prompt": "dsir", "answer": "dsir"}),
            json!({"prompt": "lab", "answer": "lab"}),
        ],
        vec!["prompt".into()],
        vec!["answer".into()],
    )?;

    let bootstrap = lab
        .optimize(
            "echo_qa",
            "toy",
            3,
            LabOptimizer::BootstrapFewShot {
                max_bootstrapped_demos: 2,
                max_labeled_demos: 1,
                metric_threshold: 1.0,
            },
            "answer",
        )
        .await?;
    println!(
        "bootstrap run={} train={:.3} val={:.3}",
        bootstrap.id, bootstrap.avg_train, bootstrap.avg_val
    );

    let copro = lab
        .optimize(
            "echo_qa",
            "toy",
            3,
            LabOptimizer::Copro {
                breadth: 3,
                depth: 1,
            },
            "answer",
        )
        .await?;
    println!(
        "copro run={} train={:.3} val={:.3}",
        copro.id, copro.avg_train, copro.avg_val
    );

    let comparison = lab.compare(&[bootstrap.id.clone(), copro.id.clone()])?;
    for row in &comparison {
        println!(
            "compare {} {} train={:.3} val={:.3}",
            row.run_id, row.optimizer, row.avg_train, row.avg_val
        );
    }

    let best = comparison
        .first()
        .map(|r| r.run_id.clone())
        .unwrap_or(bootstrap.id);
    let promoted = lab.promote(&best, 0.0)?;
    println!("promoted -> {}", promoted.display());

    let out = lab
        .execute(
            "echo_qa",
            UntypedExample::new(
                [("prompt".into(), json!("hello"))]
                    .into_iter()
                    .collect(),
                vec!["prompt".into()],
                vec![],
            ),
        )
        .await?;
    println!("execute answer={:?}", out.data.get("answer"));
    println!("workdir={}", workdir.display());
    Ok(())
}
