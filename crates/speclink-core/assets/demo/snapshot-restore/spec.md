## ADDED Requirements

### Requirement: Automatic Snapshots

The system SHALL create automatic snapshots of the project state at configurable intervals and before destructive operations.

#### Scenario: Periodic snapshot
- **WHEN** the configured interval has elapsed since the last snapshot
- **THEN** the system SHALL create a new snapshot in the background without interrupting the user

#### Scenario: Pre-operation snapshot
- **WHEN** the user initiates a bulk delete or archive operation
- **THEN** the system SHALL create a snapshot before executing the operation

### Requirement: Selective Restore

The system SHALL allow users to restore individual files from a snapshot without replacing the entire project state.

#### Scenario: Single file restore
- **WHEN** the user selects a specific file from a snapshot and confirms restore
- **THEN** the system SHALL replace only that file with the snapshot version

#### Scenario: Conflict warning
- **WHEN** the file to be restored has been modified since the snapshot
- **THEN** the system SHALL warn the user and offer to create a backup before restoring
