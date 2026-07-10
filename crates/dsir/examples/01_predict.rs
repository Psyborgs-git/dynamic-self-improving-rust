//! Smoke example: Signature → Predict (requires OPENAI_API_KEY).
//!
//! ```bash
//! cargo run -p dsir --example 01_predict
//! ```

use dsir::{ChatAdapter, LM, Predict, Signature, configure};

/// Answer questions accurately and concisely.
#[derive(Signature, Clone, Debug)]
struct QA {
    #[input]
    question: String,
    #[output]
    answer: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let lm = LM::builder()
        .model("openai:gpt-4o-mini".to_string())
        .build()
        .await?;
    configure(lm, ChatAdapter);

    let predict = Predict::<QA>::new();
    let out = predict
        .call(QAInput {
            question: "What is the capital of France?".into(),
        })
        .await?;
    println!("answer={}", out.answer);
    Ok(())
}
