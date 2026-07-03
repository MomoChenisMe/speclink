## ADDED Requirements

### Requirement: Custom Palette

The system SHALL allow users to define custom color palettes with primary, secondary, accent, background, and text colors.

#### Scenario: Create palette
- **WHEN** the user opens the theme editor and sets custom color values
- **THEN** the system SHALL apply the palette immediately as a live preview

#### Scenario: Invalid color
- **WHEN** the user enters an invalid color value
- **THEN** the system SHALL display a validation error and keep the previous color

### Requirement: Theme Persistence

The system SHALL persist the active theme selection across application restarts.

#### Scenario: Save on switch
- **WHEN** the user selects a different theme
- **THEN** the system SHALL save the selection to local storage and apply it on next launch

#### Scenario: Fallback on corruption
- **WHEN** the persisted theme data is corrupted or missing
- **THEN** the system SHALL fall back to the default theme without crashing
