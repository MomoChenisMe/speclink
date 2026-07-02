## ADDED Requirements

### Requirement: Slingshot Rebound And Scoring

The system SHALL place two slingshots in the lower play field, one on the left side and one on the right side above the inlanes. WHEN the ball contacts a slingshot face, the ball SHALL rebound away with added impulse and the score SHALL increase by the slingshot base value multiplied by the current combo multiplier.

#### Scenario: Slingshot kicks the ball back

- **WHEN** the ball contacts a slingshot face
- **THEN** the ball SHALL rebound away from the slingshot and the score SHALL increase

##### Example: slingshot award

- **GIVEN** the score is 0, the slingshot base value is 75 points, and the combo multiplier is x1
- **WHEN** the ball hits one slingshot
- **THEN** the score becomes 75

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

### Requirement: Tilt Lockout

The system SHALL declare TILT when the player performs more than 3 nudges within any 3000ms window (180 frames at the 60fps baseline). While tilted, the flippers SHALL NOT respond to ArrowLeft/ArrowRight or A/L, further N presses SHALL have no effect, and the HUD SHALL display TILT. The tilt SHALL end when the current ball drains; the next served ball SHALL have normal flipper control and no TILT indicator.

#### Scenario: Fourth nudge within 3 seconds tilts the table

- **WHEN** the player presses N four times within 3000ms while a ball is in play
- **THEN** TILT SHALL be visible on the HUD and holding ArrowLeft or A SHALL NOT raise the left flipper

#### Scenario: Tilt clears when the ball drains

- **WHEN** a tilted ball drains and at least one ball remains
- **THEN** the next served ball SHALL have working flippers and the TILT indicator SHALL be gone

## MODIFIED Requirements

### Requirement: Bumper Scoring

The system SHALL place bumpers in the play field that rebound the ball and increase the score on contact. The score increase SHALL equal the bumper base value of 100 points multiplied by the current combo multiplier.

#### Scenario: Hit a bumper

- **WHEN** the ball collides with a bumper
- **THEN** the ball SHALL rebound away from the bumper center and the score SHALL increase by the bumper base value multiplied by the current combo multiplier

##### Example: bumper award with combo

- **GIVEN** the score is 0, each bumper awards a base of 100 points, and no scoring element was hit in the last 3000ms
- **WHEN** the ball hits two bumpers with less than 3000ms between the hits
- **THEN** the first hit awards 100 (x1), the second awards 200 (x2), and the score becomes 300
