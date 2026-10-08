# Speclink brand assets

[繁體中文](README.md) · **English**

The interlocking S consists of two complementary shapes, expressing connections between requirements, specifications and implementation. The app icon uses a teal rounded tile with a white S. Outside the tile is transparent, including in the Windows ICO; rounded corners do not depend on an OS mask.

## Colors

| Role | Value | Use |
| --- | --- | --- |
| Teal | `#167873` | Symbol, app tile and `link` |
| Charcoal | `#292c32` | `Spec` on light backgrounds |
| Off-white | `#f5f5f5` | `Spec` on dark backgrounds |
| Light teal | `#4bb9b3` | Symbol and `link` on dark backgrounds |

Brand colors are fixed artwork values. UI tokens remain in `packages/ui/src/theme.css`.

## Files

`svg/` contains editable vector masters with outlined lettering, requiring no installed font. `transparent/` contains transparent PNG exports. Top-level PNGs have white backgrounds (dark backgrounds for `-dark` files). The system sheet is a presentation board with a background.

- `speclink-app-icon`: rounded app tile and white S.
- `speclink-logo-mark`: standalone teal S, no tile or text.
- `speclink-wordmark`: text only; charcoal Spec and teal link.
- `speclink-wordmark-dark`: text-only variant for dark backgrounds.
- `speclink-logo-horizontal`: symbol and wordmark.
- `speclink-logo-horizontal-dark`: horizontal dark-background variant.
- `speclink-logo-vertical`: symbol above wordmark.
- `speclink-logo-system-sheet`: lockups and small-size previews.

## App integration

Desktop and Server Web share three SVGs in `packages/ui/src/assets/`: `logo-mark.svg` (the bare mark, copied from `speclink-logo-mark.svg` with its fill changed to `currentColor`), `logo-horizontal.svg` and `logo-horizontal-dark.svg` (copied verbatim). `packages/ui/src/components/Brand.tsx` renders them as `BrandMark` (the mark follows the primary color) and `Wordmark` (the horizontal lockup, switching to the dark version under a dark system preference). To change the logo, replace those three files. The desktop favicon stays at `apps/desktop/public/logo-mark.png` (referenced by `index.html`). Native desktop icons live in `apps/desktop/src-tauri/icons/`, including ICO, ICNS and sized PNGs. The tray uses separate monochrome 18/36 px symbols; the 36 px PNG is also embedded in `apps/desktop/src/trayIcon.ts`.

To regenerate native icons:

```sh
npx tauri icon docs/assets/brand/transparent/speclink-app-icon.png -o /tmp/speclink-icons
```

Copy only the top-level desktop outputs into `apps/desktop/src-tauri/icons/`; exclude generated mobile assets. Regenerate the tray PNGs separately and synchronize their embedded base64 constant.
