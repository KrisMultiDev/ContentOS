use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::{batches, ids, CoreError, Result};

pub const INBOX_DIR: &str = "00_INBOX";
pub const RAW_DIR: &str = "01_RAW";
const VIDEO_EXTS: &[&str] = &["mp4", "mov", "mkv", "avi", "m4v", "webm"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetView {
    pub id: String,
    pub root_id: String,
    pub rel_path: String,
    pub filename: String,
    pub kind: String,
    pub size_bytes: i64,
    pub duration_ms: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub imported_at: String,
    pub missing: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanReport {
    pub scanned: i64,
    pub added: i64,
    pub duplicates: i64,
}

fn root_path(conn: &Connection, root_id: &str) -> Result<PathBuf> {
    let path: Option<String> = conn
        .query_row("SELECT path FROM storage_roots WHERE id = ?1", [root_id], |r| r.get(0))
        .optional()?;
    path.map(PathBuf::from)
        .ok_or_else(|| CoreError::NotFound(format!("storage root {root_id}")))
}

fn rel(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| VIDEO_EXTS.contains(&e.to_lowercase().as_str()))
}

fn hash_file(path: &Path) -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    let mut file = std::fs::File::open(path)?;
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize().to_hex().to_string())
}

/// Index new video files in the root's 00_INBOX (creates it if missing).
/// Files already indexed (same path) are skipped; files whose content hash is
/// already known elsewhere are recorded as duplicates and NOT re-added.
pub fn scan_inbox(conn: &Connection, root_id: &str) -> Result<ScanReport> {
    let root = root_path(conn, root_id)?;
    let inbox = root.join(INBOX_DIR);
    std::fs::create_dir_all(&inbox)?;

    let mut report = ScanReport { scanned: 0, added: 0, duplicates: 0 };
    let mut stack = vec![inbox];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !is_video(&path) {
                continue;
            }
            report.scanned += 1;
            let rel_path = rel(&root, &path);

            let known: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM assets WHERE root_id = ?1 AND rel_path = ?2)",
                rusqlite::params![root_id, rel_path],
                |r| r.get(0),
            )?;
            if known {
                continue;
            }

            let hash = hash_file(&path)?;
            let dupe: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM assets WHERE blake3 = ?1 AND missing = 0)",
                [&hash],
                |r| r.get(0),
            )?;
            if dupe {
                report.duplicates += 1;
                continue;
            }

            let meta = std::fs::metadata(&path)?;
            conn.execute(
                "INSERT INTO assets (id, root_id, rel_path, filename, kind, size_bytes, blake3, imported_at)
                 VALUES (?1, ?2, ?3, ?4, 'raw', ?5, ?6, ?7)",
                rusqlite::params![
                    ids::new_id(),
                    root_id,
                    rel_path,
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    meta.len() as i64,
                    hash,
                    ids::now_iso()
                ],
            )?;
            report.added += 1;
        }
    }
    Ok(report)
}

