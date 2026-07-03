## Context

For compliance and debugging, teams need a complete history of who changed what and when, but no audit logging exists today.

## Goals / Non-Goals

**Goals:**
- Add append-only audit log for all write operations
- Implement audit log viewer with filtering and pagination
- Support exporting audit logs for external compliance tools

**Non-Goals:**
- Complete rewrite of existing functionality
- Third-party service integrations

## Decisions

### Append-Only Storage

Use a dedicated SQLite table with no UPDATE/DELETE permissions to guarantee immutability at the storage level.

### Structured Log Format

Store entries as structured JSON rather than plain text to enable programmatic querying and filtering.

## Risks / Trade-offs

- Log volume may grow unbounded → Implement configurable retention period with archival
- High-frequency writes may impact performance → Batch audit writes with a flush interval
