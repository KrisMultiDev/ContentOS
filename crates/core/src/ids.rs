use rusqlite::Connection;

use crate::Result;

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn now_iso() -> String {
    chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
}

/// Next human-readable code for a prefix, e.g. `next_code(conn, "R", 4)` → "R0001".
/// Backed by the `counters` table so codes are immutable and never reused.
pub fn next_code(conn: &Connection, prefix: &str, width: usize) -> Result<String> {
    conn.execute(
        "INSERT INTO counters (name, value) VALUES (?1, 1)
         ON CONFLICT(name) DO UPDATE SET value = value + 1",
        [prefix],
    )?;
    let value: i64 = conn.query_row("SELECT value FROM counters WHERE name = ?1", [prefix], |r| {
        r.get(0)
    })?;
    Ok(format!("{prefix}{value:0width$}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    #[test]
    fn codes_increment_per_prefix() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(next_code(&db.conn, "R", 4).unwrap(), "R0001");
        assert_eq!(next_code(&db.conn, "R", 4).unwrap(), "R0002");
        assert_eq!(next_code(&db.conn, "B", 3).unwrap(), "B001");
        assert_eq!(next_code(&db.conn, "HK", 4).unwrap(), "HK0001");
    }
}
