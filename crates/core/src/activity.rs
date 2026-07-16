use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;

use crate::{ids, Result};

#[derive(Debug, Serialize)]
pub struct ActivityEntry {
    pub id: i64,
    pub at: String,
    pub entity: String,
    pub entity_id: String,
    pub action: String,
    pub detail: Option<Value>,
}

pub fn log(
    conn: &Connection,
    entity: &str,
    entity_id: &str,
    action: &str,
    detail: Option<&Value>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO activity_log (at, entity, entity_id, action, detail)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![ids::now_iso(), entity, entity_id, action, detail.map(|d| d.to_string())],
    )?;
    Ok(())
}

pub fn recent(conn: &Connection, limit: i64) -> Result<Vec<ActivityEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, at, entity, entity_id, action, detail
         FROM activity_log ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit], |r| {
        let detail: Option<String> = r.get(5)?;
        Ok(ActivityEntry {
            id: r.get(0)?,
            at: r.get(1)?,
            entity: r.get(2)?,
            entity_id: r.get(3)?,
            action: r.get(4)?,
            detail: detail.and_then(|d| serde_json::from_str(&d).ok()),
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;
    use serde_json::json;

    #[test]
    fn log_and_read_back() {
        let db = Db::open_in_memory().unwrap();
        log(&db.conn, "reel", "R0001", "status:shot→assembled", Some(&json!({"batch": "B001"}))).unwrap();
        log(&db.conn, "reel", "R0002", "created", None).unwrap();
        let entries = recent(&db.conn, 10).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].entity_id, "R0002"); // newest first
        assert!(entries[1].detail.is_some());
    }
}
