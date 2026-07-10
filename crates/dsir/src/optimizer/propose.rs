//! Shared helpers for LM-proposed instruction candidates.

use anyhow::Result;

use crate::core::lm::chat::{Chat, Message};
use crate::core::settings::GLOBAL_SETTINGS;
use crate::LM;

/// Ask an LM to propose improved instruction variants.
///
/// Always includes `base_instruction` as candidate 0. If no LM is configured (or the
/// call fails), falls back to deterministic heuristic variants so offline tests work.
pub async fn propose_instructions_with_hint(
    base_instruction: &str,
    output_hint: &str,
    count: usize,
    prompt_model: Option<&LM>,
) -> Result<Vec<String>> {
    let count = count.max(1);
    let mut candidates = Vec::with_capacity(count);
    candidates.push(base_instruction.to_string());
    if count == 1 {
        return Ok(candidates);
    }

    let lm = prompt_model.cloned().or_else(|| {
        GLOBAL_SETTINGS
            .read()
            .ok()
            .and_then(|g| g.as_ref().map(|s| (*s.lm).clone()))
    });

    let Some(lm) = lm else {
        return Ok(heuristic_variants(base_instruction, output_hint, count));
    };

    let mut chat = Chat::new(vec![Message::system(
        "You improve LM program instructions. Reply with exactly the requested number of \
         candidate instructions, each separated by a line containing only ---.",
    )]);
    chat.push_message(Message::user(format!(
        "Current instruction:\n{base_instruction}\n\n\
         Output fields: {output_hint}\n\n\
         Propose {} improved instruction candidates. Be specific and actionable. \
         Separate candidates with a line containing only ---.",
        count - 1
    )));

    match lm.call(chat, vec![]).await {
        Ok(response) => {
            let text = response.output.content().to_string();
            for part in text.split("\n---\n") {
                let trimmed = part.trim();
                if !trimmed.is_empty() && candidates.len() < count {
                    candidates.push(trimmed.to_string());
                }
            }
            while candidates.len() < count {
                let idx = candidates.len();
                candidates.push(format!(
                    "{base_instruction}\n\nBe explicit and concise for outputs: {output_hint} (variant {idx})."
                ));
            }
            Ok(candidates)
        }
        Err(_) => Ok(heuristic_variants(base_instruction, output_hint, count)),
    }
}

fn heuristic_variants(base: &str, output_hint: &str, count: usize) -> Vec<String> {
    let mut candidates = vec![base.to_string()];
    for idx in 1..count {
        candidates.push(format!(
            "{base}\n\nOptimization hint ({idx}): Be explicit and concise for `{output_hint}`."
        ));
    }
    candidates
}
