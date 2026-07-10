//! Program-level save/load of optimized predictor state.

use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::{PredictState, visit_named_predictors_mut};
use crate::Facet;

/// Snapshot of every [`Predict`](crate::Predict) leaf in a module.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProgramState {
    /// Map from dotted predictor path → state (demos + instruction).
    pub predictors: BTreeMap<String, PredictState>,
}

/// Dump all predictor leaves in `module` to a [`ProgramState`].
pub fn dump_program_state<M>(module: &mut M) -> Result<ProgramState>
where
    M: for<'a> Facet<'a>,
{
    let mut predictors = BTreeMap::new();
    visit_named_predictors_mut(module, |name, predictor| {
        predictors.insert(name.to_string(), predictor.dump_state());
        ControlFlow::Continue(())
    })?;
    Ok(ProgramState { predictors })
}

/// Restore predictor state into `module` from a [`ProgramState`].
///
/// Predictors present in the snapshot but missing from the module are ignored.
/// Predictors in the module but missing from the snapshot are left unchanged.
///
/// If any predictor fails to load, previously updated predictors are rolled back to
/// their pre-load state before the error is returned.
pub fn load_program_state<M>(module: &mut M, state: ProgramState) -> Result<()>
where
    M: for<'a> Facet<'a>,
{
    let backup = dump_program_state(module)?;
    if let Err(err) = apply_program_state(module, &state) {
        restore_program_state(module, backup);
        return Err(err);
    }
    Ok(())
}

fn apply_program_state<M>(module: &mut M, state: &ProgramState) -> Result<()>
where
    M: for<'a> Facet<'a>,
{
    let mut first_err: Option<anyhow::Error> = None;
    visit_named_predictors_mut(module, |name, predictor| {
        if let Some(leaf) = state.predictors.get(name) {
            if let Err(err) = predictor.load_state(leaf.clone()) {
                first_err = Some(err);
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    })?;
    if let Some(err) = first_err {
        return Err(err);
    }
    Ok(())
}

fn restore_program_state<M>(module: &mut M, state: ProgramState)
where
    M: for<'a> Facet<'a>,
{
    let _ = apply_program_state(module, &state);
}

/// Save program state to a JSON file.
pub fn save_program<M, P: AsRef<Path>>(module: &mut M, path: P) -> Result<()>
where
    M: for<'a> Facet<'a>,
{
    let state = dump_program_state(module)?;
    let json = serde_json::to_string_pretty(&state).context("serialize program state")?;
    std::fs::write(path.as_ref(), json).context("write program state")?;
    Ok(())
}

/// Load program state from a JSON file into `module`.
pub fn load_program<M, P: AsRef<Path>>(module: &mut M, path: P) -> Result<()>
where
    M: for<'a> Facet<'a>,
{
    let bytes = std::fs::read(path.as_ref()).context("read program state")?;
    let state: ProgramState = serde_json::from_slice(&bytes).context("parse program state")?;
    load_program_state(module, state)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use dsir_macros::Signature;
    use serde_json::json;

    use crate::core::DynPredictor;
    use crate::data::example::Example as RawExample;
    use crate::{Module, Predict, PredictError, Predicted};

    use super::*;

    #[derive(Signature, Clone, Debug)]
    struct Toy {
        #[input]
        prompt: String,
        #[output]
        answer: String,
    }

    #[derive(facet::Facet)]
    #[facet(crate = facet)]
    struct TwoPredictorModule {
        first: Predict<Toy>,
        second: Predict<Toy>,
    }

    impl Module for TwoPredictorModule {
        type Input = ToyInput;
        type Output = ToyOutput;

        async fn forward(&self, _input: ToyInput) -> Result<Predicted<ToyOutput>, PredictError> {
            unimplemented!()
        }
    }

    #[test]
    fn load_program_state_rolls_back_on_failure() -> Result<()> {
        let mut module = TwoPredictorModule {
            first: Predict::new(),
            second: Predict::new(),
        };
        DynPredictor::set_instruction(&mut module.first, "first-original".to_string());
        DynPredictor::set_instruction(&mut module.second, "second-original".to_string());

        let good_state = dump_program_state(&mut module)?;
        let first_path = good_state
            .predictors
            .keys()
            .find(|name| name.ends_with("first"))
            .expect("first predictor path")
            .clone();
        let second_path = good_state
            .predictors
            .keys()
            .find(|name| name.ends_with("second"))
            .expect("second predictor path")
            .clone();

        let mut bad_state = good_state.clone();
        bad_state.predictors.insert(
            first_path,
            PredictState {
                instruction_override: Some("first-updated".to_string()),
                ..PredictState::default()
            },
        );
        bad_state.predictors.insert(
            second_path,
            PredictState {
                demos: vec![RawExample::new(
                    HashMap::from([("missing_field".to_string(), json!("x"))]),
                    vec!["missing_field".to_string()],
                    vec!["answer".to_string()],
                )],
                instruction_override: Some("second-should-not-stick".to_string()),
            },
        );

        let err =
            load_program_state(&mut module, bad_state).expect_err("invalid second demo should fail");
        assert!(!err.to_string().is_empty());
        assert_eq!(DynPredictor::instruction(&module.first), "first-original");
        assert_eq!(DynPredictor::instruction(&module.second), "second-original");
        Ok(())
    }
}
