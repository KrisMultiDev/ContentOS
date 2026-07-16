# 08 – Design System: Fieldhouse Green

Approved direction (see the live preview in [`style-preview.html`](style-preview.html)). **Dark theme is the app default**; light remains available in Settings.

## Tokens

| Token | Dark (default) | Light |
|---|---|---|
| `--bg` (app ground) | `#10140F` | `#EFF1E8` |
| `--panel` | `#181E17` | `#FBFBF7` |
| `--panel-2` (nested surfaces) | `#1E251D` | `#F3F5EC` |
| `--ink` | `#E7ECE2` | `#1C241C` |
| `--muted` | `#93A08F` | `#5C685C` |
| `--line` (borders) | `#2A332A` | `#D6DCCB` |
| `--accent` (leaf / pine) | `#7CC08A` | `#2F6B4F` |
| `--accent-ink` (text on accent) | `#10140F` | `#FFFFFF` |
| `--accent-soft` (selection/hover tint) | `#24352A` | `#DFEAD9` |
| `--good` | `#7CC08A` | `#3C8A4E` |
| `--warn` | `#D9B35E` | `#B4831F` |
| `--bad` | `#E0796A` | `#C24936` |
| `--good-soft` / `--warn-soft` / `--bad-soft` | `#22301F` / `#322A18` / `#38211D` | `#DFEEDB` / `#F3E8CC` / `#F6DDD7` |

Pillar colors (calendar/boards) — dark theme uses light tints with dark text, light theme uses solid earth tones with white text:

| Pillar slot | Dark | Light |
|---|---|---|
| Moss | `#8AB894` | `#4A7856` |
| Clay | `#D19071` | `#A9603E` |
| Ochre | `#CBAF63` | `#9A7B2D` |
| Teal | `#7FAEB5` | `#3E6E76` |

## Rules

1. **One accent moment per screen.** The leaf-green accent goes to the single primary action (e.g. "Push week to Metricool"); everything else sits on panel tones. Never use the accent for decoration.
2. **Semantic ≠ brand.** Verified/published use `--good`, warnings `--warn`, failures `--bad` — never the accent — so state colors and brand identity stay legible independently.
3. **Reel codes are mono chips.** `R0142`, `B012`, `HK0031` always render in Cascadia Mono inside an `--accent-soft` chip. They are the visual thread between app, filenames, and DaVinci.
4. **Type:** Segoe UI Variable / Segoe UI for all UI text; Cascadia Mono (fallback Consolas) for codes, filenames, timecodes, and any tabular data. `font-variant-numeric: tabular-nums` wherever digits align.
5. **Geometry:** 6–8 px radii on panels, 4 px on chips. Density first — this is an operations tool, not a marketing site.
6. **Theme implementation:** all components style through CSS custom properties only; themes are token swaps at `:root`. No per-component theme conditionals.

## Implementation note

These tokens live in `src/styles/tokens.css` (dark on `:root`, light under `[data-theme="light"]`), with shared component styles in `src/styles/app.css`. Every screen from Phase 1 onward inherits the approved look with zero restyling later. Tailwind/shadcn onboarding is deferred to when the dense Phase 1+ screens need it — the tokens are the contract either way.
