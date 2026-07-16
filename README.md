# ContentOS

A Windows desktop app for planning, producing, and publishing short-form video at industrial scale — built for a workflow of **100 reels per week** across Instagram Reels, TikTok, and YouTube Shorts.

ContentOS is the operating system for the whole pipeline:

```
Idea → Script → Shot list → Batch shoot → DaVinci edit → Final render → Schedule → Verify posted
```

The app plans and tracks everything, organizes every file on disk, generates ready-to-edit DaVinci Resolve timelines, and pushes finished reels to Metricool for scheduling — then verifies they actually went out.

## Status

**Phase 4 — publishing.** The full pipeline is live end to end: queue edited reels into per-platform posts (IG/TikTok/Shorts), write captions + hashtags once per reel, bulk-fill schedule times from calendar dates, then export a Metricool bundle — a bulk-import CSV plus the finals hardlinked and named by slot (2026-07-20_0900_IG_R0142.mp4) for the free-plan manual flow. A tracking board walks posts through scheduled → published → verified (with failure lanes), and reel status follows automatically to `verified`. The Metricool API provider drops in behind the same interface when the plan upgrade lands ([roadmap](docs/07-roadmap.md)).

## Development

Prerequisites on Windows: [Node 20+](https://nodejs.org), [Rust](https://rustup.rs), and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) (WebView2 is preinstalled on Windows 11).

```powershell
npm install
npm run tauri dev      # run the desktop app (hot-reloading UI)
npm run tauri build    # produce the Windows installer (MSI/NSIS)

npm run build          # frontend type-check + bundle only
cargo test -p contentos-core   # core (database/logic) test suite
npm run icons          # regenerate placeholder app icons
```

Layout: `src/` React UI · `src-tauri/` Tauri shell (thin command layer) · `crates/core/` `contentos-core`, the OS-independent brain — database, migrations, storage roots, jobs — fully unit-tested without the GUI.

## Design documents

| Doc | Contents |
|---|---|
| [01 – Product Overview](docs/01-product-overview.md) | Vision, the 100/week workflow, throughput math, screen map |
| [02 – Architecture](docs/02-architecture.md) | Tauri + React + SQLite stack, process model, storage roots (PC → NAS migration) |
| [03 – Data Model](docs/03-data-model.md) | Every entity + full SQLite schema |
| [04 – File System](docs/04-file-system.md) | Folder layout, naming conventions, watch folders, integrity |
| [05 – DaVinci Integration](docs/05-davinci-integration.md) | Batch handoff folders, generated FCPXML timelines, export auto-matching |
| [06 – Metricool Publishing](docs/06-metricool-publishing.md) | Push scheduling, posted-verification, metrics, CSV fallback |
| [07 – Roadmap](docs/07-roadmap.md) | Build phases from empty repo to full pipeline |
| [08 – Design System](docs/08-design-system.md) | Fieldhouse Green tokens (dark-first), type, UI rules — [live preview](docs/style-preview.html) |

## Core decisions (locked)

- **Single user, local-first.** No accounts, no cloud backend. SQLite on disk. Data is yours.
- **Tauri 2 + React + TypeScript** desktop shell; Rust core for file watching, hashing, ffprobe, FCPXML generation, and the Metricool client.
- **The app manages files, DaVinci edits them.** No built-in video editor. ContentOS indexes and links media (never silently copies your originals), stages organized handoff folders, and generates pre-assembled timelines so each edit starts ~80% done.
- **Reels can be modular or standalone.** Modular reels compose reusable hooks / bodies / CTAs; standalone reels are one script shot straight through. One data model covers both.
- **Metricool is the publishing rail.** ContentOS pushes scheduled posts through the Metricool API and reads back posted status + metrics. A CSV bulk-export fallback exists for plans without API access.
- **Storage-root abstraction from day one** so moving the library from the local PC to the Unraid NAS is a re-point, not a migration.
