## Context

Users occasionally need to revert to a previous state of their project after accidental changes, but undo only works for the current session.

## Goals / Non-Goals

**Goals:**
- Implement automatic periodic snapshots of project state
- Add snapshot browser showing timestamped restore points
- Support selective restore of individual files from a snapshot

**Non-Goals:**
- Complete rewrite of existing functionality
- Third-party service integrations

## Decisions

### Copy-on-Write Storage

Use hard links for unchanged files between snapshots to minimize disk usage while maintaining full restore capability.

### Retention Policy

Keep hourly snapshots for 24 hours, daily for 7 days, then weekly for 4 weeks to balance storage with recovery needs.

## Risks / Trade-offs

- Disk usage may grow rapidly → Enforce retention policy and show storage usage in settings
- Snapshot during active write may capture inconsistent state → Use file locking during snapshot creation
