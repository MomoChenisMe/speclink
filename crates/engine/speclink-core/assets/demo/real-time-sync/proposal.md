## Why

Collaborative editing requires real-time synchronization so multiple users can work on the same document without conflicts.

## What Changes

- Implement WebSocket-based sync protocol
- Add conflict resolution using operational transforms
- Display live cursors and presence indicators

## Capabilities

### New Capabilities

- `real-time-sync`: Implement WebSocket-based sync protocol

### Modified Capabilities

（None）

## Impact

- **Code**: `src/lib/stores/sync/`, `src-tauri/src/commands/sync.rs`
- **Dependencies**: WebSocket library for Rust backend
- **Behavior**: Changes propagate to all connected clients within 200ms
