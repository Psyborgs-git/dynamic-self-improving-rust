//! ReAct / Agent API sketch (requires OPENAI_API_KEY and tools for a real run).
//!
//! This example shows construction; wire real `rig` tools for production use.

use dsir::{Agent, ChatAdapter, LM, ReAct, Signature, configure};

/// Answer a question, optionally using tools.
#[derive(Signature, Clone, Debug)]
struct ToolQA {
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

    // ReAct with no tools still runs thought→extract.
    let react = ReAct::<ToolQA>::builder().max_steps(3).build();
    let _agent = Agent::<ToolQA>::from_react(react);

    println!("ReAct/Agent constructed. Add tools via ReAct::builder().tool(...) for live runs.");
    Ok(())
}
