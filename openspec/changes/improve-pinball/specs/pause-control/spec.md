## ADDED Requirements

### Requirement: Pause Toggle

The system SHALL pause the game when the player presses P while a game is in progress, and SHALL resume it when P is pressed again. While paused, the ball position, score, remaining balls, and all gameplay timers (including the combo window) SHALL NOT change, and the screen SHALL display a PAUSED indicator. Pressing P on the game-over screen SHALL have no effect.

#### Scenario: Pause freezes play

- **WHEN** a ball is in play and the player presses P
- **THEN** the ball SHALL stop moving, the score and remaining balls SHALL stay constant, and PAUSED SHALL be visible on screen

#### Scenario: Resume continues from the same state

- **WHEN** the game is paused and the player presses P
- **THEN** the PAUSED indicator SHALL disappear and the ball SHALL continue from the same position and velocity it had when paused

#### Scenario: Combo window is frozen while paused

- **WHEN** the game is paused for longer than 3000ms with an active combo multiplier above x1
- **THEN** after resuming, the combo multiplier SHALL still hold the value it had at the moment of pausing

#### Scenario: Inputs ignored while paused

- **WHEN** the game is paused and the player presses Space, ArrowLeft, ArrowRight, A, or L
- **THEN** the ball and flippers SHALL NOT change their on-screen state until the game is resumed
