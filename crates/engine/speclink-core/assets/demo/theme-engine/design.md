## Context

Users want to customize the application appearance beyond the built-in light/dark toggle, including custom color palettes and font choices.

## Goals / Non-Goals

**Goals:**
- Build a theme engine supporting user-defined color palettes
- Add theme preview and live switching without page reload
- Persist theme preferences per user profile

**Non-Goals:**
- Complete rewrite of existing functionality
- Third-party service integrations

## Decisions

### CSS Custom Properties

Use CSS custom properties for runtime theme switching without rebuilding stylesheets or reloading the page.

### Theme Schema Validation

Validate theme objects against a JSON schema before applying to prevent invalid styles from breaking the UI.

## Risks / Trade-offs

- Complex themes may cause contrast issues → Provide WCAG contrast checker in theme editor
- Too many custom properties may impact render performance → Limit palette to 12 core variables
