use rusqlite::{Connection, OptionalExtension};
use serde_json::Value;

use crate::Result;

pub fn get(conn: &Connection, key: &str) -> Result<Option<Value>> {
    let raw: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()?;
    Ok(match raw {
        Some(s) => Some(serde_json::from_str(&s).unwrap_or(Value::String(s))),
        None => None,
    })
}

pub fn set(conn: &Connection, key: &str, value: &Value) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value.to_string()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;
    use serde_json::json;

    #[test]
    fn roundtrip_and_overwrite() {
        let db = Db::open_in_memory().unwrap();
        assert!(get(&db.conn, "weekly_target").unwrap().is_none());
        set(&db.conn, "weekly_target", &json!(100)).unwrap();
        assert_eq!(get(&db.conn, "weekly_target").unwrap(), Some(json!(100)));
        set(&db.conn, "weekly_target", &json!({ "reels": 100, "platforms": 3 })).unwrap();
        assert_eq!(
            get(&db.conn, "weekly_target").unwrap(),
            Some(json!({ "reels": 100, "platforms": 3 }))
        );
    }
}
