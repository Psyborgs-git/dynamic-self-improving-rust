# Roadmap

## Phase 0 — Documentation

Goals, architecture, module map, vendor notes, README.

## Phase 1 — Vendor + migrate + scaffold

1. Copy DSRs crates into `crates/` at a pinned commit.
2. Rename to `dsir` / `dsir-macros`.
3. Build on latest stable Rust; fix Edition/dep breakage.
4. Smoke example: Signature → Predict.

## Phase 2 — Compose + run

BestOfN, Refine, Agent, richer Evaluate, program save/load.

## Phase 3 — Self-improving prompts

LabeledFewShot, BootstrapFewShot, LM-proposed COPRO/MIPROv2.

## Phase 4 — Polish

Examples, tests, feature flags, crate docs.

## Out of scope (v0.1)

External GEPA, GRPO, finetune, PoT sandbox, ColBERT, streaming, multimodal, crates.io `dspy-rs` dependency.
