## ADDED Requirements

### Requirement: `state.db` schema MUST be upgraded to version 5 with the `config_state` and `config_change` tables

The state.db migration sequence SHALL include a v5 step that creates two tables (`config_state` singleton row keyed on `id=1`, and `config_change` autoincrement audit table) and inserts a row into `_migrations` with `version=5`. The v5 schema SHALL be additive: the `project`, `change`, and `state_transition` tables defined by earlier migrations SHALL NOT be altered, dropped, or renamed by v5.

The complete column-level schema is normative and is defined in the `config-rw` capability spec under the requirement "state.db SHALL be upgraded to version 5 with `config_state` and `config_change` tables". This requirement in the `local-storage-layout` capability SHALL stand as a sibling pointer ensuring the layout-level contract (which tables exist after a v5 migration) is also captured in the storage spec, paralleling how v3 / v4 migrations were anchored.

Migration v5 SHALL be forward-only; no downgrade SHALL be supported.

When a state.db at schema version 4 is opened by a v5-capable binary, the migration runner SHALL apply v5 atomically: either both tables exist and `_migrations` records `5`, or none of the above changes have been applied.

#### Scenario: v5 migration produces additive schema

- **GIVEN** a state.db at schema version 4 with at least one `project` row, one `change` row, and one `state_transition` row
- **WHEN** the engine opens the db
- **THEN** the migration runner SHALL apply v5, the `_migrations` table SHALL contain a row with `version=5`, all pre-existing `project`, `change`, and `state_transition` rows SHALL remain byte-identical, the `config_state` table SHALL exist with exactly one row (`id=1`, populated by the migration's `INSERT OR IGNORE` step), and the `config_change` table SHALL exist (empty)

#### Scenario: Idempotent re-open after v5 migration

- **GIVEN** a state.db that has already had v5 applied
- **WHEN** the engine re-opens the db
- **THEN** no migration SHALL re-run, `_migrations` SHALL still contain exactly one row per applied version (1, 2, 3, 4, 5), and the `config_state` row SHALL NOT be re-inserted or modified
