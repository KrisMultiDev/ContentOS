# 03 – Data Model

## Entity overview

```
Pillar ──< Idea ──promote──> Reel ──< ScriptBlock >── Component (reusable hook/body/CTA)
                              │
             ShootBatch ──< Shot >── (reel, block)
                              │         └──< Take >── Asset
                              │
                              ├── final_asset ── Asset ──> StorageRoot
                              ├──< Post (per platform) ──< MetricSnapshot
                              └── CalendarSlot (target date)
```

Key ideas:

- **`reels` is the spine.** Everything hangs off a reel; its `status` column drives every board and dashboard count.
- **Modular vs standalone is not a fork in the schema.** Every reel has ordered `script_blocks`. A block either embeds its own text (standalone) or references a shared `components` row (modular). A reel mixing both is legal.
- **Files are `assets`,** always `(root_id, rel_path)`, never absolute paths (see [Architecture](02-architecture.md)).
- **A reel is one piece of content; a `post` is one platform publication.** One reel → up to 3 posts (IG/TikTok/YT), each with its own caption, schedule time, Metricool id, and status.

## Human-readable codes

Every reel gets an immutable code `R0001, R0002, …` (batches: `B001…`, components: `HK0001/BD0001/CT0001`). Codes appear in **every filename** the app generates — they are the join key between the filesystem, DaVinci, and the DB. Slugs can change; codes never do.

## SQLite schema

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ── Configuration ────────────────────────────────────────────────

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL              -- JSON
);

CREATE TABLE storage_roots (
  id         TEXT PRIMARY KEY,     -- uuid
  name       TEXT NOT NULL UNIQUE, -- 'media'
  path       TEXT NOT NULL,        -- 'D:\ContentOS'  → later '\\unraid\content'
  kind       TEXT NOT NULL CHECK (kind IN ('local','nas')),
  online     INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
);

CREATE TABLE pillars (
  id              TEXT PRIMARY KEY,
  name            TEXT NOT NULL UNIQUE,
  color           TEXT NOT NULL,          -- hex, used across calendar/boards
  description     TEXT,
  target_per_week INTEGER NOT NULL DEFAULT 0,
  sort_order      INTEGER NOT NULL DEFAULT 0,
  archived        INTEGER NOT NULL DEFAULT 0
);

-- ── Ideation ─────────────────────────────────────────────────────

CREATE TABLE ideas (
  id         TEXT PRIMARY KEY,
  title      TEXT NOT NULL,
  notes      TEXT,
  pillar_id  TEXT REFERENCES pillars(id),
  status     TEXT NOT NULL DEFAULT 'open'
             CHECK (status IN ('open','promoted','killed')),
  reel_id    TEXT REFERENCES reels(id),   -- set when promoted
  created_at TEXT NOT NULL
);

-- ── Reels & scripts ──────────────────────────────────────────────

CREATE TABLE reels (
  id             TEXT PRIMARY KEY,
  code           TEXT NOT NULL UNIQUE,        -- 'R0142'
  title          TEXT NOT NULL,
  slug           TEXT NOT NULL,               -- 'gym-myths'
  pillar_id      TEXT REFERENCES pillars(id),
  status         TEXT NOT NULL DEFAULT 'idea' CHECK (status IN
                 ('idea','scripted','shotlisted','shot','assembled',
                  'editing','edited','scheduled','posted','verified',
                  'archived','killed')),
  notes          TEXT,
  target_date    TEXT,                        -- calendar slot (date, nullable)
  final_asset_id TEXT REFERENCES assets(id),  -- matched final render
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL
);
CREATE INDEX idx_reels_status ON reels(status);
CREATE INDEX idx_reels_target ON reels(target_date);

-- Reusable library components (modular hooks / bodies / CTAs)
CREATE TABLE components (
  id             TEXT PRIMARY KEY,
  code           TEXT NOT NULL UNIQUE,        -- 'HK0031'
  kind           TEXT NOT NULL CHECK (kind IN ('hook','body','cta')),
  text           TEXT NOT NULL,               -- the script text
  tags           TEXT,                        -- JSON array
  pillar_id      TEXT REFERENCES pillars(id),
  -- a component can have its own canonical recorded clip, reusable across reels:
  master_take_id TEXT REFERENCES takes(id),
  archived       INTEGER NOT NULL DEFAULT 0,
  created_at     TEXT NOT NULL
);

