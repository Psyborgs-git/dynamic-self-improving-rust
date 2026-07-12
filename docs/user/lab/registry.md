# Registry

HTTP remote run registry for distributed lab workflows.

The `dsir-registry` companion binary stores experiment runs remotely so teams can share optimization results.

## Start the registry

```bash
cargo run -p dsir-registry
```

Configure the listen address via environment variables (see crate source for defaults).

## Client usage

The lab's registry client helpers upload and fetch runs from a remote registry URL. Use this when:

- Multiple machines run optimizations against the same program
- You want a central store of promoted programs
- The [lab UI](lab-ui.md) reads from a shared backend

## Local vs remote

By default, `Lab::open` uses a local filesystem workdir. Point the lab at a remote registry when you need shared state across machines.

## See also

- [Lab overview](overview.md)
- [Lab UI](lab-ui.md)
