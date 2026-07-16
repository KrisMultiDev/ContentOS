use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::{activity, ids, reels, CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub id: String,
    pub code: String,
    pub name: String,
    pub shoot_date: Option<String>,
    pub status: String,
    pub shots_total: i64,
    pub shots_recorded: i64,
    pub reel_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeView {
    pub id: String,
    pub take_number: i64,
    pub asset_id: String,
    pub filename: String,
    pub rating: Option<i64>,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShotView {
    pub id: String,
    pub position: i64,
    pub status: String,
    pub kind: String,
    /// Filename key: component code (HK0031) or reel-block (R0142-S03).
    pub clip_key: String,
    pub text: String,
    /// Reel codes that use this shot's material.
    pub used_by: Vec<String>,
    pub takes: Vec<TakeView>,
    pub selected_take_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDetail {
    pub id: String,
    pub code: String,
    pub name: String,
    pub shoot_date: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub shots: Vec<ShotView>,
}

pub fn create(conn: &Connection, name: &str, shoot_date: Option<&str>) -> Result<BatchSummary> {
    if name.trim().is_empty() {
        return Err(CoreError::Invalid("batch name is required".into()));
    }
    let code = ids::next_code(conn, "B", 3)?;
    let id = ids::new_id();
    let created_at = ids::now_iso();
    conn.execute(
        "INSERT INTO shoot_batches (id, code, name, shoot_date, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 'planning', ?5)",
        rusqlite::params![id, code, name.trim(), shoot_date, created_at],
    )?;
    activity::log(conn, "batch", &code, "created", None)?;
    Ok(BatchSummary {
        id,
        code,
        name: name.trim().into(),
        shoot_date: shoot_date.map(str::to_string),
        status: "planning".into(),
        shots_total: 0,
        shots_recorded: 0,
        reel_count: 0,
        created_at,
    })
}

/// Add reels' script blocks to a batch as shots, deduplicating shared
/// components: five reels sharing BD0007 produce ONE shot for it.
/// Added reels in `scripted` advance to `shotlisted`.
pub fn add_reels(conn: &Connection, batch_id: &str, reel_ids: &[String]) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    let mut next_pos: i64 = tx.query_row(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM shots WHERE batch_id = ?1",
        [batch_id],
        |r| r.get(0),
    )?;

    for reel_id in reel_ids {
        let detail = reels::get_detail(&tx, reel_id)?;
        for block in &detail.blocks {
            if let Some(component_id) = &block.component_id {
                // shared material: one shot per component per batch
                let exists: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM shots WHERE batch_id = ?1 AND component_id = ?2)",
                    rusqlite::params![batch_id, component_id],
                    |r| r.get(0),
                )?;
                if exists {
                    continue;
                }
                // material already recorded in an earlier batch and selected? skip.
                let has_master: bool = tx.query_row(
                    "SELECT master_take_id IS NOT NULL FROM components WHERE id = ?1",
                    [component_id],
                    |r| r.get(0),
                )?;
                if has_master {
                    continue;
                }
                tx.execute(
                    "INSERT INTO shots (id, batch_id, position, component_id) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![ids::new_id(), batch_id, next_pos, component_id],
                )?;
                next_pos += 1;
            } else {
                let exists: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM shots WHERE batch_id = ?1 AND script_block_id = ?2)",
                    rusqlite::params![batch_id, block.id],
                    |r| r.get(0),
                )?;
                if exists {
                    continue;
                }
                tx.execute(
                    "INSERT INTO shots (id, batch_id, position, script_block_id) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![ids::new_id(), batch_id, next_pos, block.id],
                )?;
                next_pos += 1;
            }
        }
        if detail.status == "scripted" {
            reels::set_status(&tx, reel_id, "shotlisted", false)?;
        }
        // everything this reel needs may already be recorded (master takes)
        let detail = reels::get_detail(&tx, reel_id)?;
        if detail.status == "shotlisted" && !detail.blocks.is_empty() {
            let mut all = true;
            for b in &detail.blocks {
                if !block_covered(&tx, &b.id, b.component_id.as_deref())? {
                    all = false;
                    break;
                }
            }
            if all {
                reels::set_status(&tx, reel_id, "shot", false)?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn list(conn: &Connection) -> Result<Vec<BatchSummary>> {
    let mut stmt = conn.prepare(
        "SELECT b.id, b.code, b.name, b.shoot_date, b.status, b.created_at,
                (SELECT COUNT(*) FROM shots s WHERE s.batch_id = b.id),
                (SELECT COUNT(*) FROM shots s WHERE s.batch_id = b.id AND s.status = 'recorded')
         FROM shoot_batches b ORDER BY b.created_at DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(BatchSummary {
            id: r.get(0)?,
            code: r.get(1)?,
            name: r.get(2)?,
            shoot_date: r.get(3)?,
            status: r.get(4)?,
            created_at: r.get(5)?,
            shots_total: r.get(6)?,
            shots_recorded: r.get(7)?,
            reel_count: 0,
        })
    })?;
    let mut batches: Vec<BatchSummary> = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    for b in &mut batches {
        b.reel_count = reels_in_batch(conn, &b.id)?.len() as i64;
    }
    Ok(batches)
}

pub fn detail(conn: &Connection, id: &str) -> Result<BatchDetail> {
    let head = conn
        .query_row(
            "SELECT id, code, name, shoot_date, status, notes FROM shoot_batches WHERE id = ?1",
            [id],
            |r| {
                Ok(BatchDetail {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    name: r.get(2)?,
                    shoot_date: r.get(3)?,
                    status: r.get(4)?,
                    notes: r.get(5)?,
                    shots: Vec::new(),
                })
            },
        )
        .optional()?
        .ok_or_else(|| CoreError::NotFound(format!("batch {id}")))?;

    let mut stmt = conn.prepare(
        "SELECT s.id, s.position, s.status, s.selected_take_id,
                s.component_id, c.code, c.kind, c.text,
                sb.reel_id, sb.position, sb.kind, sb.text
         FROM shots s
         LEFT JOIN components c ON c.id = s.component_id
         LEFT JOIN script_blocks sb ON sb.id = s.script_block_id
         WHERE s.batch_id = ?1 ORDER BY s.position",
    )?;
    #[allow(clippy::type_complexity)]
    let raw: Vec<(String, i64, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>)> = stmt
        .query_map([id], |r| {
            Ok((
                r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?,
                r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?, r.get(10)?, r.get(11)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut shots = Vec::with_capacity(raw.len());
    for (shot_id, position, status, selected_take_id, component_id, c_code, c_kind, c_text, sb_reel, sb_pos, sb_kind, sb_text) in raw {
        let (clip_key, kind, text, used_by) = if let Some(component_id) = &component_id {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT r.code FROM script_blocks sb JOIN reels r ON r.id = sb.reel_id
                 WHERE sb.component_id = ?1 ORDER BY r.code",
            )?;
            let used: Vec<String> = stmt
                .query_map([component_id], |r| r.get(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            (
                c_code.unwrap_or_default(),
                c_kind.unwrap_or_default(),
                c_text.unwrap_or_default(),
                used,
            )
        } else {
            let reel_code: String = conn.query_row(
                "SELECT code FROM reels WHERE id = ?1",
                [sb_reel.as_deref().unwrap_or_default()],
                |r| r.get(0),
            )?;
            (
                format!("{reel_code}-S{:02}", sb_pos.unwrap_or(0) + 1),
                sb_kind.unwrap_or_default(),
                sb_text.unwrap_or_default(),
                vec![reel_code],
            )
        };

        let mut tstmt = conn.prepare(
            "SELECT t.id, t.take_number, t.asset_id, a.filename, t.rating
             FROM takes t JOIN assets a ON a.id = t.asset_id
             WHERE t.shot_id = ?1 ORDER BY t.take_number",
        )?;
        let takes: Vec<TakeView> = tstmt
            .query_map([&shot_id], |r| {
                let take_id: String = r.get(0)?;
                Ok(TakeView {
                    selected: Some(take_id.as_str()) == selected_take_id.as_deref(),
                    id: take_id,
                    take_number: r.get(1)?,
                    asset_id: r.get(2)?,
                    filename: r.get(3)?,
                    rating: r.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        shots.push(ShotView {
            id: shot_id,
            position,
            status,
            kind,
            clip_key,
            text,
            used_by,
            takes,
            selected_take_id,
        });
    }

    Ok(BatchDetail { shots, ..head })
}

pub fn set_status(conn: &Connection, id: &str, status: &str) -> Result<()> {
    if !matches!(status, "planning" | "ready" | "shooting" | "done") {
        return Err(CoreError::Invalid(format!("invalid batch status: {status}")));
    }
    let changed = conn.execute(
        "UPDATE shoot_batches SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("batch {id}")));
    }
    Ok(())
}

pub fn set_shot_status(conn: &Connection, shot_id: &str, status: &str) -> Result<()> {
    if !matches!(status, "pending" | "recorded" | "skipped") {
        return Err(CoreError::Invalid(format!("invalid shot status: {status}")));
    }
    let changed = conn.execute(
        "UPDATE shots SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, shot_id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("shot {shot_id}")));
    }
    Ok(())
}

/// Mark a take as the shot's selected take. Component shots also set the
/// component's master take (a component records once, reused everywhere).
/// Reels whose every block now has selected material advance to `shot`.
pub fn select_take(conn: &Connection, shot_id: &str, take_id: &str) -> Result<()> {
    let belongs: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM takes WHERE id = ?1 AND shot_id = ?2)",
        [take_id, shot_id],
        |r| r.get(0),
    )?;
    if !belongs {
        return Err(CoreError::Invalid("take does not belong to this shot".into()));
    }
    conn.execute(
        "UPDATE shots SET selected_take_id = ?1, status = 'recorded' WHERE id = ?2",
        [take_id, shot_id],
    )?;
    let component_id: Option<String> = conn.query_row(
        "SELECT component_id FROM shots WHERE id = ?1",
        [shot_id],
        |r| r.get(0),
    )?;
    if let Some(component_id) = &component_id {
        conn.execute(
            "UPDATE components SET master_take_id = ?1 WHERE id = ?2",
            [take_id, component_id],
        )?;
    }
    sync_covered_reels(conn, shot_id)?;
    Ok(())
}

pub fn set_take_rating(conn: &Connection, take_id: &str, rating: Option<i64>) -> Result<()> {
    if let Some(r) = rating {
        if !(1..=5).contains(&r) {
            return Err(CoreError::Invalid("rating must be 1–5".into()));
        }
    }
    let changed = conn.execute(
        "UPDATE takes SET rating = ?1 WHERE id = ?2",
        rusqlite::params![rating, take_id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("take {take_id}")));
    }
    Ok(())
}

/// Distinct reels whose material appears in this batch.
pub fn reels_in_batch(conn: &Connection, batch_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT r.id FROM shots s
         JOIN script_blocks sb ON (sb.component_id = s.component_id OR sb.id = s.script_block_id)
         JOIN reels r ON r.id = sb.reel_id
         WHERE s.batch_id = ?1",
    )?;
    let rows = stmt.query_map([batch_id], |r| r.get(0))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// A reel's block is covered when its component has a master take, or (own
/// text) some shot for that block has a selected take.
fn block_covered(conn: &Connection, block_id: &str, component_id: Option<&str>) -> Result<bool> {
    if let Some(component_id) = component_id {
        Ok(conn.query_row(
            "SELECT master_take_id IS NOT NULL FROM components WHERE id = ?1",
            [component_id],
            |r| r.get(0),
        )?)
    } else {
        Ok(conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM shots WHERE script_block_id = ?1 AND selected_take_id IS NOT NULL)",
            [block_id],
            |r| r.get(0),
        )?)
    }
}

fn sync_covered_reels(conn: &Connection, shot_id: &str) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT sb.reel_id FROM script_blocks sb
         JOIN shots s ON (sb.component_id = s.component_id OR sb.id = s.script_block_id)
         WHERE s.id = ?1",
    )?;
    let reel_ids: Vec<String> = stmt
        .query_map([shot_id], |r| r.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    for reel_id in reel_ids {
        let detail = reels::get_detail(conn, &reel_id)?;
        if detail.status != "shotlisted" || detail.blocks.is_empty() {
            continue;
        }
        let mut all = true;
        for b in &detail.blocks {
            if !block_covered(conn, &b.id, b.component_id.as_deref())? {
                all = false;
                break;
            }
        }
        if all {
            reels::set_status(conn, &reel_id, "shot", false)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components, library, roots, Db};

    fn fixture(db: &Db) -> (String, String, String) {
        // shared hook + shared body across two reels; reel B adds its own CTA
        let hook = components::create(&db.conn, "hook", "Hook A", &[], None).unwrap();
        let body = components::create(&db.conn, "body", "Body core", &[], None).unwrap();
        let a = reels::create(&db.conn, "Reel A", None).unwrap();
        let b = reels::create(&db.conn, "Reel B", None).unwrap();
        reels::set_blocks(&db.conn, &a.id, &[
            reels::BlockInput { kind: "hook".into(), component_id: Some(hook.id.clone()), text: None, est_seconds: None },
            reels::BlockInput { kind: "body".into(), component_id: Some(body.id.clone()), text: None, est_seconds: None },
        ]).unwrap();
        reels::set_blocks(&db.conn, &b.id, &[
            reels::BlockInput { kind: "hook".into(), component_id: Some(hook.id.clone()), text: None, est_seconds: None },
            reels::BlockInput { kind: "body".into(), component_id: Some(body.id.clone()), text: None, est_seconds: None },
            reels::BlockInput { kind: "cta".into(), component_id: None, text: Some("Follow me".into()), est_seconds: None },
        ]).unwrap();
        (a.id, b.id, hook.id)
    }

    #[test]
    fn dedup_shot_list() {
        let db = Db::open_in_memory().unwrap();
        let (a, b, _) = fixture(&db);
        let batch = create(&db.conn, "Studio day", None).unwrap();
        add_reels(&db.conn, &batch.id, &[a.clone(), b.clone()]).unwrap();

        let d = detail(&db.conn, &batch.id).unwrap();
        // 5 blocks across two reels collapse to 3 shots (hook, body shared; CTA own)
        assert_eq!(d.shots.len(), 3);
        let hook_shot = &d.shots[0];
        assert_eq!(hook_shot.used_by.len(), 2, "shared hook shows both reels");
        assert!(d.shots[2].clip_key.contains("-S03"), "standalone CTA keyed by reel-block");

        // both reels advanced to shotlisted
        assert_eq!(reels::get_detail(&db.conn, &a).unwrap().status, "shotlisted");
        assert_eq!(reels::get_detail(&db.conn, &b).unwrap().status, "shotlisted");

        // idempotent: re-adding creates nothing new
        add_reels(&db.conn, &batch.id, &[a, b]).unwrap();
        assert_eq!(detail(&db.conn, &batch.id).unwrap().shots.len(), 3);
    }

    #[test]
    fn takes_selection_advances_reels() {
        let db = Db::open_in_memory().unwrap();
        let (a, b, _) = fixture(&db);
        let batch = create(&db.conn, "Studio day", None).unwrap();
        add_reels(&db.conn, &batch.id, &[a.clone(), b.clone()]).unwrap();
        let d = detail(&db.conn, &batch.id).unwrap();

        // stage a root with fake clips and ingest one take per shot
        let dir = tempfile::tempdir().unwrap();
        let root = roots::add_root(&db.conn, "media", dir.path().to_str().unwrap(), "local").unwrap();
        std::fs::create_dir_all(dir.path().join("00_INBOX")).unwrap();
        for (i, shot) in d.shots.iter().enumerate() {
            let f = dir.path().join("00_INBOX").join(format!("clip{i}.mp4"));
            std::fs::write(&f, format!("fake video {i}")).unwrap();
            library::scan_inbox(&db.conn, &root.id).unwrap();
            let inbox = library::list_inbox(&db.conn, &root.id).unwrap();
            let asset = inbox.iter().find(|a| a.filename.contains(&format!("clip{i}"))).unwrap();
            let take = library::ingest_take(&db.conn, &shot.id, &asset.id, Some(4)).unwrap();
            select_take(&db.conn, &shot.id, &take.id).unwrap();
        }

        // every block covered → both reels advanced to `shot`
        assert_eq!(reels::get_detail(&db.conn, &a).unwrap().status, "shot");
        assert_eq!(reels::get_detail(&db.conn, &b).unwrap().status, "shot");

        // canonical rename happened: B001_<clipkey>_T01.mp4 under 01_RAW/B001/
        let d = detail(&db.conn, &batch.id).unwrap();
        let f = &d.shots[0].takes[0].filename;
        assert!(f.starts_with("B001_") && f.ends_with("_T01.mp4"), "got {f}");

        // a later batch skips material that already has a master take
        let c = reels::create(&db.conn, "Reel C", None).unwrap();
        let hook_id = db.conn.query_row::<String, _, _>(
            "SELECT id FROM components WHERE kind = 'hook'", [], |r| r.get(0)).unwrap();
        reels::set_blocks(&db.conn, &c.id, &[
            reels::BlockInput { kind: "hook".into(), component_id: Some(hook_id), text: None, est_seconds: None },
        ]).unwrap();
        let batch2 = create(&db.conn, "Pickups", None).unwrap();
        add_reels(&db.conn, &batch2.id, &[c.id.clone()]).unwrap();
        assert_eq!(detail(&db.conn, &batch2.id).unwrap().shots.len(), 0, "hook already mastered");
    }
}
