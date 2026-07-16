-- Full-text search across reels (title + notes + script text) and components.
-- Maintained by core functions on every write (no triggers).
CREATE VIRTUAL TABLE fts USING fts5(
  content,
  entity    UNINDEXED,
  entity_id UNINDEXED,
  tokenize = 'unicode61'
);
