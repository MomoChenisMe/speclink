## ADDED Requirements

### Requirement: Fuzzy Matching

The system SHALL support fuzzy matching with a configurable edit distance to tolerate typos in search queries.

#### Scenario: Single typo
- **WHEN** the user searches for 'authentcation'
- **THEN** the system SHALL return results matching 'authentication'

#### Scenario: No match
- **WHEN** the user searches for a term with no close matches
- **THEN** the system SHALL display an empty result set with a suggestion to refine the query

### Requirement: Relevance Ranking

The system SHALL rank search results by relevance score computed from term frequency, field weights, and recency.

#### Scenario: Title match priority
- **WHEN** a search term appears in both title and body of different items
- **THEN** the item with the title match SHALL rank higher

#### Scenario: Recent items boost
- **WHEN** two items have equal term frequency
- **THEN** the more recently modified item SHALL rank higher
