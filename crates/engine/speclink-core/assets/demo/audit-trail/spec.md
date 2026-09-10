## ADDED Requirements

### Requirement: Log Immutability

The system SHALL store audit entries in an append-only log that prevents modification or deletion of existing entries.

#### Scenario: Write protection
- **WHEN** any attempt is made to update or delete an existing audit entry
- **THEN** the system SHALL reject the operation and return an error

#### Scenario: Entry creation
- **WHEN** a user performs a write operation
- **THEN** the system SHALL create an audit entry containing the actor identity, timestamp, operation type, and affected resource

### Requirement: Log Viewer

The system SHALL provide a paginated viewer for browsing audit entries with filters for date range, actor, and operation type.

#### Scenario: Filter by actor
- **WHEN** the user selects a specific actor from the filter dropdown
- **THEN** the viewer SHALL display only audit entries created by that actor

#### Scenario: Pagination
- **WHEN** the audit log contains more than 50 entries
- **THEN** the viewer SHALL display entries in pages of 50 with navigation controls
