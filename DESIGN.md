# Design

Visual system for ContentOS — "Fieldhouse Green". Canonical tokens live in `src/styles/tokens.css` (dark on `:root` = default theme; light under `[data-theme="light"]`); shared component styles in `src/styles/app.css`. The narrative spec is `docs/08-design-system.md`; this file is the machine-readable summary every design task reads first.

## Theme

Dark-first. The app is used in long editing/planning sessions, often beside DaVinci Resolve (a dark tool) in a dim studio — dark is the working default, light is a Settings option. Both themes are token swaps only; components never branch on theme.

## Colors

| Role | Dark (default) | Light |
|---|---|---|
| Background `--bg` | `#10140F` | `#EFF1E8` |
| Panel `--panel` | `#181E17` | `#FBFBF7` |
| Nested surface `--panel-2` | `#1E251D` | `#F3F5EC` |
| Ink `--ink` | `#E7ECE2` | `#1C241C` |
| Muted `--muted` | `#93A08F` | `#5C685C` |
| Border `--line` | `#2A332A` | `#D6DCCB` |
| Accent `--accent` | `#7CC08A` (leaf) | `#2F6B4F` (pine) |
| Text on accent `--accent-ink` | `#10140F` | `#FFFFFF` |
| Selection tint `--accent-soft` | `#24352A` | `#DFEAD9` |
| Good | `#7CC08A` | `#3C8A4E` |
| Warn | `#D9B35E` | `#B4831F` |
| Bad | `#E0796A` | `#C24936` |
| Soft state tints | `--good-soft` `--warn-soft` `--bad-soft` | (see tokens.css) |
| Pillar slots | moss / clay / ochre / teal (light tints, dark text) | solid earth tones, white text |

Rules: strategy is **Restrained** — neutrals carry the surface, accent ≤10%. One accent moment per screen (the primary action). Semantic state colors are never the accent. Pillar colors appear only on calendar chips, dots, and board markers.

## Typography

Single family: `"Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif` (native on Windows — the platform's own voice). Mono: `"Cascadia Mono", Consolas, ui-monospace, monospace` for codes (`R0142`), filenames, timecodes, tabular data. Fixed rem scale, ratio ~1.2: body 13px, section labels 11px uppercase +0.06em, screen titles 18px/650, tile values 22px/650. `font-variant-numeric: tabular-nums` wherever digits align. No display fonts anywhere.

## Layout

Desktop app shell: fixed 200px sidebar (panel surface) + scrollable main column, 14px gaps. Density first — tables and lists over cards; the repeated primitive is the **bordered panel** (`--panel`, 1px `--line`, 8px radius) with an uppercase label header. Chips are 4px radius. Content grids use `repeat(auto-fit, minmax(...))`.

## Components

Established vocabulary (keep consistent; extend, don't reinvent):

- **`.panel`** — the layout primitive: surface + border + uppercase `h2` label.
- **`.tile`** — dashboard stat: label + tabular number (alert variant colors value `--bad`).
- **`.code`** — mono chip on `--accent-soft`; every reel/batch/component code renders this way.
- **`.pill`** — status: `good` / `warn` / `bad` / `brand` soft-tinted, text carries meaning.
- **`.btn`** — default panel-toned; `primary` = accent (one per screen); `danger` = red text. States: hover (surface shift), focus-visible (2px accent outline), disabled (50% opacity).
- **`.stage`** — pipeline funnel cell; hot state uses accent tint.
- **`table.list`** — dense data table with uppercase column labels.
- **`.ev`** — calendar chip in pillar color, grab cursor when draggable.
- Inputs/selects/textareas — `--panel-2` surface, 1px `--line`, same focus ring as buttons.

## Motion

150–250ms, ease-out only, transform/opacity (background-color on hover is fine). Motion conveys state — hover, press, panel/route transitions, skeleton shimmer. No page-load choreography, no scroll reveals, no decorative animation. `prefers-reduced-motion: reduce` → instant transitions.

## Voice

Labels name the user's task, not the system's mechanism ("Push week to Metricool", not "Sync API"). Empty states are directions, not verdicts ("Drag onto a day to plan it"), errors say what happened and the next action. Sentence case everywhere except the small uppercase section labels.