-- Ordered blocks that make up a reel's script.
-- Standalone reel: blocks carry their own text (component_id NULL).
-- Modular reel:    blocks reference components (text NULL, read from component).
CREATE TABLE script_blocks (
  id           TEXT PRIMARY KEY,
  reel_id      TEXT NOT NULL REFERENCES reels(id) ON DELETE CASCADE,
  position     INTEGER NOT NULL,
  kind         TEXT NOT NULL CHECK (kind IN ('hook','body','cta','segment')),
  component_id TEXT REFERENCES components(id),
  text         TEXT,                          -- used when component_id IS NULL
  est_seconds  INTEGER,
  CHECK (component_id IS NOT NULL OR text IS NOT NULL),
  UNIQUE (reel_id, position)
);

-- ── Shooting ─────────────────────────────────────────────────────

CREATE TABLE shoot_batches (
  id         TEXT PRIMARY KEY,
  code       TEXT NOT NULL UNIQUE,            -- 'B012'
  name       TEXT NOT NULL,
  shoot_date TEXT,
  status     TEXT NOT NULL DEFAULT 'planning'
             CHECK (status IN ('planning','ready','shooting','done')),
  notes      TEXT,
  created_at TEXT NOT NULL
);

-- One shot = one script block to record within a batch.
-- Deduplication: if 5 reels share body BD0007, the batch gets ONE shot for it.
CREATE TABLE shots (
  id               TEXT PRIMARY KEY,
  batch_id         TEXT NOT NULL REFERENCES shoot_batches(id) ON DELETE CASCADE,
  position         INTEGER NOT NULL,            -- shoot order (grouped by setup)
  component_id     TEXT REFERENCES components(id),
  script_block_id  TEXT REFERENCES script_blocks(id),
  setup_tags       TEXT,                        -- JSON: outfit/location/props
  status           TEXT NOT NULL DEFAULT 'pending'
                   CHECK (status IN ('pending','recorded','skipped')),
  selected_take_id TEXT REFERENCES takes(id),
  CHECK (component_id IS NOT NULL OR script_block_id IS NOT NULL)
);
CREATE INDEX idx_shots_batch ON shots(batch_id, position);

CREATE TABLE takes (
  id          TEXT PRIMARY KEY,
  shot_id     TEXT NOT NULL REFERENCES shots(id) ON DELETE CASCADE,
  asset_id    TEXT NOT NULL REFERENCES assets(id),
  take_number INTEGER NOT NULL,
  rating      INTEGER,                          -- 1..5
  notes       TEXT
);

-- ── Media assets ─────────────────────────────────────────────────

CREATE TABLE assets (
  id          TEXT PRIMARY KEY,
  root_id     TEXT NOT NULL REFERENCES storage_roots(id),
  rel_path    TEXT NOT NULL,                    -- forward-slash, root-relative
  filename    TEXT NOT NULL,
  kind        TEXT NOT NULL CHECK (kind IN ('raw','final','other')),
  size_bytes  INTEGER NOT NULL,
  blake3      TEXT,
  duration_ms INTEGER,
  width       INTEGER,
  height      INTEGER,
  fps         REAL,
  codec       TEXT,
  recorded_at TEXT,                             -- from file metadata
  imported_at TEXT NOT NULL,
  missing     INTEGER NOT NULL DEFAULT 0,       -- set by verify_files job
  UNIQUE (root_id, rel_path)
);
CREATE INDEX idx_assets_hash ON assets(blake3);

-- ── Publishing ───────────────────────────────────────────────────

CREATE TABLE posts (
  id                TEXT PRIMARY KEY,
  reel_id           TEXT NOT NULL REFERENCES reels(id) ON DELETE CASCADE,
  platform          TEXT NOT NULL CHECK (platform IN ('instagram','tiktok','youtube')),
  caption           TEXT,
  hashtags          TEXT,                       -- JSON array
  scheduled_at      TEXT,                       -- ISO datetime, local tz stored w/ offset
  status            TEXT NOT NULL DEFAULT 'draft' CHECK (status IN
                    ('draft','queued','pushed','scheduled','published',
                     'verified','failed','canceled')),
  metricool_post_id TEXT,
  published_url     TEXT,
  error             TEXT,                       -- last failure message
  pushed_at         TEXT,
  published_at      TEXT,
  verified_at       TEXT,
  UNIQUE (reel_id, platform)
);
CREATE INDEX idx_posts_sched  ON posts(scheduled_at);
CREATE INDEX idx_posts_status ON posts(status);

