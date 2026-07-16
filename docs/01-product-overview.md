# 01 – Product Overview

## Vision

One person producing 100 reels a week is not an editing problem — it's a **logistics problem**. The bottleneck is never "make one video"; it's keeping hundreds of scripts, clips, edits, captions, and schedule slots from collapsing into chaos. ContentOS is the logistics layer: a single desktop app where every reel has a state, every file has a home, and every day has a plan.

## The user

- Solo creator on a Windows PC (later: media library on an Unraid NAS).
- Shoots in batches (many hooks/scripts in one recording session).
- Edits in DaVinci Resolve.
- Schedules through Metricool to Instagram Reels, TikTok, and YouTube Shorts.

## The pipeline

Every reel moves through a fixed lifecycle. The app is organized around this state machine:

```
IDEA → SCRIPTED → SHOTLISTED → SHOT → ASSEMBLED → EDITING → EDITED → SCHEDULED → POSTED → VERIFIED
                                                                          ↘ FAILED (retry)
   (any state) → ARCHIVED / KILLED
```

| State | Meaning | What moves it forward |
|---|---|---|
| `idea` | Captured in the idea backlog | Writing a script |
| `scripted` | Script finished (blocks defined) | Adding its shots to a shoot batch |
| `shotlisted` | On a shot list for an upcoming batch | Recording day: all shots marked recorded + takes selected |
| `shot` | All required clips exist and takes are picked | Generating the DaVinci handoff |
| `assembled` | Handoff folder + FCPXML timeline generated | Opening it in Resolve |
| `editing` | Being edited in DaVinci | Final render lands in the Exports watch folder |
| `edited` | Final file matched and verified | Assigning a calendar slot + captions |
| `scheduled` | Pushed to Metricool with date/time per platform | Metricool publishes it |
| `posted` | Metricool reports published | Verification sync confirms live URL |
| `verified` | Confirmed live on platform(s) | Metrics collection (ongoing) |

## Reel anatomy: modular and standalone

Two kinds of reels, one model:

- **Modular** — a reel is a composition of reusable components: `HOOK + BODY + CTA`. One strong body gets 5 different hooks → 5 reels from one core shoot. Components live in a library, track usage counts, and can be recombined endlessly.
- **Standalone** — one script, shot start to finish, one reel.

Internally both are "a reel with an ordered list of script blocks"; for modular reels the blocks point at shared library components, for standalone reels the blocks are private to that reel. (See [Data Model](03-data-model.md).)

## Throughput math (why every feature exists)

100 reels/week, sustained:

- **Writing:** if ~60% are modular (e.g. 12 bodies × 5 hooks) and 40% standalone, a week needs roughly **12 bodies + 60 hooks + a few CTAs + 40 standalone scripts**. Nobody writes that in one-at-a-time forms → the script editor is built for rapid-fire batch writing and hook variation lists.
- **Shooting:** 2–3 recording sessions/week, each covering 50+ clips. A 50-clip session dies without a ruthless shot list → recording-day mode is a full-screen teleprompter/checklist that orders shots to minimize outfit/location/setup changes and marks takes in one keystroke.
- **Editing:** ~15–20 finals per editing day. Hunting for files kills this → the DaVinci handoff stages everything pre-named, and generated timelines mean an edit starts with clips already in order on the timeline.
- **Publishing:** 100 reels × up to 3 platforms = up to **300 posts/week (~43/day)**. Manual scheduling is untenable → calendar bulk-fill, caption templates per platform, one-click "push week to Metricool".
- **Tracking:** at this volume you *will* lose reels in the cracks unless the dashboard shouts about them → pipeline counters and "stuck item" alerts (e.g. `shot` for >4 days, schedule gaps this week).

## Screen map

| Screen | Purpose |
|---|---|
| **Dashboard** | Pipeline funnel counts, this week's schedule fill vs. 100-target, stuck items, storage/watch-folder health, Metricool sync status |
| **Calendar** | Month/week grid of slots; drag reels onto days; platform lanes; pillar color coding; capacity warnings; bulk auto-fill from the `edited` queue |
| **Ideas** | Fast-capture backlog with pillar tags; promote idea → reel |
| **Scripts** | Block-based script editor (hook/body/CTA or full); component library of reusable hooks/bodies/CTAs with usage counts; batch "write 10 hooks for this body" view |
| **Shoot** | Build shoot batches from scripted reels; auto-generated shot lists grouped/ordered by setup; **Record Mode**: full-screen teleprompter + take logger |
| **Library** | Every indexed media file: thumbnails, duration, resolution, link state (which shot/reel it belongs to), orphans, watch-folder inbox triage |
| **Assemble** | Reels ready for edit; generate/regenerate DaVinci handoff (folders + FCPXML); export-matching review (confirm auto-matched finals) |
| **Publish** | Caption + hashtag editor per platform (with templates), schedule queue, push to Metricool, posted/verified status board, failures needing retry |
| **Settings** | Storage roots, watch folders, naming rules, Metricool credentials, pillars, platform defaults, backup |

## Non-goals (v1)

- No video editing/trimming/rendering inside the app (DaVinci owns that; we don't even auto-concat — the generated timeline does the assembly *in* Resolve).
- No multi-user/collaboration. Schema keeps `created_by`-style columns out until needed.
- No direct platform APIs (IG/TikTok/YT) — Metricool is the only publishing integration.
- No AI features in v1 — but the schema reserves space for them (see roadmap phase 6).
