use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{ids, CoreError, Result};

pub const MARKER_FILE: &str = ".contentos-root";

/// The standard library folder structure created inside every storage root.
pub const LIBRARY_DIRS: &[&str] = &[
    "00_INBOX",   // drop raw shoot footage here
    "01_RAW",     // app files selected takes here, per batch
    "02_HANDOFF", // app stages DaVinci import folders here
    "03_EXPORTS", // render finished reels from DaVinci to here
    "04_FINALS",  // app files matched finals here, by month
    "05_ARCHIVE", // retired material
    "06_PUBLISH", // app writes Metricool export bundles here
];

const README: &str = "\
ContentOS media library
=======================

This folder is managed by ContentOS. You only ever touch two folders by hand:

  00_INBOX    -> put your raw shoot footage here, then hit \"Scan inbox\" in the app.
  03_EXPORTS  -> render your finished reels from DaVinci Resolve to here.

Everything else is filled in automatically by the app:

  01_RAW      selected takes, renamed and filed per shoot batch
  02_HANDOFF  ready-to-edit DaVinci folders + timelines (disposable)
  04_FINALS   your matched final renders, filed by month
  05_ARCHIVE  retired material
  06_PUBLISH  Metricool export bundles (CSV + videos)

Do not rename these folders or the hidden .contentos-root file. To move this
library to another drive or your NAS, copy the WHOLE folder, then use
Settings -> Re-point in the app.
";

/// Create the standard folder structure (best-effort; a read-only subfolder
/// on a NAS shouldn't block anything).
pub fn scaffold_dirs(dir: &Path) {
    for d in LIBRARY_DIRS {
        let _ = std::fs::create_dir_all(dir.join(d));
    }
    let readme = dir.join("_READ_ME_FIRST.txt");
    if !readme.exists() {
        let _ = std::fs::write(&readme, README);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRoot {
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub online: bool,
    pub created_at: String,
    /// Free disk space at the root's volume (None when offline).
    #[serde(default)]
    pub free_bytes: Option<i64>,
}

/// Register a directory as a storage root. Writes a `.contentos-root` marker
/// (containing the root's id) so a moved root can be recognized and re-pointed.
/// If the directory already carries a marker from a previous install, that id
/// is adopted rather than replaced.
pub fn add_root(conn: &Connection, name: &str, path: &str, kind: &str) -> Result<StorageRoot> {
    if !matches!(kind, "local" | "nas") {
        return Err(CoreError::Invalid(format!("invalid root kind: {kind}")));
    }
    if name.trim().is_empty() {
        return Err(CoreError::Invalid("root name is required".into()));
    }
    let dir = Path::new(path);
    if !dir.is_dir() {
        return Err(CoreError::Invalid(format!(
            "folder does not exist or is not a directory: {path}"
        )));
    }

    let marker = dir.join(MARKER_FILE);
    let id = if marker.is_file() {
        let existing = std::fs::read_to_string(&marker)?.trim().to_string();
        let registered: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM storage_roots WHERE id = ?1)",
            [&existing],
            |r| r.get(0),
        )?;
        if registered {
            return Err(CoreError::Invalid(
                "this folder is already registered as a storage root".into(),
            ));
        }
        existing
    } else {
        let id = ids::new_id();
        std::fs::write(&marker, &id)?;
        id
    };

    let root = StorageRoot {
        id,
        name: name.trim().to_string(),
        path: path.to_string(),
        kind: kind.to_string(),
        online: true,
        created_at: ids::now_iso(),
        free_bytes: fs2::available_space(dir).ok().map(|v| v as i64),
    };
    conn.execute(
        "INSERT INTO storage_roots (id, name, path, kind, online, created_at)
         VALUES (?1, ?2, ?3, ?4, 1, ?5)",
        rusqlite::params![root.id, root.name, root.path, root.kind, root.created_at],
    )?;
    scaffold_dirs(dir);
    Ok(root)
}

