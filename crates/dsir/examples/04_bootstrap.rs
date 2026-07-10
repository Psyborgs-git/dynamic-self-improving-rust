//! BootstrapFewShot + save/load demo with a mock-friendly offline path.
//!
//! When no API key is set, runs the optimizer against a deterministic stub module.

use anyhow::Result;
use dsir::{
    BootstrapFewShot, CallMetadata, Example, MetricOutcome, Module, Optimizer, Predict,
    PredictError, Predicted, Signature, TypedMetric, load_program, save_program,
};

#[derive(Signature, Clone, Debug)]
struct Echo {
    #[input]
    prompt: String,
    #[output]
    answer: String,
}

#[derive(facet::Facet)]
#[facet(crate = facet)]
struct EchoModule {
    predictor: Predict<Echo>,
}

impl Module for EchoModule {
    type Input = EchoInput;
    type Output = EchoOutput;

    async fn forward(&self, input: EchoInput) -> Result<Predicted<EchoOutput>, PredictError> {
        // Deterministic "teacher": echo the prompt as the answer.
        Ok(Predicted::new(
            EchoOutput {
                answer: input.prompt,
            },
            CallMetadata::default(),
        ))
    }
}

struct ExactMatch;

impl TypedMetric<Echo, EchoModule> for ExactMatch {
    async fn evaluate(
        &self,
        example: &Example<Echo>,
        prediction: &Predicted<EchoOutput>,
    ) -> Result<MetricOutcome> {
        let score = if prediction.answer == example.output.answer {
            1.0
        } else {
            0.0
        };
        Ok(MetricOutcome::score(score))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut module = EchoModule {
        predictor: Predict::<Echo>::new(),
    };

    let trainset = vec![
        Example::new(
            EchoInput {
                prompt: "hello".into(),
            },
            EchoOutput {
                answer: "hello".into(),
            },
        ),
        Example::new(
            EchoInput {
                prompt: "world".into(),
            },
            EchoOutput {
                answer: "world".into(),
            },
        ),
    ];

    let optimizer = BootstrapFewShot::builder()
        .max_bootstrapped_demos(2)
        .max_labeled_demos(1)
        .metric_threshold(1.0)
        .build();
    let report = optimizer
        .compile(&mut module, trainset, &ExactMatch)
        .await?;
    println!(
        "labeled={} bootstrapped={} avg={:.3}",
        report.labeled, report.bootstrapped, report.average_train_score
    );

    let path = std::env::temp_dir().join("dsir_echo_program.json");
    save_program(&mut module, &path)?;
    load_program(&mut module, &path)?;
    println!("saved+loaded {}", path.display());
    Ok(())
}
