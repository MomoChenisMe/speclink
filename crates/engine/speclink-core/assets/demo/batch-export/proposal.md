## Why

Users need to export multiple items at once for external reporting, but currently only single-item export is available.

## What Changes

- Add batch selection UI for exportable items
- Implement server-side export pipeline supporting CSV and JSON formats
- Add progress indicator for long-running exports

## Capabilities

### New Capabilities

- `batch-export`: Add batch selection UI for exportable items

### Modified Capabilities

（None）

## Impact

- **Code**: `src/lib/components/export/`, `src-tauri/src/commands/export.rs`
- **Dependencies**: None (uses built-in fs operations)
- **Behavior**: Users can select multiple items and export them in one action
