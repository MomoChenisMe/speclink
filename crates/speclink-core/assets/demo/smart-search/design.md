## Context

The current keyword-based search often returns irrelevant results. Users need a smarter search that understands context and synonyms.

## Goals / Non-Goals

**Goals:**
- Replace basic text matching with tokenized search engine
- Add fuzzy matching and typo tolerance
- Implement search result ranking by relevance score

**Non-Goals:**
- Complete rewrite of existing functionality
- Third-party service integrations

## Decisions

### Inverted Index Strategy

Build an inverted index at project load time for O(1) lookup performance instead of scanning all files per query.

### Scoring Formula

Use BM25 scoring with custom field boosts (title: 3x, body: 1x) for balanced relevance.

## Risks / Trade-offs

- Index build time may slow startup → Build index in background thread
- Index size may grow large → Limit indexed fields and use compression
