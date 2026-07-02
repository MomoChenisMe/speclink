# audio-feedback Specification

## Purpose

TBD - created by archiving change 'improve-pinball'. Update Purpose after archive.

## Requirements

### Requirement: Synthesized Sound Effects

The system SHALL synthesize all sound effects at runtime with the WebAudio API and SHALL NOT load any external audio file. The system SHALL play a distinct short sound for each of these game events: flipper activation, bumper hit, slingshot hit, ball drain, and game over.

#### Scenario: Flipper strike sound

- **WHEN** the player presses ArrowLeft or A (left flipper) or ArrowRight or L (right flipper) while sound is enabled
- **THEN** a synthesized flipper sound SHALL start within one frame (16.7ms at the 60fps baseline)

#### Scenario: Bumper and slingshot hit sounds

- **WHEN** the ball collides with a bumper or with a slingshot while sound is enabled
- **THEN** a synthesized hit sound SHALL play, and the bumper sound SHALL differ in pitch from the slingshot sound

#### Scenario: Drain and game over sounds

- **WHEN** the ball falls below the flippers into the drain while sound is enabled
- **THEN** a drain sound SHALL play, and if that ball was the last remaining ball a game-over sound SHALL also play

---
### Requirement: Mute Toggle

The system SHALL toggle all sound output off and on when the player presses M, at any point during play or at the game-over screen.

#### Scenario: Mute silences all effects

- **WHEN** the player presses M while sound is enabled
- **THEN** no sound SHALL play for any subsequent game event until M is pressed again

#### Scenario: Unmute restores effects

- **WHEN** the player presses M while sound is muted
- **THEN** subsequent game events SHALL play their sounds again