# Roadmap

## Phase 0 — Documentation (done)

- [x] Goals, architecture, module map, vendor notes, README
- [x] User-facing guides under [`docs/user/`](../user/index.md)

## Phase 1 — Vendor + migrate + scaffold (done)

1. Copy DSRs crates into `crates/` at a pinned commit.
2. Rename to `dsir` / `dsir-macros`.
3. Build on latest stable Rust; fix Edition/dep breakage.
4. Smoke example: Signature → Predict.

## Phase 2 — Compose + run (done)

BestOfN, Refine, Agent, richer Evaluate, program save/load.

## Phase 3 — Self-improving prompts (done)

LabeledFewShot, BootstrapFewShot, LM-proposed COPRO/MIPROv2.

## Phase 4 — Polish (in progress)

- [x] Examples `01`–`06`, crate docs, user guides
- [ ] Port additional upstream examples (tracing, HotPotQA, GEPA live demos)
- [ ] Optional hosted docs site (Mintlify)

## Phase 5 — Dynamic signatures + self-improvement lab (done)

1. `DynSignature` from JSON schema + string DSL (nested object/list).
2. `DynPredict` / `DynModule`, raw adapter I/O.
3. `ProgramGraph`, `StrategyFactory`, structural optimization.
4. `compile_dyn` for Bootstrap/COPRO/MIPRO/GEPA.
5. `Lab` + local/remote registry + experiment UI.
6. Examples `05_lab`, `06_graph`.

## Out of scope (v0.1)

External GEPA, GRPO, finetune, PoT sandbox, ColBERT, streaming, multimodal, crates.io `dspy-rs` dependency.

## See also

- [User documentation hub](../README.md)
- [Examples index](../user/guides/examples.md)
