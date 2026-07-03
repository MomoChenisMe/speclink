## Why

The current keyword-based search often returns irrelevant results. Users need a smarter search that understands context and synonyms.

## What Changes

- Replace basic text matching with tokenized search engine
- Add fuzzy matching and typo tolerance
- Implement search result ranking by relevance score

## Capabilities

### New Capabilities

- `smart-search`: Replace basic text matching with tokenized search engine

### Modified Capabilities

（None）

## Impact

- **Code**: `src/lib/stores/search/`, `src-tauri/src/commands/search.rs`
- **Dependencies**: tantivy (Rust full-text search library)
- **Behavior**: Search results are ranked by relevance with highlighted matches
