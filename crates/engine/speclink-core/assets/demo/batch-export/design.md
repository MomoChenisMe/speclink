## Context

Users need to export multiple items at once for external reporting, but currently only single-item export is available.

## Goals / Non-Goals

**Goals:**
- Add batch selection UI for exportable items
- Implement server-side export pipeline supporting CSV and JSON formats
- Add progress indicator for long-running exports

**Non-Goals:**
- Complete rewrite of existing functionality
- Third-party service integrations

## Decisions

### Streaming File Writer

Use streaming writes instead of buffering the entire export in memory to handle large datasets without OOM.

### Format Adapter Pattern

Abstract export format behind a trait so adding new formats (e.g., XML) requires only a new adapter.

## Risks / Trade-offs

- Large exports may take significant time → Show progress bar and allow cancellation
- Concurrent exports could overwhelm disk I/O → Queue exports and process sequentially
