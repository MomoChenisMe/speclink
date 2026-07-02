# high-scores Specification

## Purpose

TBD - created by archiving change 'improve-pinball'. Update Purpose after archive.

## Requirements

### Requirement: Persistent High Score Table

The system SHALL keep the top 3 final scores across browser sessions in localStorage. WHEN a game ends, the system SHALL insert the final score into the stored list if it ranks among the top 3, keep the list sorted in descending order, truncate it to 3 entries, and persist it. WHEN localStorage is unavailable or throws, the game SHALL continue without persistence and SHALL NOT crash.

#### Scenario: New high score is recorded

- **WHEN** the last ball drains and the final score is higher than at least one stored score
- **THEN** the stored top-3 list SHALL contain the final score in its ranked position after the game ends

##### Example: insertion into a full table

- **GIVEN** the stored scores are 5000, 3000, 1000
- **WHEN** a game ends with a final score of 4000
- **THEN** the stored scores become 5000, 4000, 3000

#### Scenario: Low score leaves the table unchanged

- **WHEN** a game ends with a final score lower than all 3 stored scores
- **THEN** the stored top-3 list SHALL remain unchanged

#### Scenario: Storage unavailable

- **WHEN** localStorage access throws an exception at game end
- **THEN** the game-over screen SHALL still appear and play SHALL remain restartable with R

---
### Requirement: High Score Display At Game Over

The system SHALL display the persisted top-3 scores on the game-over screen.

#### Scenario: Game over shows the table

- **WHEN** the last ball drains and the game-over screen appears
- **THEN** the screen SHALL list up to 3 high scores in descending order, including the just-finished score if it ranks