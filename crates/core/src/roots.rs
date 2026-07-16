use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{ids, CoreError, Result};

pub const MARKER_FILE: &str = ".contentos-root";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRoot {
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub online: bool,
    pub created_at: String,
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
    };
    conn.execute(
        "INSERT INTO storage_roots (id, name, path, kind, online, created_at)
         VALUES (?1, ?2, ?3, ?4, 1, ?5)",
        rusqlite::params![root.id, root.name, root.path, root.kind, root.created_at],
    )?;
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
        roots.push(StorageRoot { id, name, path, kind, online, created_at });
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
