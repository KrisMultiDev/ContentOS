use rusqlite::Connection;
use serde::Serialize;

use crate::Result;

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub entity: String,
    pub entity_id: String,
    pub snippet: String,
}

pub fn upsert(conn: &Connection, entity: &str, entity_id: &str, content: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM fts WHERE entity = ?1 AND entity_id = ?2",
        [entity, entity_id],
    )?;
    conn.execute(
        "INSERT INTO fts (content, entity, entity_id) VALUES (?1, ?2, ?3)",
        [content, entity, entity_id],
    )?;
    Ok(())
}

pub fn remove(conn: &Connection, entity: &str, entity_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM fts WHERE entity = ?1 AND entity_id = ?2",
        [entity, entity_id],
    )?;
    Ok(())
}

/// Escape user input into an FTS5 prefix query: each token quoted + '*'.
pub fn to_match_query(input: &str) -> String {
    input
        .split_whitespace()
        .map(|tok| format!("\"{}\"*", tok.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn query(conn: &Connection, q: &str, limit: i64) -> Result<Vec<SearchHit>> {
    let match_q = to_match_query(q);
    if match_q.is_empty() {
        return Ok(Vec::new());
    }
    let mut stmt = conn.prepare(
        "SELECT entity, entity_id, snippet(fts, 0, '[', ']', '…', 12)
         FROM fts WHERE fts MATCH ?1 ORDER BY rank LIMIT ?2",
    )?;
    let rows = stmt.query_map(rusqlite::params![match_q, limit], |r| {
        Ok(SearchHit {
            entity: r.get(0)?,
            entity_id: r.get(1)?,
            snippet: r.get(2)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// IDs of one entity type matching a query — used for filtered list views.
pub fn matching_ids(conn: &Connection, entity: &str, q: &str) -> Result<Vec<String>> {
    let match_q = to_match_query(q);
    if match_q.is_empty() {
        return Ok(Vec::new());
    }
    let mut stmt = conn.prepare(
        "SELECT entity_id FROM fts WHERE fts MATCH ?1 AND entity = ?2 ORDER BY rank",
    )?;
    let rows = stmt.query_map(rusqlite::params![match_q, entity], |r| r.get(0))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    #[test]
    fn fts5_is_available_and_searchable() {
        let db = Db::open_in_memory().unwrap();
        upsert(&db.conn, "reel", "r1", "Gym myths that keep you small").unwrap();
        upsert(&db.conn, "component", "c1", "Stop doing cardio before weights").unwrap();

        let hits = query(&db.conn, "gym my", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity_id, "r1");

        // update replaces, remove deletes
        upsert(&db.conn, "reel", "r1", "Protein timing is a lie").unwrap();
        assert!(query(&db.conn, "gym", 10).unwrap().is_empty());
        remove(&db.conn, "component", "c1").unwrap();
        assert!(query(&db.conn, "cardio", 10).unwrap().is_empty());
    }

    #[test]
    fn match_query_escapes_quotes() {
        assert_eq!(to_match_query("hook \"quoted\""), "\"hook\"* \"\"\"quoted\"\"\"*");
    }
}