CREATE TABLE metric_snapshots (
  id         TEXT PRIMARY KEY,
  post_id    TEXT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
  fetched_at TEXT NOT NULL,
  views      INTEGER, likes INTEGER, comments INTEGER,
  shares     INTEGER, saves INTEGER,
  raw        TEXT                               -- full JSON from Metricool
);

CREATE TABLE caption_templates (
  id       TEXT PRIMARY KEY,
  name     TEXT NOT NULL,
  platform TEXT CHECK (platform IN ('instagram','tiktok','youtube')), -- NULL = any
  body     TEXT NOT NULL              -- supports {title} {pillar} {hashtags} vars
);

-- ── Ops ──────────────────────────────────────────────────────────

CREATE TABLE jobs (
  id          TEXT PRIMARY KEY,
  kind        TEXT NOT NULL,
  payload     TEXT NOT NULL,                    -- JSON
  status      TEXT NOT NULL DEFAULT 'queued'
              CHECK (status IN ('queued','running','done','failed')),
  attempts    INTEGER NOT NULL DEFAULT 0,
  error       TEXT,
  created_at  TEXT NOT NULL,
  finished_at TEXT
);

CREATE TABLE activity_log (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  at         TEXT NOT NULL,
  entity     TEXT NOT NULL,                     -- 'reel','post','asset',…
  entity_id  TEXT NOT NULL,
  action     TEXT NOT NULL,                     -- 'status:shot→assembled', …
  detail     TEXT                               -- JSON
);

-- ── Reserved for AI phase (created now, unused in v1) ────────────

CREATE TABLE ai_generations (
  id         TEXT PRIMARY KEY,
  kind       TEXT NOT NULL,                     -- 'hooks','script','caption'
  input      TEXT NOT NULL,                     -- JSON prompt context
  output     TEXT NOT NULL,
  model      TEXT,
  accepted   INTEGER NOT NULL DEFAULT 0,
  entity     TEXT, entity_id TEXT,
  created_at TEXT NOT NULL
);
```

## Notable modeling decisions

**Component reuse and recording.** A modular component records **once** — `components.master_take_id` points at its canonical selected take. When 5 reels share body `BD0007`, their handoff timelines all reference the same clip file. If you deliberately re-record a hook for a specific reel, that shot links via `script_block_id` instead, overriding the master take for that reel only.

**Shot-list deduplication is the core batching win.** Building a batch from 30 reels collapses shared components into single shots. 30 modular reels might collapse to ~40 unique clips to record instead of 90.

**Status is derived-checked, not free.** Rust command layer enforces legal transitions (e.g. a reel can't reach `assembled` unless every block resolves to a selected take with an existing asset). Illegal jumps require an explicit override flag, which lands in `activity_log`.

**`posts.scheduled_at` vs `reels.target_date`.** Target date is the planning intent (calendar drag-drop, capacity math). Post scheduled times are the per-platform reality pushed to Metricool. Calendar shows both — intent for unpublished reels, actual for scheduled ones — and flags drift.

**Timestamps** are ISO-8601 strings with offset. SQLite has no date type; strings sort correctly and survive timezone hell (Metricool scheduling cares).

## Expected volumes (sizing sanity check)

| Table | Rows/year @ 100/wk | Note |
|---|---|---|
| reels | ~5,200 | trivial |
| script_blocks | ~15k | trivial |
| assets | ~30–60k | raw takes dominate; ffprobe on import is the only cost |
| posts | ~15,600 | trivial |
| metric_snapshots | ~200k (daily × 30d/post) | prune policy: keep daily for 90d, weekly after |

SQLite laughs at all of this. Full-text search (FTS5) over scripts/components/captions ships in v1 — at 5k reels/year, finding "that hook about protein myths" must be instant.
