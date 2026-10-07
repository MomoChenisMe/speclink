# Speclink brand assets

[繁體中文](README.md) · **English**

This folder holds the source files of the Speclink logo and icons. The mark is a document outline with circuit nodes: a spec is a document, and the nodes and traces are the links and flows between specs.

## Colors

| Role | Value | Use |
| --- | --- | --- |
| Ink (navy) | Dark navy | Document outline, the `Spec` text, main lines |
| Teal (primary) | `oklch(0.52 0.1 192)` | Circuit nodes, traces, the `link` text, the app primary color (`--primary`) |

For the full color tokens of the app, see `apps/desktop/src/index.css` (Tailwind v4 tokens, primary hue 192).

## Files

The official assets are in this folder. Each lockup has a version with a solid white background and a transparent version:

| File | Description | Recommended use |
| --- | --- | --- |
| `speclink-logo-horizontal.png` | Horizontal lockup (mark and text side by side) | README hero, site header, document cover |
| `speclink-logo-vertical.png` | Vertical lockup (mark above, text below) | Square layouts, social avatars, splash screens |
| `speclink-logo-mark.png` | Mark only, no text | App and window icons, favicon, small sizes |
| `speclink-wordmark.png` | Text only, no mark | A header lockup next to the mark, or when you need text without the icon |
| `speclink-logo-system-sheet.png` | All three lockups on one sheet | Proposals and slides |
| `transparent/` | Transparent versions of the files above | On a background that is not white |

The desktop app uses two files from `transparent/`:

- `speclink-logo-mark.png` is the source of `apps/desktop/src-tauri/icons/` (through `tauri icon`, for window and taskbar icons) and of `apps/desktop/public/logo-mark.png` (header icon and favicon).
- `speclink-wordmark.png`, cropped to tight bounds, becomes `apps/desktop/public/speclink-wordmark.png` (the header text next to the mark).

## Dark backgrounds

The official lockups use navy text and are for light backgrounds. On a dark background, use the version with the solid white background so that the logo stays readable. No dark-mode variant exists yet.
