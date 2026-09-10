## Why

Users occasionally need to revert to a previous state of their project after accidental changes, but undo only works for the current session.

## What Changes

- Implement automatic periodic snapshots of project state
- Add snapshot browser showing timestamped restore points
- Support selective restore of individual files from a snapshot

## Capabilities

### New Capabilities

- `snapshot-restore`: Implement automatic periodic snapshots of project state

### Modified Capabilities

（None）

## Impact

- **Code**: `src/lib/components/snapshots/`, `src-tauri/src/commands/snapshot.rs`
- **Dependencies**: None (uses filesystem copies)
- **Behavior**: Project state is snapshotted every 30 minutes and on significant operations