/// List roots with a live online check (directory reachable + marker matches).
pub fn list_roots(conn: &Connection) -> Result<Vec<StorageRoot>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, kind, created_at FROM storage_roots ORDER BY created_at",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;

    let mut roots = Vec::new();
    for row in rows {
        let (id, name, path, kind, created_at) = row?;
        let online = check_online(&path, &id);
        conn.execute(
            "UPDATE storage_roots SET online = ?1 WHERE id = ?2",
            rusqlite::params![online as i64, id],
        )?;
        let free_bytes = if online {
            // self-heal folder structure for roots added before scaffolding,
            // or if a folder was deleted by hand
            scaffold_dirs(Path::new(&path));
            fs2::available_space(Path::new(&path)).ok().map(|v| v as i64)
        } else {
            None
        };
        roots.push(StorageRoot { id, name, path, kind, online, created_at, free_bytes });
    }
    Ok(roots)
}

/// Point an existing root at a new location (the PC → NAS move).
/// The new directory must carry this root's marker, or none (marker is written).
pub fn relocate_root(conn: &Connection, id: &str, new_path: &str) -> Result<()> {
    let dir = Path::new(new_path);
    if !dir.is_dir() {
        return Err(CoreError::Invalid(format!(
            "folder does not exist or is not a directory: {new_path}"
        )));
    }
    let marker = dir.join(MARKER_FILE);
    if marker.is_file() {
        let existing = std::fs::read_to_string(&marker)?.trim().to_string();
        if existing != id {
            return Err(CoreError::Invalid(
                "that folder belongs to a different storage root".into(),
            ));
        }
    } else {
        std::fs::write(&marker, id)?;
    }
    let changed = conn.execute(
        "UPDATE storage_roots SET path = ?1, online = 1 WHERE id = ?2",
        rusqlite::params![new_path, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("storage root {id}")));
    }
    Ok(())
}

/// Remove a root's registration. Refuses if assets still reference it.
/// Never touches files on disk (the marker is left behind on purpose).
pub fn remove_root(conn: &Connection, id: &str) -> Result<()> {
    let assets: i64 = conn.query_row(
        "SELECT COUNT(*) FROM assets WHERE root_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    if assets > 0 {
        return Err(CoreError::Invalid(format!(
            "cannot remove: {assets} indexed files still reference this root"
        )));
    }
    let changed = conn.execute("DELETE FROM storage_roots WHERE id = ?1", [id])?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("storage root {id}")));
    }
    Ok(())
}

fn check_online(path: &str, id: &str) -> bool {
    let dir = Path::new(path);
    if !dir.is_dir() {
        return false;
    }
    match std::fs::read_to_string(dir.join(MARKER_FILE)) {
        Ok(contents) => contents.trim() == id,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    #[test]
    fn add_list_remove_root() {
        let db = Db::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let root = add_root(&db.conn, "media", path, "local").unwrap();
        assert!(dir.path().join(MARKER_FILE).is_file());

        let listed = list_roots(&db.conn).unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].online);

        // duplicate registration is refused
        assert!(add_root(&db.conn, "media2", path, "local").is_err());

        remove_root(&db.conn, &root.id).unwrap();
        assert!(list_roots(&db.conn).unwrap().is_empty());
    }

    #[test]
    fn relocate_moves_the_root() {
        let db = Db::open_in_memory().unwrap();
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();

        let root = add_root(&db.conn, "media", old_dir.path().to_str().unwrap(), "local").unwrap();
        // simulate the copy: marker travels with the tree
        std::fs::copy(
            old_dir.path().join(MARKER_FILE),
            new_dir.path().join(MARKER_FILE),
        )
        .unwrap();
        drop(old_dir); // old location gone

        relocate_root(&db.conn, &root.id, new_dir.path().to_str().unwrap()).unwrap();
        let listed = list_roots(&db.conn).unwrap();
        assert!(listed[0].online);
        assert_eq!(listed[0].path, new_dir.path().to_str().unwrap());
    }

    #[test]
    fn offline_root_is_reported() {
        let db = Db::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        add_root(&db.conn, "media", &path, "local").unwrap();
        drop(dir); // directory disappears (NAS unplugged)
        let listed = list_roots(&db.conn).unwrap();
        assert!(!listed[0].online);
    }
}
