use anyhow::Result;
use dsir::{
    BestOfN, BootstrapFewShot, CallMetadata, Example, LabeledFewShot, MetricOutcome, Module,
    Optimizer, Predict, PredictError, Predicted, Signature, TypedMetric, dump_program_state,
    load_program_state, propose_instructions_with_hint,
};

#[derive(Signature, Clone, Debug)]
struct Toy {
    #[input]
    prompt: String,
    #[output]
    answer: String,
}

#[derive(facet::Facet)]
#[facet(crate = facet)]
struct ToyModule {
    predictor: Predict<Toy>,
}

impl Module for ToyModule {
    type Input = ToyInput;
    type Output = ToyOutput;

    async fn forward(&self, input: ToyInput) -> Result<Predicted<ToyOutput>, PredictError> {
        Ok(Predicted::new(
            ToyOutput {
                answer: input.prompt,
            },
            CallMetadata::default(),
        ))
    }
}

struct Exact;

impl TypedMetric<Toy, ToyModule> for Exact {
    async fn evaluate(
        &self,
        example: &Example<Toy>,
        prediction: &Predicted<ToyOutput>,
    ) -> Result<MetricOutcome> {
        Ok(MetricOutcome::score(
            if prediction.answer == example.output.answer {
                1.0
            } else {
                0.0
            },
        ))
    }
}

impl TypedMetric<Toy, BestOfN<ToyModule>> for Exact {
    async fn evaluate(
        &self,
        example: &Example<Toy>,
        prediction: &Predicted<ToyOutput>,
    ) -> Result<MetricOutcome> {
        Ok(MetricOutcome::score(
            if prediction.answer == example.output.answer {
                1.0
            } else {
                0.0
            },
        ))
    }
}

fn trainset() -> Vec<Example<Toy>> {
    vec![
        Example::new(
            ToyInput {
                prompt: "a".into(),
            },
            ToyOutput {
                answer: "a".into(),
            },
        ),
        Example::new(
            ToyInput {
                prompt: "b".into(),
            },
            ToyOutput {
                answer: "b".into(),
            },
        ),
    ]
}

#[tokio::test]
async fn labeled_fewshot_assigns_demos() -> Result<()> {
    let mut module = ToyModule {
        predictor: Predict::new(),
    };
    let n = LabeledFewShot::builder()
        .k(2)
        .build()
        .compile(&mut module, trainset(), &Exact)
        .await?;
    assert_eq!(n, 1);
    Ok(())
}

#[tokio::test]
async fn bootstrap_fewshot_compiles() -> Result<()> {
    let mut module = ToyModule {
        predictor: Predict::new(),
    };
    let report = BootstrapFewShot::builder()
        .max_bootstrapped_demos(2)
        .max_labeled_demos(1)
        .build()
        .compile(&mut module, trainset(), &Exact)
        .await?;
    assert!(report.bootstrapped >= 1);
    assert_eq!(report.labeled, 1);
    Ok(())
}

#[tokio::test]
async fn best_of_n_picks_candidate() -> Result<()> {
    let inner = ToyModule {
        predictor: Predict::new(),
    };
    let best = BestOfN::new(inner, 3, |_| 1.0);
    let out = best
        .call(ToyInput {
            prompt: "x".into(),
        })
        .await?;
    assert_eq!(out.answer, "x");
    Ok(())
}

#[tokio::test]
async fn program_state_roundtrip() -> Result<()> {
    let mut module = ToyModule {
        predictor: Predict::new(),
    };
    LabeledFewShot::builder()
        .k(1)
        .build()
        .compile(&mut module, trainset(), &Exact)
        .await?;
    let state = dump_program_state(&mut module)?;
    assert!(!state.predictors.is_empty());
    load_program_state(&mut module, state)?;
    Ok(())
}

#[tokio::test]
async fn propose_instructions_falls_back_offline() -> Result<()> {
    let candidates = propose_instructions_with_hint("Do the task.", "answer", 3, None).await?;
    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0], "Do the task.");
    Ok(())
}
