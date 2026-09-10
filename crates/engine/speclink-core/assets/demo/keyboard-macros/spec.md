## ADDED Requirements

### Requirement: Recording Mode

The system SHALL provide a recording mode that captures all user actions as an ordered sequence of commands.

#### Scenario: Start recording
- **WHEN** the user activates recording mode via the keyboard shortcut
- **THEN** the system SHALL display a recording indicator and begin capturing actions

#### Scenario: Stop recording
- **WHEN** the user deactivates recording mode
- **THEN** the system SHALL store the captured sequence and offer to save it to a named slot

### Requirement: Macro Playback

The system SHALL replay a saved macro by executing its command sequence in order with configurable speed.

#### Scenario: Normal playback
- **WHEN** the user triggers a saved macro
- **THEN** the system SHALL execute each command in sequence at the configured speed

#### Scenario: Error during playback
- **WHEN** a command in the sequence fails during playback
- **THEN** the system SHALL halt playback and display the error with the failed step number
