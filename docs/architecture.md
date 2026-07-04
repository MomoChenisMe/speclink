# Architecture

> 繁體中文版:[architecture.zh-TW.md](architecture.zh-TW.md)

Speclink is a spec-driven development (SDD) engine. Its central architectural
commitment is that **the engine does not know how spec documents are stored**.
Documents may live as Markdown in a git repository today and behind a team
system's REST API tomorrow — the engine's workflow logic stays identical.

## The three layers

```
┌────────────────────────────────────────────────────────┐
│  Presentation / Host                                   │
│  speclink-cli — argument parsing, rendering, colors,   │
│  the assembly point that picks a storage adapter       │
└──────────────┬─────────────────────────────────────────┘
               │  calls engine flows with  &dyn Store
┌──────────────▼─────────────────────────────────────────┐
│  Engine                                                │
│  speclink-core — SDD workflow logic: changes,          │
│  artifacts, validate/analyze/drift/archive, tasks,     │
│  discussions, schemas, instructions                    │
└──────────────┬─────────────────────────────────────────┘
               │  Store trait (the storage seam)
┌──────────────▼─────────────────────────────────────────┐
│  Storage                                               │
│  speclink-fs — the default adapter: the classic        │
│  openspec/ directory layout on the local filesystem    │
└────────────────────────────────────────────────────────┘
```

- **speclink-core** (engine) owns every workflow rule: what makes a change
  complete, how deltas merge into canonical specs, what drift means, how
  discussions conclude. It never touches the spec directory with `std::fs`;
  an architectural test (`crates/speclink-core/tests/no_direct_fs.rs`)
  enforces this.
- **speclink-fs** (storage) owns every layout fact: `specs/<cap>/spec.md`,
  `changes/<name>/`, `changes/archive/<date>-<name>/`, `discussions/<slug>.md`,
  `config.yaml`, mtime-derived ordering, and archive naming. Swapping this
  crate out is how a different storage backend plugs in.
- **speclink-cli** (host) is the only place the two meet: each command builds
  an `FsStore` and hands it to core flows as `&dyn Store`.

## The storage seam: `Store`

`speclink_core::store::Store` is a synchronous, object-safe trait whose
vocabulary is the SDD domain, not the filesystem:

- changes — list / find / create / exists / `updated_at_secs`
- artifacts — read / write / exists (identified by their schema output path,
  e.g. `proposal.md`, `specs/<cap>/spec.md`)
- delta and canonical specs — capability enumeration, read / write
- archive — move a change under its dated name, stamp its metadata
- discussions — create / read / append / archive, with collision-safe naming
- workflow config — raw document read (`openspec/config.yaml` in fs terms)

Two kinds of data deliberately stay **outside** the seam:

- **Host workspace data** (`speclink_core::workspace::Workspace`): the
  `.speclink/` work directory (touched records, archive snapshots), the
  `.speclink.yaml` app config, and project-root discovery. These belong to
  the machine running the engine — a remote storage backend would still keep
  them local.
- **Git interaction**: drift's commit-window analysis and archive's `@trace`
  collection are engine-flow concerns about the *code* repository, not about
  where spec documents live.

## Behavioral guarantee

The seam was cut refactoring-only: every CLI command's human output, `--json`
payload, exit code, and filesystem effect is byte-identical to the
pre-refactor engine (verified by a twin-sandbox regression harness across
parity, color, and drift/archive scenarios).

## Where this goes next

This seam is the foundation for three planned changes:

```
store-trait-and-fs-adapter (this)
        │
        ├─► config-system-rework
        │     re-homes workflow-level settings into the store-side
        │     config.yaml; host-side .speclink.yaml keeps bootstrap keys;
        │     tools become self-describing descriptors
        │
        ├─► verb-contract-and-remote-client
        │     a remote Store implementation speaking a REST verb contract
        │     (PAT auth, optimistic locking) — PO/PM tools work without a
        │     local repo while RD/QA stay on git
        │
        └─► node-sdk
              @speclink/engine via napi: createEngine with an injectable
              JavaScript Store object — the object-safe seam is exactly
              what makes dynamic injection possible
```

Each of those changes will extend this document with its own section when it
lands.
