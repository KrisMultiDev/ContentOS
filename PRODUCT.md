# Product

## Register

product

## Platform

web

## Users

A solo short-form content creator on Windows producing ~100 reels per week for Instagram Reels, TikTok, and YouTube Shorts. They are always mid-task — planning a week, writing scripts in bulk, running a 50-clip recording session, triaging footage after a shoot, or scheduling a batch of posts. Sessions are long and keyboard-heavy; the app is a daily production tool, not an occasional visitor. Later, media lives on an Unraid NAS, but the user stays the same one person.

## Product Purpose

ContentOS is the operating system for a high-volume reels pipeline: idea → script → shot list → batch shoot → DaVinci edit → final render → schedule → verify posted. It exists because 100 reels/week is a logistics problem, not an editing problem — the product tracks every reel's state, gives every file a home, generates ready-to-edit DaVinci timelines, and pushes finished posts through Metricool. Success = a full week (100 reels, up to 300 posts) planned, produced, and verified from inside the app without anything falling through the cracks.

## Positioning

The only place where every reel, clip, script, and schedule slot has exactly one state — and nothing gets lost between shooting hundreds of clips and verifying hundreds of posts.

## Brand Personality

Calm, dependable, industrial. A production binder in software form: dense, keyboard-first, quietly confident. The interface should disappear into the task; the only loud thing on any screen is the one action that moves work forward and the problems that need attention.

## Anti-references

- Generic AI-generated SaaS: purple/indigo gradients, glassmorphism, hero-metric tiles, icon-card grids.
- Social-media-manager dashboards that decorate with brand colors of the platforms.
- Consumer creator apps that trade density for whitespace; this user needs 50 rows on screen, not 5 cards.

## Design Principles

1. **State is the interface.** Every screen exists to move reels to the next pipeline state or to shout when one is stuck. If a view doesn't answer "what's next?" it doesn't ship.
2. **Codes are the thread.** Reel/batch/component codes (R0142, B012, HK0031) appear identically in the UI, filenames, and DaVinci — always in mono chips. Never hide them.
3. **One accent moment per screen.** The leaf-green accent goes to the single primary action; semantic good/warn/bad colors never reuse the brand green.
4. **Keyboard-first at volume.** Anything done 100× a week (triage, take rating, captioning) must work without the mouse.
5. **No silent failure.** Errors, unmatched files, and skipped work land in a visible Problems feed with a next action — never only in a log.

## Accessibility & Inclusion

Dark theme is the default; both themes must hold WCAG AA contrast (body ≥4.5:1). Full keyboard operability with visible focus states (≥3:1). Respect `prefers-reduced-motion` — state changes fall back to instant/crossfade. Color is never the only carrier of state (pills carry text, not just hue).
