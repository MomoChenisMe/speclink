# speclink

SpecLink — Spec-Driven Development workflow CLI for AI agents.

This repository hosts the Cargo workspace with four crates:

| crate            | role                                                             |
| ---------------- | ---------------------------------------------------------------- |
| `cli`            | `speclink` binary, command surface (clap)                        |
| `runtime`        | workflow orchestration over the `Provider` trait                 |
| `provider`       | `Provider` async trait, shared types, config + resolution        |
| `provider-local` | local filesystem implementation of `Provider` (no remote calls)  |

## Status

Pre-alpha. The first vertical slice (`speclink propose create`) is being
bootstrapped under the change `bootstrap-workspace-and-propose-create` in
`openspec/changes/`.

## License

Dual-licensed under MIT OR Apache-2.0. See `LICENSE` for details.
