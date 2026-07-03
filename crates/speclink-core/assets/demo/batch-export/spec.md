## ADDED Requirements

### Requirement: Batch Selection

The system SHALL allow users to select multiple items for export using checkbox controls.

#### Scenario: Select all items
- **WHEN** the user clicks the 'Select All' checkbox
- **THEN** all visible items SHALL be selected for export

#### Scenario: Partial selection
- **WHEN** the user selects individual checkboxes
- **THEN** only the checked items SHALL be included in the export

### Requirement: Export Format

The system SHALL support exporting selected items in CSV and JSON formats.

#### Scenario: CSV export
- **WHEN** the user selects CSV format and confirms export
- **THEN** the system SHALL generate a valid CSV file with headers matching the item fields

#### Scenario: JSON export
- **WHEN** the user selects JSON format and confirms export
- **THEN** the system SHALL generate a JSON array containing all selected items
