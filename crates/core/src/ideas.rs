use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{ids, reels, CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idea {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub pillar_id: Option<String>,
    pub status: String,
    pub reel_id: Option<String>,
    pub created_at: String,
}

pub fn create(conn: &Connection, title: &str, pillar_id: Option<&str>) -> Result<Idea> {
    if title.trim().is_empty() {
        return Err(CoreError::Invalid("idea title is required".into()));
    }
    let idea = Idea {
        id: ids::new_id(),
        title: title.trim().to_string(),
        notes: None,
        pillar_id: pillar_id.map(str::to_string),
        status: "open".into(),
        reel_id: None,
        created_at: ids::now_iso(),
    };
    conn.execute(
        "INSERT INTO ideas (id, title, pillar_id, status, created_at)
         VALUES (?1, ?2, ?3, 'open', ?4)",
        rusqlite::params![idea.id, idea.title, idea.pillar_id, idea.created_at],
    )?;
    Ok(idea)
}

pub fn update(
    conn: &Connection,
    id: &str,
    title: &str,
    notes: Option<&str>,
    pillar_id: Option<&str>,
) -> Result<()> {
    let changed = conn.execute(
        "UPDATE ideas SET title = ?1, notes = ?2, pillar_id = ?3 WHERE id = ?4",
        rusqlite::params![title.trim(), notes, pillar_id, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("idea {id}")));
    }
    Ok(())
}

pub fn kill(conn: &Connection, id: &str) -> Result<()> {
    let changed = conn.execute(
        "UPDATE ideas SET status = 'killed' WHERE id = ?1 AND status = 'open'",
        [id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("open idea {id}")));
    }
    Ok(())
}

/// Turn an open idea into a reel; the idea keeps a link to it.
pub fn promote(conn: &Connection, id: &str) -> Result<reels::ReelDetail> {
    let idea = list(conn, Some("open"))?
        .into_iter()
        .find(|i| i.id == id)
        .ok_or_else(|| CoreError::NotFound(format!("open idea {id}")))?;
    let reel = reels::create(conn, &idea.title, idea.pillar_id.as_deref())?;
    if let Some(notes) = &idea.notes {
        reels::update_meta(conn, &reel.id, &reel.title, Some(notes), idea.pillar_id.as_deref())?;
    }
    conn.execute(
        "UPDATE ideas SET status = 'promoted', reel_id = ?1 WHERE id = ?2",
        rusqlite::params![reel.id, id],
    )?;
    reels::get_detail(conn, &reel.id)
}

pub fn list(conn: &Connection, status: Option<&str>) -> Result<Vec<Idea>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, notes, pillar_id, status, reel_id, created_at
         FROM ideas WHERE (?1 IS NULL OR status = ?1) ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([status], |r| {
        Ok(Idea {
            id: r.get(0)?,
            title: r.get(1)?,
            notes: r.get(2)?,
            pillar_id: r.get(3)?,
            status: r.get(4)?,
            reel_id: r.get(5)?,
            created_at: r.get(6)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    #[test]
    fn capture_promote_kill() {
        let db = Db::open_in_memory().unwrap();
        let a = create(&db.conn, "Why your hooks flop", None).unwrap();
        let b = create(&db.conn, "Morning routine myth", None).unwrap();

        let reel = promote(&db.conn, &a.id).unwrap();
        assert_eq!(reel.title, "Why your hooks flop");
        assert_eq!(reel.status, "idea");

        // promoted idea is no longer open, carries the reel link
        let open = list(&db.conn, Some("open")).unwrap();
        assert_eq!(open.len(), 1);
        let all = list(&db.conn, None).unwrap();
        assert!(all.iter().any(|i| i.reel_id.as_deref() == Some(reel.id.as_str())));

        // can't promote twice
        assert!(promote(&db.conn, &a.id).is_err());

        kill(&db.conn, &b.id).unwrap();
        assert!(list(&db.conn, Some("open")).unwrap().is_empty());
    }
}
