# 07 – Roadmap

Phases are ordered so that **every phase ends with something you use in production that week**. Estimates assume focused solo/pair development with AI assistance.

## Phase 0 — Foundation (week 1)

- Tauri 2 + React + TS + Tailwind scaffold; CI (build + test on Windows).
- SQLite with migration runner; full schema from [doc 03](03-data-model.md) lands migration-first.
- Storage roots + settings + `.contentos-root` markers.
- Job queue skeleton + Problems feed + activity log.
- ffprobe sidecar wired; hash + thumbnail pipeline proven on a test folder.

**Exit:** app opens, points at `D:\ContentOS`, indexes a folder of clips with thumbnails.

## Phase 1 — Plan: reels, scripts, calendar (weeks 2–3)

- Pillars, ideas backlog, reel CRUD with codes.
- Block-based script editor; component library (hooks/bodies/CTAs) with reuse + usage counts; FTS search.
- Calendar (month/week), drag reels to target dates, pillar colors, capacity counts vs. weekly target.
- Pipeline (Kanban) board driven by reel status.

**Exit:** the next real content week is planned entirely in ContentOS.

## Phase 2 — Shoot: batches, shot lists, ingest (weeks 4–5)

- Build shoot batches from scripted reels; **shot-list deduplication** of shared components; setup-tag grouping and ordering.
- **Record Mode:** full-screen teleprompter checklist, take logging, keyboard-only.
- `00_INBOX` watcher; triage view (clips × shot list side-by-side linking); canonical rename into `01_RAW`; take selection with ratings.
- Reels auto-flip `shotlisted → shot`.

**Exit:** one real shoot day (50+ clips) runs through the app end to end.

## Phase 3 — Edit: DaVinci handoff (weeks 6–7)

- Handoff staging generator (hardlink folders, batch sheet, script txts).
- **FCPXML timeline generation** + golden-file tests + Resolve smoke-test doc.
- `03_EXPORTS` watcher, export matching, `04_FINALS` filing, preflight checks.
- Assemble screen (generate/regenerate, match review).

**Exit:** an editing day starts from imported timelines and ends with finals auto-matched. This phase is the biggest single time-saver in the system.

## Phase 4 — Publish: Metricool (weeks 8–9)

- **Spike first:** confirm API tier/capabilities ([doc 06](06-metricool-publishing.md)); decide API vs CSV as primary.
- Captions/hashtags editor + templates; schedule defaults + bulk-fill.
- Push, sync, verification board, failure escalation; metrics snapshots.
- CSV fallback exporter regardless of spike outcome.

**Exit:** a full week (~100 reels / up to 300 posts) scheduled and verified from inside the app.

## Phase 5 — Scale & polish (weeks 10–12)

- Dashboard v2: funnel, stuck-item alerts, schedule-fill, disk watermark, daily digest.
- Retention/archive rules; DB backup automation; performance pass with 5k+ reels seeded.
- **Unraid NAS support hardening:** offline-root behavior, scratch staging root for edits, re-point rehearsal on real hardware.
- Keyboard-shortcut coverage + command palette (Ctrl+K).

## Phase 6 — AI layer (later, by design)

Schema (`ai_generations`) and module seams already exist. Candidates, in payoff order:

1. Hook variation generator ("10 hooks for body BD0007 in my voice" — trained on the component library's own top performers using metric snapshots).
2. Caption + hashtag drafting per platform from the script text.
3. Script drafting from an idea + pillar.
4. Performance insights ("hooks mentioning numbers outperform by 32%").

## Risks worth naming now

| Risk | Mitigation |
|---|---|
| Metricool API doesn't cover video scheduling on our tier | Phase-4 spike before build; CSV fallback is a first-class feature |
| Resolve FCPXML import quirks across versions | Golden-file tests + pinned verified-versions doc; scripting-API backend possible behind the same seam |
| Raw footage outgrows local disk before NAS arrives | Disk watermark on dashboard from Phase 0; retention rules in Phase 5 |
| Solo-user data loss | WAL + daily zipped DB backups from Phase 0; media is never deleted automatically, ever |
