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
pub fn load_program_state<M>(module: &mut M, state: ProgramState) -> Result<()>
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
