# combo-scoring Specification

## Purpose

TBD - created by archiving change 'improve-pinball'. Update Purpose after archive.

## Requirements

### Requirement: Combo Multiplier

The system SHALL maintain a combo multiplier that starts at x1. WHEN the ball hits a bumper, slingshot, or drop target within 3000ms (180 frames at the 60fps baseline) of the previous such hit, the multiplier SHALL increase by 1 up to a maximum of x5 before the points for that hit are awarded. WHEN 3000ms elapse without any such hit, the multiplier SHALL reset to x1. Points awarded for bumper, slingshot, and drop target hits SHALL equal the element base value multiplied by the multiplier in effect for that hit.

#### Scenario: Multiplier increments inside the window

- **WHEN** the ball hits scoring elements repeatedly with less than 3000ms between consecutive hits
- **THEN** each successive hit SHALL award its base points multiplied by an increasing factor, capped at x5

##### Example: chained bumper hits

- **GIVEN** the score is 0, each bumper awards a base of 100 points, and no element was hit in the last 3000ms
- **WHEN** the ball hits a bumper three times with 1000ms between hits
- **THEN** the hits award 100 (x1), 200 (x2), 300 (x3) and the score becomes 600

#### Scenario: Multiplier resets after timeout

- **WHEN** 3000ms pass after the last bumper/slingshot/drop-target hit
- **THEN** the next hit SHALL be awarded at x1

#### Scenario: Multiplier cap

- **WHEN** the ball chains six or more hits, each within 3000ms of the previous one
- **THEN** the fifth and later hits SHALL be awarded at x5 and the multiplier SHALL NOT exceed x5

---
### Requirement: Multiplier HUD Display

The system SHALL display the current combo multiplier on the HUD during play.

#### Scenario: HUD shows current multiplier

- **WHEN** the combo multiplier changes to any value from x1 to x5
- **THEN** the HUD SHALL show that multiplier value within the next rendered frame