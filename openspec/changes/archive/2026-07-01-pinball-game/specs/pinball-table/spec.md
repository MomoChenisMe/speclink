## ADDED Requirements

### Requirement: Ball Launch

The system SHALL start each ball in a launch lane and release it into the play field when the player presses the launch key.

#### Scenario: Launch a waiting ball
- **WHEN** a ball is waiting in the launch lane and the player presses Space
- **THEN** the ball SHALL be given an upward velocity and enter the play field

#### Scenario: Launch is ignored while a ball is already in play
- **WHEN** a ball is already moving in the play field and the player presses Space
- **THEN** the system SHALL ignore the launch input

### Requirement: Flipper Control

The system SHALL provide two bottom flippers that swing from a rest angle to a raised angle while their key is held, and impart upward impulse to a contacting ball.

#### Scenario: Raise the left flipper
- **WHEN** the player holds ArrowLeft
- **THEN** the left flipper SHALL move to its raised angle and return to rest when released

#### Scenario: Flipper strikes the ball
- **WHEN** a descending ball contacts a raising flipper
- **THEN** the ball SHALL rebound upward with added impulse

### Requirement: Bumper Scoring

The system SHALL place bumpers in the play field that rebound the ball and increase the score on contact.

#### Scenario: Hit a bumper
- **WHEN** the ball collides with a bumper
- **THEN** the ball SHALL rebound away from the bumper center and the score SHALL increase

##### Example: bumper award
- **GIVEN** the score is 0 and each bumper awards 100 points
- **WHEN** the ball hits two bumpers in succession
- **THEN** the score becomes 200

### Requirement: Ball Drain And Lives

The system SHALL track a finite number of balls and consume one when a ball drains past the flippers.

#### Scenario: Ball drains
- **WHEN** the ball falls below the flippers into the drain
- **THEN** the remaining ball count SHALL decrease by one and a new ball SHALL be served to the launch lane if any remain

##### Example: lives countdown
- **GIVEN** the player starts with 3 balls
- **WHEN** two balls drain
- **THEN** the remaining ball count is 1

### Requirement: Game Over And Restart

The system SHALL end the game when no balls remain and allow the player to restart.

#### Scenario: Game over
- **WHEN** the last ball drains
- **THEN** the system SHALL display a game-over state and stop serving balls

#### Scenario: Restart after game over
- **WHEN** the game is over and the player presses R
- **THEN** the score SHALL reset to 0, the ball count SHALL reset, and a new ball SHALL be served

### Requirement: Score Display

The system SHALL display the current score and remaining ball count during play.

#### Scenario: HUD updates
- **WHEN** the score or remaining ball count changes
- **THEN** the on-screen HUD SHALL reflect the new values
