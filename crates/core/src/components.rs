use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{ids, search, CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub code: String,
    pub kind: String,
    pub text: String,
    pub tags: Vec<String>,
    pub pillar_id: Option<String>,
    pub archived: bool,
    pub created_at: String,
    /// How many script blocks reference this component.
    pub times_used: i64,
}

fn code_prefix(kind: &str) -> Result<&'static str> {
    match kind {
        "hook" => Ok("HK"),
        "body" => Ok("BD"),
        "cta" => Ok("CT"),
        _ => Err(CoreError::Invalid(format!("invalid component kind: {kind}"))),
    }
}

fn fts_content(text: &str, tags: &[String]) -> String {
    if tags.is_empty() {
        text.to_string()
    } else {
        format!("{text} {}", tags.join(" "))
    }
}

pub fn create(
    conn: &Connection,
    kind: &str,
    text: &str,
    tags: &[String],
    pillar_id: Option<&str>,
) -> Result<Component> {
    if text.trim().is_empty() {
        return Err(CoreError::Invalid("component text is required".into()));
    }
    let prefix = code_prefix(kind)?;
    let code = ids::next_code(conn, prefix, 4)?;
    let component = Component {
        id: ids::new_id(),
        code,
        kind: kind.to_string(),
        text: text.trim().to_string(),
        tags: tags.to_vec(),
        pillar_id: pillar_id.map(str::to_string),
        archived: false,
        created_at: ids::now_iso(),
        times_used: 0,
    };
    conn.execute(
        "INSERT INTO components (id, code, kind, text, tags, pillar_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            component.id,
            component.code,
            component.kind,
            component.text,
            serde_json::to_string(&component.tags).unwrap(),
            component.pillar_id,
            component.created_at
        ],
    )?;
    search::upsert(conn, "component", &component.id, &fts_content(&component.text, &component.tags))?;
    Ok(component)
}

pub fn update(
    conn: &Connection,
    id: &str,
    text: &str,
    tags: &[String],
    pillar_id: Option<&str>,
) -> Result<()> {
    if text.trim().is_empty() {
        return Err(CoreError::Invalid("component text is required".into()));
    }
    let changed = conn.execute(
        "UPDATE components SET text = ?1, tags = ?2, pillar_id = ?3 WHERE id = ?4",
        rusqlite::params![text.trim(), serde_json::to_string(tags).unwrap(), pillar_id, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("component {id}")));
    }
    search::upsert(conn, "component", id, &fts_content(text.trim(), tags))?;
    Ok(())
}

pub fn set_archived(conn: &Connection, id: &str, archived: bool) -> Result<()> {
    let changed = conn.execute(
        "UPDATE components SET archived = ?1 WHERE id = ?2",
        rusqlite::params![archived as i64, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("component {id}")));
    }
    Ok(())
}

pub fn list(
    conn: &Connection,
    kind: Option<&str>,
    include_archived: bool,
    query: Option<&str>,
) -> Result<Vec<Component>> {
    // When a search query is present, restrict to FTS matches (preserving rank).
    let match_ids: Option<Vec<String>> = match query {
        Some(q) if !q.trim().is_empty() => Some(search::matching_ids(conn, "component", q)?),
        _ => None,
    };

    let mut stmt = conn.prepare(
        "SELECT c.id, c.code, c.kind, c.text, c.tags, c.pillar_id, c.archived, c.created_at,
                (SELECT COUNT(*) FROM script_blocks sb WHERE sb.component_id = c.id)
         FROM components c
         WHERE (c.archived = 0 OR ?1) AND (?2 IS NULL OR c.kind = ?2)
         ORDER BY c.created_at DESC",
    )?;
    let rows = stmt.query_map(rusqlite::params![include_archived as i64, kind], |r| {
        let tags: String = r.get(4)?;
        Ok(Component {
            id: r.get(0)?,
            code: r.get(1)?,
            kind: r.get(2)?,
            text: r.get(3)?,
            tags: serde_json::from_str(&tags).unwrap_or_default(),
            pillar_id: r.get(5)?,
            archived: r.get::<_, i64>(6)? != 0,
            created_at: r.get(7)?,
            times_used: r.get(8)?,
        })
    })?;
    let mut all: Vec<Component> = rows.collect::<std::result::Result<Vec<_>, _>>()?;

    if let Some(ids) = match_ids {
        let rank: std::collections::HashMap<&str, usize> =
            ids.iter().enumerate().map(|(i, id)| (id.as_str(), i)).collect();
        all.retain(|c| rank.contains_key(c.id.as_str()));
        all.sort_by_key(|c| rank[c.id.as_str()]);
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    #[test]
    fn create_codes_and_search() {
        let db = Db::open_in_memory().unwrap();
        let h = create(&db.conn, "hook", "Stop doing these 3 exercises", &["gym".into()], None).unwrap();
        let b = create(&db.conn, "body", "Here is why form beats weight…", &[], None).unwrap();
        assert_eq!(h.code, "HK0001");
        assert_eq!(b.code, "BD0001");

        let hooks = list(&db.conn, Some("hook"), false, None).unwrap();
        assert_eq!(hooks.len(), 1);

        let found = list(&db.conn, None, false, Some("exercises")).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, h.id);

        update(&db.conn, &h.id, "Stop doing these 5 exercises", &["gym".into(), "myths".into()], None).unwrap();
        let found = list(&db.conn, None, false, Some("myths")).unwrap();
        assert_eq!(found.len(), 1, "tags are searchable after update");

        assert!(create(&db.conn, "banana", "x", &[], None).is_err());
    }
}
