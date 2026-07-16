# 02 – Architecture

## Stack

| Layer | Choice | Why |
|---|---|---|
| Desktop shell | **Tauri 2** | Small native Windows binary, first-class filesystem access, sidecar support for ffprobe, auto-updater |
| UI | **React 18 + TypeScript + Vite** | Fast iteration on a data-heavy UI |
| UI state/data | **TanStack Query** (server-state over Tauri commands) + **Zustand** (local UI state) | Query cache maps cleanly onto "invoke command, cache by key, invalidate on mutation" |
| Styling/components | **Tailwind CSS + shadcn/ui** | Dense, keyboard-friendly desktop UI without design overhead |
| Calendar UI | **FullCalendar** (or custom grid if licensing chafes) | Month/week grids + drag-drop are solved problems |
| Database | **SQLite** via `rusqlite` on the Rust side | Local-first, zero-ops, easily handles tens of thousands of reels; single file to back up |
| Media probing | **ffprobe** shipped as a Tauri sidecar binary | Duration/resolution/codec extraction + thumbnail generation (via ffmpeg `-ss ... -frames:v 1`) |
| File watching | Rust `notify` crate | Watch folders (Exports, Raw inbox) with debounced events |
| Hashing | `blake3` | Fast content hashing for dedup and file-move detection |
| HTTP | `reqwest` (Rust) | Metricool API client lives in Rust, not the webview |
| Secrets | Windows Credential Manager via `keyring` crate | Metricool API key never sits in SQLite or JSON |

## Process model

```
┌────────────────────────────────────────────────────┐
│ Tauri app (single process + webview)               │
│                                                    │
│  React UI (webview)                                │
│    │  invoke() commands / listen() events          │
│  Rust core                                         │
│    ├── db: rusqlite + migration runner             │
│    ├── media: ffprobe sidecar, thumbnails, blake3  │
│    ├── watcher: notify-based watch folder service  │
│    ├── davinci: handoff staging + FCPXML writer    │
│    ├── metricool: API client + sync jobs           │
│    └── jobs: background task queue (indexing,      │
│         thumbnailing, sync) with progress events   │
└────────────────────────────────────────────────────┘
```

Rules of the split:

- **All business logic and I/O in Rust.** The webview never touches the filesystem or network directly; it calls typed commands (`list_reels`, `create_batch`, `generate_handoff`, `push_schedule`, …).
- **Long work goes through the job queue.** Indexing a folder of 500 clips, thumbnailing, or a Metricool sync runs as a background job emitting `job://progress` events; the UI shows a jobs tray. Nothing blocks the interface.
- **Events push state changes.** The watcher emits `asset://discovered`, the export matcher emits `reel://final-matched`, sync emits `post://status-changed`; TanStack Query invalidates on these.

## Storage roots — the PC → Unraid NAS migration plan

The single most important storage decision: **the DB never stores absolute paths.** Every file reference is `(root_id, relative_path)`.

- A **storage root** is a named mount: `{ id, name, path, kind: local|nas, online }` — e.g. root `media` = `D:\ContentOS` today, `\\unraid\content` (or a mapped drive letter) next year.
- Moving to the NAS = copy the tree, edit one row's `path`. Zero relinking.
- Each root gets a **marker file** (`.contentos-root` containing the root's UUID) so the app can detect a root that moved and offer to re-point it automatically.
- Roots have an **online/offline state**. When the NAS is unreachable the Library shows cached metadata + thumbnails (thumbnails live in local app data, not on the root) and every file action degrades gracefully instead of erroring.
- Content hashes make file *moves within a root* self-healing: if a path 404s but a scan finds the same blake3 hash elsewhere, the app offers (or auto-applies, per setting) a relink.

App-local data (always on `C:`, in `%APPDATA%/ContentOS`):

```
%APPDATA%/ContentOS/
  contentos.db          ← SQLite (WAL mode)
  thumbnails/{asset-id}.jpg
  logs/
  backups/              ← daily zipped DB snapshots, keep 30
```

## Background jobs

A tiny persistent job table + in-process worker (no external queue). Job kinds:

- `index_folder` — walk a folder, ffprobe new files, hash, insert assets
- `thumbnail` — batch thumbnail generation
- `match_exports` — reconcile Exports watch folder against reels awaiting finals
- `metricool_push` — create/update scheduled posts
- `metricool_sync` — poll post statuses + pull metrics (every 30 min while app runs)
- `verify_files` — periodic existence/hash spot-check of linked assets

Jobs are idempotent and resumable; the queue survives app restarts.

## Error philosophy

At 100/week, silent failure is the enemy. Every job failure lands in a visible **Problems** feed on the Dashboard (file missing, export unmatched, Metricool push rejected, watch folder offline) with a one-click action where possible. The app never swallows an error into a log file the user won't read.

## Packaging & updates

- MSI/NSIS installer via Tauri bundler; auto-update from GitHub Releases.
- ffprobe/ffmpeg shipped as sidecars (licensing: use LGPL builds).
- Single-instance lock (two instances writing one SQLite file is asking for pain).

## Future-proofing hooks (explicitly cheap now, valuable later)

- **AI phase:** an `ai_generations` table and a Rust `ai` module boundary already reserved; script editor UI leaves a slot for a "variations" panel.
- **NAS phase:** storage roots as above; nothing else changes.
- **Team phase (maybe never):** all mutations already flow through Rust commands, so adding a sync layer later doesn't require rewriting the UI.
