## Why

For compliance and debugging, teams need a complete history of who changed what and when, but no audit logging exists today.

## What Changes

- Add append-only audit log for all write operations
- Implement audit log viewer with filtering and pagination
- Support exporting audit logs for external compliance tools

## Capabilities

### New Capabilities

- `audit-trail`: Add append-only audit log for all write operations

### Modified Capabilities

（None）

## Impact

- **Code**: `src/lib/components/audit/`, `src-tauri/src/commands/audit.rs`
- **Dependencies**: None (uses SQLite for storage)
- **Behavior**: All mutations are logged with actor, timestamp, and changed fields