/// Inbox assets not yet linked to any take, oldest first (shoot order).
pub fn list_inbox(conn: &Connection, root_id: &str) -> Result<Vec<AssetView>> {
    let mut stmt = conn.prepare(
        "SELECT id, root_id, rel_path, filename, kind, size_bytes, duration_ms, width, height, imported_at, missing
         FROM assets
         WHERE root_id = ?1 AND rel_path LIKE '00_INBOX/%'
           AND id NOT IN (SELECT asset_id FROM takes)
         ORDER BY imported_at, filename",
    )?;
    let rows = stmt.query_map([root_id], row_to_asset)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn list_assets(conn: &Connection, kind: Option<&str>) -> Result<Vec<AssetView>> {
    let mut stmt = conn.prepare(
        "SELECT id, root_id, rel_path, filename, kind, size_bytes, duration_ms, width, height, imported_at, missing
         FROM assets WHERE (?1 IS NULL OR kind = ?1) ORDER BY imported_at DESC",
    )?;
    let rows = stmt.query_map([kind], row_to_asset)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Link an inbox asset to a shot as its next take. The file moves to
/// `01_RAW/<batch>/` with its canonical name `B012_HK0031_T03.mp4`, the shot
/// flips to `recorded`, and the take is returned.
pub fn ingest_take(
    conn: &Connection,
    shot_id: &str,
    asset_id: &str,
    rating: Option<i64>,
) -> Result<batches::TakeView> {
    let (batch_id, batch_code): (String, String) = conn
        .query_row(
            "SELECT b.id, b.code FROM shots s JOIN shoot_batches b ON b.id = s.batch_id WHERE s.id = ?1",
            [shot_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| CoreError::NotFound(format!("shot {shot_id}")))?;

    let batch = batches::detail(conn, &batch_id)?;
    let shot = batch
        .shots
        .iter()
        .find(|s| s.id == shot_id)
        .ok_or_else(|| CoreError::NotFound(format!("shot {shot_id}")))?;

    let (root_id, rel_path): (String, String) = conn
        .query_row(
            "SELECT root_id, rel_path FROM assets WHERE id = ?1",
            [asset_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| CoreError::NotFound(format!("asset {asset_id}")))?;

    let take_number: i64 = conn.query_row(
        "SELECT COALESCE(MAX(take_number), 0) + 1 FROM takes WHERE shot_id = ?1",
        [shot_id],
        |r| r.get(0),
    )?;

    // canonical rename + move (same volume: instant)
    let root = root_path(conn, &root_id)?;
    let src = root.join(&rel_path);
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp4")
        .to_lowercase();
    let new_name = format!("{batch_code}_{}_T{take_number:02}.{ext}", shot.clip_key);
    let dest_dir = root.join(RAW_DIR).join(&batch_code);
    std::fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(&new_name);
    if dest.exists() {
        return Err(CoreError::Invalid(format!("{new_name} already exists")));
    }
    std::fs::rename(&src, &dest)?;

    let new_rel = rel(&root, &dest);
    conn.execute(
        "UPDATE assets SET rel_path = ?1, filename = ?2 WHERE id = ?3",
        rusqlite::params![new_rel, new_name, asset_id],
    )?;

    let take_id = ids::new_id();
    conn.execute(
        "INSERT INTO takes (id, shot_id, asset_id, take_number, rating) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![take_id, shot_id, asset_id, take_number, rating],
    )?;
    batches::set_shot_status(conn, shot_id, "recorded")?;

    Ok(batches::TakeView {
        id: take_id,
        take_number,
        asset_id: asset_id.to_string(),
        filename: new_name,
        rating,
        selected: false,
    })
}

fn row_to_asset(r: &rusqlite::Row<'_>) -> rusqlite::Result<AssetView> {
    Ok(AssetView {
        id: r.get(0)?,
        root_id: r.get(1)?,
        rel_path: r.get(2)?,
        filename: r.get(3)?,
        kind: r.get(4)?,
        size_bytes: r.get(5)?,
        duration_ms: r.get(6)?,
        width: r.get(7)?,
        height: r.get(8)?,
        imported_at: r.get(9)?,
        missing: r.get::<_, i64>(10)? != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{roots, Db};

    #[test]
    fn scan_dedup_and_inbox() {
        let db = Db::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let root = roots::add_root(&db.conn, "media", dir.path().to_str().unwrap(), "local").unwrap();

        let inbox = dir.path().join(INBOX_DIR);
        std::fs::create_dir_all(&inbox).unwrap();
        std::fs::write(inbox.join("a.mp4"), "video-a").unwrap();
        std::fs::write(inbox.join("b.MOV"), "video-b").unwrap();
        std::fs::write(inbox.join("copy-of-a.mp4"), "video-a").unwrap(); // duplicate content
        std::fs::write(inbox.join("notes.txt"), "not a video").unwrap();

        let report = scan_inbox(&db.conn, &root.id).unwrap();
        assert_eq!(report.scanned, 3);
        assert_eq!(report.added, 2);
        assert_eq!(report.duplicates, 1);

        // rescan is a no-op
        let report = scan_inbox(&db.conn, &root.id).unwrap();
        assert_eq!(report.added, 0);

        let inbox_assets = list_inbox(&db.conn, &root.id).unwrap();
        assert_eq!(inbox_assets.len(), 2);
        assert!(inbox_assets.iter().all(|a| a.kind == "raw"));
    }
}
