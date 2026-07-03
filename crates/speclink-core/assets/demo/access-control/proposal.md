## Why

Shared projects need role-based access control so team leads can restrict who can edit specs vs. who can only view them.

## What Changes

- Define role hierarchy: viewer, editor, admin
- Implement permission checks on all write operations
- Add role assignment UI in project settings

## Capabilities

### New Capabilities

- `access-control`: Define role hierarchy: viewer, editor, admin

### Modified Capabilities

（None）

## Impact

- **Code**: `src/lib/stores/auth/`, `src-tauri/src/commands/permissions.rs`
- **Dependencies**: None (roles stored in project config)
- **Behavior**: Write operations are gated by the user's assigned role
