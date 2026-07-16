use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{ids, CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pillar {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub target_per_week: i64,
    pub sort_order: i64,
    pub archived: bool,
}

pub fn create(conn: &Connection, name: &str, color: &str, target_per_week: i64) -> Result<Pillar> {
    if name.trim().is_empty() {
        return Err(CoreError::Invalid("pillar name is required".into()));
    }
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM pillars",
        [],
        |r| r.get(0),
    )?;
    let pillar = Pillar {
        id: ids::new_id(),
        name: name.trim().to_string(),
        color: color.to_string(),
        description: None,
        target_per_week,
        sort_order: next_order,
        archived: false,
    };
    conn.execute(
        "INSERT INTO pillars (id, name, color, target_per_week, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![pillar.id, pillar.name, pillar.color, pillar.target_per_week, pillar.sort_order],
    )?;
    Ok(pillar)
}

pub fn update(
    conn: &Connection,
    id: &str,
    name: &str,
    color: &str,
    target_per_week: i64,
) -> Result<()> {
    let changed = conn.execute(
        "UPDATE pillars SET name = ?1, color = ?2, target_per_week = ?3 WHERE id = ?4",
        rusqlite::params![name.trim(), color, target_per_week, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("pillar {id}")));
    }
    Ok(())
}

pub fn set_archived(conn: &Connection, id: &str, archived: bool) -> Result<()> {
    let changed = conn.execute(
        "UPDATE pillars SET archived = ?1 WHERE id = ?2",
        rusqlite::params![archived as i64, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("pillar {id}")));
    }
    Ok(())
}

pub fn list(conn: &Connection, include_archived: bool) -> Result<Vec<Pillar>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, description, target_per_week, sort_order, archived
         FROM pillars WHERE archived = 0 OR ?1 ORDER BY sort_order",
    )?;
    let rows = stmt.query_map([include_archived as i64], |r| {
        Ok(Pillar {
            id: r.get(0)?,
            name: r.get(1)?,
            color: r.get(2)?,
            description: r.get(3)?,
            target_per_week: r.get(4)?,
            sort_order: r.get(5)?,
            archived: r.get::<_, i64>(6)? != 0,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    #[test]
    fn crud_and_archive() {
        let db = Db::open_in_memory().unwrap();
        let p = create(&db.conn, "Gym myths", "moss", 30).unwrap();
        create(&db.conn, "Nutrition", "clay", 20).unwrap();

        assert_eq!(list(&db.conn, false).unwrap().len(), 2);
        update(&db.conn, &p.id, "Gym Myths", "teal", 25).unwrap();
        set_archived(&db.conn, &p.id, true).unwrap();
        assert_eq!(list(&db.conn, false).unwrap().len(), 1);
        assert_eq!(list(&db.conn, true).unwrap().len(), 2);

        // duplicate name refused by UNIQUE
        assert!(create(&db.conn, "Nutrition", "ochre", 5).is_err());
    }
}
