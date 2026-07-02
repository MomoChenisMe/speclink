# pinball-table Specification

## Purpose

TBD - created by archiving change 'pinball-game'. Update Purpose after archive.

## Requirements

### Requirement: Ball Launch

The system SHALL start each ball in a launch lane and release it into the play field when the player presses the launch key.

#### Scenario: Launch a waiting ball
- **WHEN** a ball is waiting in the launch lane and the player presses Space
- **THEN** the ball SHALL be given an upward velocity and enter the play field

#### Scenario: Launch is ignored while a ball is already in play
- **WHEN** a ball is already moving in the play field and the player presses Space
- **THEN** the system SHALL ignore the launch input


<!-- @trace
source: pinball-game
updated: 2026-07-01
code:
  - pinball/index.html
-->

---
### Requirement: Flipper Control

The system SHALL provide two bottom flippers that swing from a rest angle to a raised angle while their key is held, and impart upward impulse to a contacting ball.

#### Scenario: Raise the left flipper
- **WHEN** the player holds ArrowLeft
- **THEN** the left flipper SHALL move to its raised angle and return to rest when released

#### Scenario: Flipper strikes the ball
- **WHEN** a descending ball contacts a raising flipper
- **THEN** the ball SHALL rebound upward with added impulse


<!-- @trace
source: pinball-game
updated: 2026-07-01
code:
  - pinball/index.html
-->

---
### Requirement: Bumper Scoring

The system SHALL place bumpers in the play field that rebound the ball and increase the score on contact. The score increase SHALL equal the bumper base value of 100 points multiplied by the current combo multiplier.

#### Scenario: Hit a bumper

- **WHEN** the ball collides with a bumper
- **THEN** the ball SHALL rebound away from the bumper center and the score SHALL increase by the bumper base value multiplied by the current combo multiplier

##### Example: bumper award with combo

- **GIVEN** the score is 0, each bumper awards a base of 100 points, and no scoring element was hit in the last 3000ms
- **WHEN** the ball hits two bumpers with less than 3000ms between the hits
- **THEN** the first hit awards 100 (x1), the second awards 200 (x2), and the score becomes 300

---
### Requirement: Ball Drain And Lives

The system SHALL track a finite number of balls and consume one when a ball drains past the flippers.

#### Scenario: Ball drains
- **WHEN** the ball falls below the flippers into the drain
- **THEN** the remaining ball count SHALL decrease by one and a new ball SHALL be served to the launch lane if any remain

##### Example: lives countdown
- **GIVEN** the player starts with 3 balls
- **WHEN** two balls drain
- **THEN** the remaining ball count is 1


<!-- @trace
source: pinball-game
updated: 2026-07-01
code:
  - pinball/index.html
-->

---
### Requirement: Game Over And Restart

The system SHALL end the game when no balls remain and allow the player to restart.

#### Scenario: Game over
- **WHEN** the last ball drains
- **THEN** the system SHALL display a game-over state and stop serving balls

#### Scenario: Restart after game over
- **WHEN** the game is over and the player presses R
- **THEN** the score SHALL reset to 0, the ball count SHALL reset, and a new ball SHALL be served


<!-- @trace
source: pinball-game
updated: 2026-07-01
code:
  - pinball/index.html
-->

---
### Requirement: Score Display

The system SHALL display the current score and remaining ball count during play.

#### Scenario: HUD updates
- **WHEN** the score or remaining ball count changes
- **THEN** the on-screen HUD SHALL reflect the new values

<!-- @trace
source: pinball-game
updated: 2026-07-01
code:
  - pinball/index.html
-->

---
### Requirement: Slingshot Rebound And Scoring

The system SHALL place two slingshots in the lower play field, one on the left side and one on the right side above the inlanes. WHEN the ball contacts a slingshot face, the ball SHALL rebound away with added impulse and the score SHALL increase by the slingshot base value multiplied by the current combo multiplier.

#### Scenario: Slingshot kicks the ball back

- **WHEN** the ball contacts a slingshot face
- **THEN** the ball SHALL rebound away from the slingshot and the score SHALL increase

##### Example: slingshot award

- **GIVEN** the score is 0, the slingshot base value is 75 points, and the combo multiplier is x1
- **WHEN** the ball hits one slingshot
- **THEN** the score becomes 75

---
### Requirement: Drop Target Bank

The system SHALL place a bank of three drop targets in the play field. WHEN the ball hits a standing drop target, that target SHALL disappear (stop colliding and stop rendering as standing) and the score SHALL increase by the target base value multiplied by the current combo multiplier. WHEN the third target of the bank is cleared, the system SHALL award a flat bonus of 2000 points (not affected by the combo multiplier) and reset all three targets to standing.

#### Scenario: Hitting a standing target

- **WHEN** the ball hits a standing drop target
- **THEN** that target SHALL disappear from the play field and the score SHALL increase

#### Scenario: Clearing the bank

- **WHEN** the ball hits the last standing target of the bank
- **THEN** the score SHALL additionally increase by 2000 and all three targets SHALL reappear as standing

##### Example: full bank sweep

- **GIVEN** the score is 0, each drop target awards a base of 150 points, and each hit happens more than 3000ms after the previous one (multiplier stays x1)
- **WHEN** the ball knocks down all three targets
- **THEN** the score becomes 150 + 150 + 150 + 2000 = 2450 and the three targets are standing again

---
### Requirement: Nudge Impulse

The system SHALL apply a small fixed horizontal impulse to the ball when the player presses N while a ball is in play. The impulse SHALL push the ball toward the horizontal center of the play field. Pressing N SHALL have no effect while the ball is waiting in the launch lane, while the game is paused, after game over, or while the table is tilted.

#### Scenario: Nudge pushes the ball sideways

- **WHEN** a ball is in play and the player presses N
- **THEN** the ball's horizontal velocity SHALL change by a fixed impulse and its position SHALL stay continuous

##### Example: nudge impulse value

- **GIVEN** a ball in play on the left half of the play field with horizontal velocity 0 px/s
- **WHEN** the player presses N
- **THEN** the ball's horizontal velocity becomes +90 px/s (toward the field center)

#### Scenario: Nudge is ignored while waiting or paused

- **WHEN** the ball is waiting in the launch lane, or the game is paused, and the player presses N
- **THEN** the ball's velocity SHALL NOT change

---
### Requirement: Tilt Lockout

The system SHALL declare TILT when the player performs more than 3 nudges within any 3000ms window (180 frames at the 60fps baseline). While tilted, the flippers SHALL NOT respond to ArrowLeft/ArrowRight or A/L, further N presses SHALL have no effect, and the HUD SHALL display TILT. The tilt SHALL end when the current ball drains; the next served ball SHALL have normal flipper control and no TILT indicator.

#### Scenario: Fourth nudge within 3 seconds tilts the table

- **WHEN** the player presses N four times within 3000ms while a ball is in play
- **THEN** TILT SHALL be visible on the HUD and holding ArrowLeft or A SHALL NOT raise the left flipper

#### Scenario: Tilt clears when the ball drains

- **WHEN** a tilted ball drains and at least one ball remains
- **THEN** the next served ball SHALL have working flippers and the TILT indicator SHALL be gone