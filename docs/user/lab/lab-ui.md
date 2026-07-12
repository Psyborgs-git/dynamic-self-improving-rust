# Lab UI

Local experiment dashboard for browsing lab runs.

`dsir-lab-ui` is a companion binary that provides a web UI for inspecting experiments, comparing runs, and viewing promoted programs.

## Start the UI

```bash
cargo run -p dsir-lab-ui
```

Open the URL printed to the terminal (default local address).

## What it shows

- Authored programs and datasets in the lab workdir
- Optimization runs with train/val scores
- Comparison tables across optimizers
- Promoted program status

## Workdir

Point the UI at the same workdir used by `Lab::open`. Runs created by `cargo run -p dsir --example 05_lab` appear after pointing at that temp directory.

## See also

- [Lab overview](overview.md)
- [Registry](registry.md)
