-- ContentOS schema v1 — see docs/03-data-model.md

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE counters (
  name  TEXT PRIMARY KEY,
  value INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE storage_roots (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL UNIQUE,
  path       TEXT NOT NULL,
  kind       TEXT NOT NULL CHECK (kind IN ('local','nas')),
  online     INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
);

CREATE TABLE pillars (
  id              TEXT PRIMARY KEY,
  name            TEXT NOT NULL UNIQUE,
  color           TEXT NOT NULL,
  description     TEXT,
  target_per_week INTEGER NOT NULL DEFAULT 0,
  sort_order      INTEGER NOT NULL DEFAULT 0,
  archived        INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE ideas (
  id         TEXT PRIMARY KEY,
  title      TEXT NOT NULL,
  notes      TEXT,
  pillar_id  TEXT REFERENCES pillars(id),
  status     TEXT NOT NULL DEFAULT 'open'
             CHECK (status IN ('open','promoted','killed')),
  reel_id    TEXT REFERENCES reels(id),
  created_at TEXT NOT NULL
);

CREATE TABLE reels (
  id             TEXT PRIMARY KEY,
  code           TEXT NOT NULL UNIQUE,
  title          TEXT NOT NULL,
  slug           TEXT NOT NULL,
  pillar_id      TEXT REFERENCES pillars(id),
  status         TEXT NOT NULL DEFAULT 'idea' CHECK (status IN
                 ('idea','scripted','shotlisted','shot','assembled',
                  'editing','edited','scheduled','posted','verified',
                  'archived','killed')),
  notes          TEXT,
  target_date    TEXT,
  final_asset_id TEXT REFERENCES assets(id),
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL
);
CREATE INDEX idx_reels_status ON reels(status);
CREATE INDEX idx_reels_target ON reels(target_date);

CREATE TABLE components (
  id             TEXT PRIMARY KEY,
  code           TEXT NOT NULL UNIQUE,
  kind           TEXT NOT NULL CHECK (kind IN ('hook','body','cta')),
  text           TEXT NOT NULL,
  tags           TEXT,
  pillar_id      TEXT REFERENCES pillars(id),
  master_take_id TEXT REFERENCES takes(id),
  archived       INTEGER NOT NULL DEFAULT 0,
  created_at     TEXT NOT NULL
);

CREATE TABLE script_blocks (
  id           TEXT PRIMARY KEY,
  reel_id      TEXT NOT NULL REFERENCES reels(id) ON DELETE CASCADE,
  position     INTEGER NOT NULL,
  kind         TEXT NOT NULL CHECK (kind IN ('hook','body','cta','segment')),
  component_id TEXT REFERENCES components(id),
  text         TEXT,
  est_seconds  INTEGER,
  CHECK (component_id IS NOT NULL OR text IS NOT NULL),
  UNIQUE (reel_id, position)
);

CREATE TABLE shoot_batches (
  id         TEXT PRIMARY KEY,
  code       TEXT NOT NULL UNIQUE,
  name       TEXT NOT NULL,
  shoot_date TEXT,
  status     TEXT NOT NULL DEFAULT 'planning'
             CHECK (status IN ('planning','ready','shooting','done')),
  notes      TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE shots (
  id               TEXT PRIMARY KEY,
  batch_id         TEXT NOT NULL REFERENCES shoot_batches(id) ON DELETE CASCADE,
  position         INTEGER NOT NULL,
  component_id     TEXT REFERENCES components(id),
  script_block_id  TEXT REFERENCES script_blocks(id),
  setup_tags       TEXT,
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
  rating      INTEGER,
  notes       TEXT
);

CREATE TABLE assets (
  id          TEXT PRIMARY KEY,
  root_id     TEXT NOT NULL REFERENCES storage_roots(id),
  rel_path    TEXT NOT NULL,
  filename    TEXT NOT NULL,
  kind        TEXT NOT NULL CHECK (kind IN ('raw','final','other')),
  size_bytes  INTEGER NOT NULL,
  blake3      TEXT,
  duration_ms INTEGER,
  width       INTEGER,
  height      INTEGER,
  fps         REAL,
  codec       TEXT,
  recorded_at TEXT,
  imported_at TEXT NOT NULL,
  missing     INTEGER NOT NULL DEFAULT 0,
  UNIQUE (root_id, rel_path)
);
CREATE INDEX idx_assets_hash ON assets(blake3);

CREATE TABLE posts (
  id                TEXT PRIMARY KEY,
  reel_id           TEXT NOT NULL REFERENCES reels(id) ON DELETE CASCADE,
  platform          TEXT NOT NULL CHECK (platform IN ('instagram','tiktok','youtube')),
  caption           TEXT,
  hashtags          TEXT,
  scheduled_at      TEXT,
  status            TEXT NOT NULL DEFAULT 'draft' CHECK (status IN
                    ('draft','queued','pushed','scheduled','published',
                     'verified','failed','canceled')),
  metricool_post_id TEXT,
  published_url     TEXT,
  error             TEXT,
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
  raw        TEXT
);

CREATE TABLE caption_templates (
  id       TEXT PRIMARY KEY,
  name     TEXT NOT NULL,
  platform TEXT CHECK (platform IN ('instagram','tiktok','youtube')),
  body     TEXT NOT NULL
);

CREATE TABLE jobs (
  id          TEXT PRIMARY KEY,
  kind        TEXT NOT NULL,
  payload     TEXT NOT NULL,
  status      TEXT NOT NULL DEFAULT 'queued'
              CHECK (status IN ('queued','running','done','failed')),
  attempts    INTEGER NOT NULL DEFAULT 0,
  error       TEXT,
  created_at  TEXT NOT NULL,
  finished_at TEXT
);
CREATE INDEX idx_jobs_status ON jobs(status, created_at);

CREATE TABLE activity_log (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  at         TEXT NOT NULL,
  entity     TEXT NOT NULL,
  entity_id  TEXT NOT NULL,
  action     TEXT NOT NULL,
  detail     TEXT
);

CREATE TABLE ai_generations (
  id         TEXT PRIMARY KEY,
  kind       TEXT NOT NULL,
  input      TEXT NOT NULL,
  output     TEXT NOT NULL,
  model      TEXT,
  accepted   INTEGER NOT NULL DEFAULT 0,
  entity     TEXT,
  entity_id  TEXT,
  created_at TEXT NOT NULL
);
