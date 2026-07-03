## Context

Power users perform repetitive sequences of actions daily. Recording and replaying keyboard macros would significantly improve productivity.

## Goals / Non-Goals

**Goals:**
- Add macro recording mode that captures user actions as a sequence
- Implement macro playback with variable speed
- Support saving macros to named slots for reuse

**Non-Goals:**
- Complete rewrite of existing functionality
- Third-party service integrations

## Decisions

### Command Serialization

Serialize actions as JSON command objects rather than raw keystrokes to make macros portable across keyboard layouts.

### Playback Isolation

Run macro playback in an isolated context to prevent accidental side effects on unsaved work.

## Risks / Trade-offs

- Macros recorded on one version may break on updates → Version-stamp macro format
- Infinite loops from recursive macros → Cap playback to 1000 steps maximum
