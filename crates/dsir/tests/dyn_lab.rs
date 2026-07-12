use anyhow::Result;
use dsir::{
    BootstrapFewShot, COPRO, DynSignature, EchoModule, FieldExactMatch, Lab, LabOptimizer,
    StrategyKind, UntypedExample,
};
use serde_json::json;

#[tokio::test]
async fn compile_dyn_bootstrap_and_copro() -> Result<()> {
    let sig = DynSignature::from_dsl("prompt -> answer")?;
    let mut module = EchoModule::from_signature(&sig, StrategyKind::Predict)?;
    let train = vec![
        UntypedExample::new(
            [("prompt".into(), json!("x")), ("answer".into(), json!("x"))]
                .into_iter()
                .collect(),
            vec!["prompt".into()],
            vec!["answer".into()],
        ),
        UntypedExample::new(
            [("prompt".into(), json!("y")), ("answer".into(), json!("y"))]
                .into_iter()
                .collect(),
            vec!["prompt".into()],
            vec!["answer".into()],
        ),
    ];
    let metric = FieldExactMatch::new("answer");
    let report = BootstrapFewShot::builder()
        .max_bootstrapped_demos(2)
        .max_labeled_demos(1)
        .metric_threshold(1.0)
        .build()
        .compile_dyn(&mut module, train.clone(), &metric)
        .await?;
    assert!(report.bootstrapped >= 1);

    COPRO::builder()
        .breadth(3)
        .depth(1)
        .build()
        .compile_dyn(&mut module, train, &metric)
        .await?;
    Ok(())
}

#[tokio::test]
async fn lab_compare_and_promote() -> Result<()> {
    let workdir = tempfile::tempdir()?;
    let mut lab = Lab::open(workdir.path())?;
    let sig = DynSignature::from_dsl("prompt -> answer")?;
    lab.author_echo("p", sig, StrategyKind::Predict)?;
    lab.load_dataset_json(
        "d",
        vec![
            json!({"prompt":"a","answer":"a"}),
            json!({"prompt":"b","answer":"b"}),
            json!({"prompt":"c","answer":"c"}),
        ],
        vec!["prompt".into()],
        vec!["answer".into()],
    )?;
    let r1 = lab
        .optimize(
            "p",
            "d",
            2,
            LabOptimizer::BootstrapFewShot {
                max_bootstrapped_demos: 2,
                max_labeled_demos: 1,
                metric_threshold: 1.0,
            },
            "answer",
        )
        .await?;
    let r2 = lab
        .optimize(
            "p",
            "d",
            2,
            LabOptimizer::LabeledFewShot { k: 1 },
            "answer",
        )
        .await?;
    let rows = lab.compare(&[r1.id.clone(), r2.id.clone()])?;
    assert_eq!(rows.len(), 2);
    let path = lab.promote(&r1.id, 0.0)?;
    assert!(path.exists());
    Ok(())
}
