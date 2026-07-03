## ADDED Requirements

### Requirement: Role Hierarchy

The system SHALL enforce a role hierarchy where admin > editor > viewer, and higher roles inherit all permissions of lower roles.

#### Scenario: Editor permissions
- **WHEN** a user with the editor role attempts to modify a spec
- **THEN** the system SHALL allow the modification

#### Scenario: Viewer restriction
- **WHEN** a user with the viewer role attempts to modify a spec
- **THEN** the system SHALL reject the modification and display an access denied message

### Requirement: Role Assignment

The system SHALL allow admins to assign and revoke roles for project members through the settings interface.

#### Scenario: Assign role
- **WHEN** an admin selects a user and assigns the editor role
- **THEN** the system SHALL update the user's permissions immediately

#### Scenario: Self-demotion guard
- **WHEN** the last admin attempts to change their own role to a lower level
- **THEN** the system SHALL reject the change to prevent lockout
