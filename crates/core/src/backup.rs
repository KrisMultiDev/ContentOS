use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::Result;

const KEEP: usize = 30;

/// Online snapshot of the live database (WAL-safe via SQLite's backup API).
pub fn backup_now(conn: &Connection, backups_dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(backups_dir)?;
    let stamp = crate::ids::now_iso();
    let name = format!("contentos-{}.db", &stamp[..10]); // one per day, overwritten
    let dest = backups_dir.join(name);
    conn.backup(rusqlite::DatabaseName::Main, &dest, None)?;
    prune(backups_dir)?;
    Ok(dest)
}

/// Startup hook: back up at most once per day.
pub fn daily_backup(conn: &Connection, backups_dir: &Path) -> Result<Option<PathBuf>> {
    let stamp = crate::ids::now_iso();
    let today = backups_dir.join(format!("contentos-{}.db", &stamp[..10]));
    if today.is_file() {
        return Ok(None);
    }
    backup_now(conn, backups_dir).map(Some)
}

fn prune(backups_dir: &Path) -> Result<()> {
    let mut backups: Vec<PathBuf> = std::fs::read_dir(backups_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("contentos-") && n.ends_with(".db"))
        })
        .collect();
    backups.sort(); // date-stamped names sort chronologically
    while backups.len() > KEEP {
        std::fs::remove_file(backups.remove(0))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    #[test]
    fn backup_and_daily_skip() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("live.db");
        let backups = dir.path().join("backups");
        let db = Db::open(&db_path).unwrap();
        db.conn.execute("INSERT INTO settings (key, value) VALUES ('x', '1')", []).unwrap();

        let first = daily_backup(&db.conn, &backups).unwrap();
        assert!(first.is_some());
        // restored copy opens and carries data
        let restored = Db::open(first.as_deref().unwrap()).unwrap();
        let v: String = restored
            .conn
            .query_row("SELECT value FROM settings WHERE key = 'x'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, "1");

        // second call the same day is a no-op
        assert!(daily_backup(&db.conn, &backups).unwrap().is_none());
        // explicit backup still works (overwrites today's)
        backup_now(&db.conn, &backups).unwrap();
    }
}
