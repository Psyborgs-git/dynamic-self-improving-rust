//! Chain-of-thought example (requires OPENAI_API_KEY).

use dsir::{ChainOfThought, ChatAdapter, LM, Signature, configure};

/// Answer with careful reasoning.
#[derive(Signature, Clone, Debug)]
struct MathQA {
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

    let cot = ChainOfThought::<MathQA>::new();
    let out = cot
        .call(MathQAInput {
            question: "What is 17 * 19?".into(),
        })
        .await?;
    println!("reasoning={}", out.reasoning);
    println!("answer={}", out.answer);
    Ok(())
}
