## Why

Users want to customize the application appearance beyond the built-in light/dark toggle, including custom color palettes and font choices.

## What Changes

- Build a theme engine supporting user-defined color palettes
- Add theme preview and live switching without page reload
- Persist theme preferences per user profile

## Capabilities

### New Capabilities

- `theme-engine`: Build a theme engine supporting user-defined color palettes

### Modified Capabilities

（None）

## Impact

- **Code**: `src/lib/stores/theme/`, `src/lib/components/settings/`
- **Dependencies**: None (uses CSS custom properties)
- **Behavior**: Users can create, edit, and switch between custom themes instantly
