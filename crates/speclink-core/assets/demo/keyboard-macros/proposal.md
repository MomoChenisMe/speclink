## Why

Power users perform repetitive sequences of actions daily. Recording and replaying keyboard macros would significantly improve productivity.

## What Changes

- Add macro recording mode that captures user actions as a sequence
- Implement macro playback with variable speed
- Support saving macros to named slots for reuse

## Capabilities

### New Capabilities

- `keyboard-macros`: Add macro recording mode that captures user actions as a sequence

### Modified Capabilities

（None）

## Impact

- **Code**: `src/lib/stores/macros/`, `src/lib/components/macro-bar/`
- **Dependencies**: None
- **Behavior**: Users can record, save, and replay action sequences via keyboard shortcuts
