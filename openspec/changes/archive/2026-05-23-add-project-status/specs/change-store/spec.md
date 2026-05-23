## ADDED Requirements

### Requirement: `change.show` response envelope SHALL include `all_tasks_done` and `next_actions`

The `change.show` operation (CLI: `speclink show change <name>`) SHALL extend its response `data` object with two additional top-level fields: `all_tasks_done: bool` and `next_actions: [string]`. The existing `change` and `artifacts` fields SHALL be preserved unchanged. Both new fields SHALL be present in every successful response — they SHALL NOT be omitted, even when their values are `false` or `[]`.

The `all_tasks_done` field SHALL be read directly from the `change` table's `all_tasks_done` column maintained by the apply-task-ops capability. The runtime SHALL NOT re-parse `tasks.md` to compute this field.

The `next_actions` field SHALL be computed by the runtime as a state-driven lookup. The mapping SHALL be exactly as follows:

| state | all_tasks_done | next_actions |
|---|---|---|
| `proposing` | (ignored) | A filtered subset of `["artifact.write proposal", "artifact.write design", "artifact.write tasks"]` containing only those kinds whose artifact file does NOT yet exist on disk under the change directory |
| `reviewing` | (ignored) | `["review.approve", "review.reject"]` |
| `ready` | (ignored) | `["apply.start"]` |
| `in_progress` | `false` | `["task.done <INDEX>"]` where `<INDEX>` is the 1-based sequential checkbox index (the SAME integer index that `speclink task done <INDEX>` CLI accepts) of the FIRST unchecked task in `tasks.md`, computed by counting both `- [ ]` and `- [x]` lines from the top of the file. When `tasks.md` does not exist OR contains no unchecked checkbox line, the value SHALL be `["task.done"]` (no `<INDEX>` suffix). The runtime SHALL NOT emit the markdown label text (e.g. `2.1`) as the suffix, because the CLI rejects non-integer task arguments with `invalid digit found in string`. |
| `in_progress` | `true` | `["archive.run"]` |
| `code_reviewing` | (ignored) | `["review.approve", "review.reject"]` |
| `archived` | (ignored) | `[]` |

#### Scenario: archived change returns empty next_actions and all_tasks_done is preserved

- **GIVEN** a change `c1` whose `state='archived'` and whose `change` table row's `all_tasks_done` column is `true`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** the response `data.all_tasks_done` SHALL be `true` AND `data.next_actions` SHALL be `[]`

#### Scenario: in_progress change with unfinished tasks suggests task.done with the first pending index

- **GIVEN** a change `c1` whose `state='in_progress'` and whose `all_tasks_done` column is `false`, AND whose `tasks.md` first unchecked line is `- [ ] 2.1 Implement parser` while three earlier `- [x]` lines have already been checked off
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["task.done 4"]` (the 4th checkbox line, 1-based; NOT `["task.done 2.1"]`, since the CLI takes integer index, not label) AND `data.all_tasks_done` SHALL be `false`

#### Scenario: next_actions emits index that the task.done CLI actually accepts (no AI retry loop)

- **GIVEN** a change `c1` whose `state='in_progress'`, whose `all_tasks_done` is `false`, and whose `tasks.md` contains any mixture of `- [ ]` and `- [x]` lines with arbitrary label content (numeric, dotted, slugged, or absent)
- **WHEN** the runtime resolves `change.show` for `c1` AND the caller pipes the resulting `next_actions[0]` (with the leading `task.done ` stripped) into `speclink task done <INDEX>` against the SAME `tasks.md`
- **THEN** the `speclink task done` invocation SHALL succeed (i.e., SHALL NOT fail with `invalid digit found in string` because of label-vs-index mismatch), AND the toggled task SHALL be the same row identified by the next_actions hint

#### Scenario: in_progress change with all tasks done suggests archive.run

- **GIVEN** a change `c1` whose `state='in_progress'` and whose `all_tasks_done` column is `true`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["archive.run"]` AND `data.all_tasks_done` SHALL be `true`

#### Scenario: ready change suggests apply.start

- **GIVEN** a change `c1` whose `state='ready'`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["apply.start"]`

#### Scenario: code_reviewing change offers both approve and reject

- **GIVEN** a change `c1` whose `state='code_reviewing'`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["review.approve", "review.reject"]`

#### Scenario: reviewing change offers both approve and reject

- **GIVEN** a change `c1` whose `state='reviewing'`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["review.approve", "review.reject"]`

#### Scenario: proposing change recommends writing only the missing artifact kinds

- **GIVEN** a change `c1` whose `state='proposing'` and whose change directory contains `proposal.md` only (no `design.md`, no `tasks.md`)
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["artifact.write design", "artifact.write tasks"]`

#### Scenario: proposing change with all three core artifacts present returns empty hint

- **GIVEN** a change `c1` whose `state='proposing'` and whose change directory contains `proposal.md`, `design.md`, and `tasks.md`
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `[]`

#### Scenario: in_progress change without tasks.md falls back to bare task.done hint

- **GIVEN** a change `c1` whose `state='in_progress'` and `all_tasks_done=false`, AND whose change directory does NOT contain a `tasks.md` file
- **WHEN** the runtime resolves `change.show` for `c1`
- **THEN** `data.next_actions` SHALL equal `["task.done"]`

#### Scenario: existing show change fields SHALL be preserved exactly

- **WHEN** the runtime resolves `change.show` for any change
- **THEN** the response `data` SHALL still contain a `change` field (with the same row metadata as before this requirement) AND an `artifacts` field (with the same array shape as before this requirement) AND no field SHALL be removed or renamed
