## ADDED Requirements

### Requirement: Particle Burst On Hit

The system SHALL emit a burst of at least 10 particles at the impact point when the ball hits a bumper or a slingshot. Each particle SHALL fade out completely within 600ms (36 frames at the 60fps baseline). The number of simultaneously live particles SHALL be capped at 150 so the fixed-timestep loop keeps its 60fps budget.

#### Scenario: Bumper hit emits particles

- **WHEN** the ball collides with a bumper
- **THEN** at least 10 particles SHALL appear at the contact point and all of them SHALL disappear within 600ms

#### Scenario: Particle cap under rapid hits

- **WHEN** repeated hits occur fast enough that more than 150 particles would be alive at once
- **THEN** the number of drawn particles SHALL NOT exceed 150

### Requirement: Hit Flash

The system SHALL flash the hit element (bumper or slingshot) with a brighter fill for a short duration after contact.

#### Scenario: Flash then restore

- **WHEN** the ball collides with a bumper or a slingshot
- **THEN** that element SHALL render in a brighter flash color for at least 100ms (6 frames at 60fps) and SHALL return to its normal color within 300ms
