use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::{activity, ids, search, CoreError, Result};

pub const STATUS_ORDER: &[&str] = &[
    "idea", "scripted", "shotlisted", "shot", "assembled", "editing", "edited",
    "scheduled", "posted", "verified",
];
const TERMINAL: &[&str] = &["archived", "killed"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReelSummary {
    pub id: String,
    pub code: String,
    pub title: String,
    pub slug: String,
    pub pillar_id: Option<String>,
    pub status: String,
    pub target_date: Option<String>,
    pub block_count: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInput {
    pub kind: String,
    pub component_id: Option<String>,
    pub text: Option<String>,
    pub est_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockView {
    pub id: String,
    pub position: i64,
    pub kind: String,
    pub component_id: Option<String>,
    pub component_code: Option<String>,
    /// Resolved script text: the component's text when linked, else the block's own.
    pub text: String,
    pub est_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReelDetail {
    pub id: String,
    pub code: String,
    pub title: String,
    pub slug: String,
    pub pillar_id: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub target_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub blocks: Vec<BlockView>,
}

pub fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = true;
    for ch in title.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() { "reel".into() } else { slug }
}

pub fn create(conn: &Connection, title: &str, pillar_id: Option<&str>) -> Result<ReelDetail> {
    if title.trim().is_empty() {
        return Err(CoreError::Invalid("reel title is required".into()));
    }
    let title = title.trim();
    let code = ids::next_code(conn, "R", 4)?;
    let now = ids::now_iso();
    let id = ids::new_id();
    conn.execute(
        "INSERT INTO reels (id, code, title, slug, pillar_id, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'idea', ?6, ?6)",
        rusqlite::params![id, code, title, slugify(title), pillar_id, now],
    )?;
    activity::log(conn, "reel", &code, "created", None)?;
    reindex(conn, &id)?;
    get_detail(conn, &id)
}

pub fn update_meta(
    conn: &Connection,
    id: &str,
    title: &str,
    notes: Option<&str>,
    pillar_id: Option<&str>,
) -> Result<()> {
    if title.trim().is_empty() {
        return Err(CoreError::Invalid("reel title is required".into()));
    }
    let title = title.trim();
    let changed = conn.execute(
        "UPDATE reels SET title = ?1, slug = ?2, notes = ?3, pillar_id = ?4, updated_at = ?5
         WHERE id = ?6",
        rusqlite::params![title, slugify(title), notes, pillar_id, ids::now_iso(), id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("reel {id}")));
    }
    reindex(conn, id)
}

/// Replace the reel's script blocks (ordered). A reel still in `idea` with a
/// non-empty script auto-advances to `scripted`.
pub fn set_blocks(conn: &Connection, reel_id: &str, blocks: &[BlockInput]) -> Result<()> {
    for b in blocks {
        if !matches!(b.kind.as_str(), "hook" | "body" | "cta" | "segment") {
            return Err(CoreError::Invalid(format!("invalid block kind: {}", b.kind)));
        }
        let has_component = b.component_id.as_deref().is_some_and(|c| !c.is_empty());
        let has_text = b.text.as_deref().is_some_and(|t| !t.trim().is_empty());
        if !has_component && !has_text {
            return Err(CoreError::Invalid(
                "every block needs either linked component or its own text".into(),
            ));
        }
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM script_blocks WHERE reel_id = ?1", [reel_id])?;
    for (position, b) in blocks.iter().enumerate() {
        let component_id = b.component_id.as_deref().filter(|c| !c.is_empty());
        let own_text = if component_id.is_some() {
            None
        } else {
            b.text.as_deref().map(str::trim)
        };
        tx.execute(
            "INSERT INTO script_blocks (id, reel_id, position, kind, component_id, text, est_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                ids::new_id(),
                reel_id,
                position as i64,
                b.kind,
                component_id,
                own_text,
                b.est_seconds
            ],
        )?;
    }
    tx.execute(
        "UPDATE reels SET updated_at = ?1,
           status = CASE WHEN status = 'idea' AND ?2 > 0 THEN 'scripted' ELSE status END
         WHERE id = ?3",
        rusqlite::params![ids::now_iso(), blocks.len() as i64, reel_id],
    )?;
    tx.commit()?;
    reindex(conn, reel_id)
}

/// Legal transitions without `overrule`: one step forward/back in the pipeline,
/// into archived/killed from anywhere, or out of archived/killed to anywhere.
pub fn set_status(conn: &Connection, id: &str, new_status: &str, overrule: bool) -> Result<()> {
    let all_valid = STATUS_ORDER.iter().chain(TERMINAL.iter());
    if !all_valid.clone().any(|s| *s == new_status) {
        return Err(CoreError::Invalid(format!("invalid status: {new_status}")));
    }
    let current: String = conn
        .query_row("SELECT status FROM reels WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| CoreError::NotFound(format!("reel {id}")))?;
    if current == new_status {
        return Ok(());
    }

    if !overrule {
        let legal = if TERMINAL.contains(&new_status) || TERMINAL.contains(&current.as_str()) {
            true
        } else {
            let from = STATUS_ORDER.iter().position(|s| *s == current.as_str());
            let to = STATUS_ORDER.iter().position(|s| *s == new_status);
            match (from, to) {
                (Some(f), Some(t)) => f.abs_diff(t) == 1,
                _ => false,
            }
        };
        if !legal {
            return Err(CoreError::Invalid(format!(
                "moving {current} → {new_status} skips pipeline stages; pass overrule to force it"
            )));
        }
    }

    conn.execute(
        "UPDATE reels SET status = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_status, ids::now_iso(), id],
    )?;
    let code: String = conn.query_row("SELECT code FROM reels WHERE id = ?1", [id], |r| r.get(0))?;
    activity::log(
        conn,
        "reel",
        &code,
        &format!("status:{current}→{new_status}"),
        overrule.then(|| serde_json::json!({ "overrule": true })).as_ref(),
    )?;
    Ok(())
}

pub fn set_target_date(conn: &Connection, id: &str, date: Option<&str>) -> Result<()> {
    if let Some(d) = date {
        let strict = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map(|parsed| parsed.format("%Y-%m-%d").to_string() == d)
            .unwrap_or(false);
        if !strict {
            return Err(CoreError::Invalid(format!("invalid date (want YYYY-MM-DD): {d}")));
        }
    }
    let changed = conn.execute(
        "UPDATE reels SET target_date = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![date, ids::now_iso(), id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("reel {id}")));
    }
    Ok(())
}

pub fn list(
    conn: &Connection,
    status: Option<&str>,
    pillar_id: Option<&str>,
    query: Option<&str>,
) -> Result<Vec<ReelSummary>> {
    let match_ids: Option<Vec<String>> = match query {
        Some(q) if !q.trim().is_empty() => Some(search::matching_ids(conn, "reel", q)?),
        _ => None,
    };

    let mut stmt = conn.prepare(
        "SELECT r.id, r.code, r.title, r.slug, r.pillar_id, r.status, r.target_date, r.updated_at,
                (SELECT COUNT(*) FROM script_blocks sb WHERE sb.reel_id = r.id)
         FROM reels r
         WHERE (?1 IS NULL OR r.status = ?1)
           AND (?2 IS NULL OR r.pillar_id = ?2)
           AND (?1 IS NOT NULL OR r.status NOT IN ('archived','killed'))
         ORDER BY r.updated_at DESC",
    )?;
    let rows = stmt.query_map(rusqlite::params![status, pillar_id], |r| {
        Ok(ReelSummary {
            id: r.get(0)?,
            code: r.get(1)?,
            title: r.get(2)?,
            slug: r.get(3)?,
            pillar_id: r.get(4)?,
            status: r.get(5)?,
            target_date: r.get(6)?,
            updated_at: r.get(7)?,
            block_count: r.get(8)?,
        })
    })?;
    let mut all: Vec<ReelSummary> = rows.collect::<std::result::Result<Vec<_>, _>>()?;

    if let Some(ids) = match_ids {
        let rank: std::collections::HashMap<&str, usize> =
            ids.iter().enumerate().map(|(i, id)| (id.as_str(), i)).collect();
        all.retain(|r| rank.contains_key(r.id.as_str()));
        all.sort_by_key(|r| rank[r.id.as_str()]);
    }
    Ok(all)
}

pub fn get_detail(conn: &Connection, id: &str) -> Result<ReelDetail> {
    let reel = conn
        .query_row(
            "SELECT id, code, title, slug, pillar_id, status, notes, target_date, created_at, updated_at
             FROM reels WHERE id = ?1",
            [id],
            |r| {
                Ok(ReelDetail {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    title: r.get(2)?,
                    slug: r.get(3)?,
                    pillar_id: r.get(4)?,
                    status: r.get(5)?,
                    notes: r.get(6)?,
                    target_date: r.get(7)?,
                    created_at: r.get(8)?,
                    updated_at: r.get(9)?,
                    blocks: Vec::new(),
                })
            },
        )
        .optional()?
        .ok_or_else(|| CoreError::NotFound(format!("reel {id}")))?;

    let mut stmt = conn.prepare(
        "SELECT sb.id, sb.position, sb.kind, sb.component_id, c.code,
                COALESCE(c.text, sb.text, ''), sb.est_seconds
         FROM script_blocks sb
         LEFT JOIN components c ON c.id = sb.component_id
         WHERE sb.reel_id = ?1 ORDER BY sb.position",
    )?;
    let blocks = stmt
        .query_map([id], |r| {
            Ok(BlockView {
                id: r.get(0)?,
                position: r.get(1)?,
                kind: r.get(2)?,
                component_id: r.get(3)?,
                component_code: r.get(4)?,
                text: r.get(5)?,
                est_seconds: r.get(6)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(ReelDetail { blocks, ..reel })
}

#[derive(Debug, Serialize)]
pub struct CalendarReel {
    pub id: String,
    pub code: String,
    pub title: String,
    pub status: String,
    pub pillar_id: Option<String>,
    pub target_date: String,
}

/// Reels with a target date inside [start, end] (inclusive, YYYY-MM-DD).
pub fn calendar_range(conn: &Connection, start: &str, end: &str) -> Result<Vec<CalendarReel>> {
    let mut stmt = conn.prepare(
        "SELECT id, code, title, status, pillar_id, target_date
         FROM reels
         WHERE target_date >= ?1 AND target_date <= ?2 AND status NOT IN ('archived','killed')
         ORDER BY target_date, code",
    )?;
    let rows = stmt.query_map([start, end], |r| {
        Ok(CalendarReel {
            id: r.get(0)?,
            code: r.get(1)?,
            title: r.get(2)?,
            status: r.get(3)?,
            pillar_id: r.get(4)?,
            target_date: r.get(5)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Reels sitting mid-pipeline with no movement for `days` — the "falling
/// through the cracks" detector. Ideas and terminal states don't count.
pub fn stuck(conn: &Connection, days: i64) -> Result<Vec<ReelSummary>> {
    let cutoff = (chrono::Local::now() - chrono::Duration::days(days))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, false);
    let mut all = list(conn, None, None, None)?;
    all.retain(|r| {
        matches!(
            r.status.as_str(),
            "scripted" | "shotlisted" | "shot" | "assembled" | "editing" | "edited"
        ) && r.updated_at.as_str() < cutoff.as_str()
    });
    all.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
    Ok(all)
}

/// Distinct reels with a post scheduled inside the current Mon–Sun week.
pub fn scheduled_this_week(conn: &Connection) -> Result<i64> {
    use chrono::Datelike;
    let today = chrono::Local::now().date_naive();
    let monday = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let next_monday = monday + chrono::Duration::days(7);
    Ok(conn.query_row(
        "SELECT COUNT(DISTINCT reel_id) FROM posts
         WHERE scheduled_at >= ?1 AND scheduled_at < ?2 AND status != 'canceled'",
        rusqlite::params![monday.format("%Y-%m-%d").to_string(), next_monday.format("%Y-%m-%d").to_string()],
        |r| r.get(0),
    )?)
}

/// Active reels with no target date yet — the calendar's "to plan" tray.
pub fn unscheduled(conn: &Connection) -> Result<Vec<ReelSummary>> {
    let mut all = list(conn, None, None, None)?;
    all.retain(|r| r.target_date.is_none() && !matches!(r.status.as_str(), "posted" | "verified"));
    Ok(all)
}

/// Rebuild a reel's search-index row: title + notes + resolved block text.
fn reindex(conn: &Connection, id: &str) -> Result<()> {
    let detail = get_detail(conn, id)?;
    let mut content = detail.title.clone();
    if let Some(notes) = &detail.notes {
        content.push(' ');
        content.push_str(notes);
    }
    for b in &detail.blocks {
        content.push(' ');
        content.push_str(&b.text);
    }
    search::upsert(conn, "reel", id, &content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components, Db};

    #[test]
    fn slugs() {
        assert_eq!(slugify("Gym Myths #3 — Cardio!?"), "gym-myths-3-cardio");
        assert_eq!(slugify("???"), "reel");
    }

    #[test]
    fn create_script_and_autostatus() {
        let db = Db::open_in_memory().unwrap();
        let hook = components::create(&db.conn, "hook", "Stop doing cardio first", &[], None).unwrap();

        let reel = create(&db.conn, "Gym myths #1", None).unwrap();
        assert_eq!(reel.code, "R0001");
        assert_eq!(reel.status, "idea");

        set_blocks(
            &db.conn,
            &reel.id,
            &[
                BlockInput { kind: "hook".into(), component_id: Some(hook.id.clone()), text: None, est_seconds: None },
                BlockInput { kind: "body".into(), component_id: None, text: Some("The real reason is…".into()), est_seconds: Some(25) },
                BlockInput { kind: "cta".into(), component_id: None, text: Some("Follow for part 2".into()), est_seconds: Some(3) },
            ],
        )
        .unwrap();

        let detail = get_detail(&db.conn, &reel.id).unwrap();
        assert_eq!(detail.status, "scripted", "idea auto-advances once scripted");
        assert_eq!(detail.blocks.len(), 3);
        assert_eq!(detail.blocks[0].text, "Stop doing cardio first", "component text resolves");
        assert_eq!(detail.blocks[0].component_code.as_deref(), Some("HK0001"));

        // reel is findable by its linked component's text
        let found = list(&db.conn, None, None, Some("cardio")).unwrap();
        assert_eq!(found.len(), 1);

        // component usage count reflects the link
        let comps = components::list(&db.conn, Some("hook"), false, None).unwrap();
        assert_eq!(comps[0].times_used, 1);
    }

    #[test]
    fn status_transitions_enforced() {
        let db = Db::open_in_memory().unwrap();
        let reel = create(&db.conn, "Test", None).unwrap();

        // skipping stages is refused…
        assert!(set_status(&db.conn, &reel.id, "edited", false).is_err());
        // …but adjacent moves, terminal moves, and overrules work
        set_status(&db.conn, &reel.id, "scripted", false).unwrap();
        set_status(&db.conn, &reel.id, "idea", false).unwrap();
        set_status(&db.conn, &reel.id, "killed", false).unwrap();
        set_status(&db.conn, &reel.id, "idea", false).unwrap(); // restore from terminal
        set_status(&db.conn, &reel.id, "edited", true).unwrap();
        assert!(set_status(&db.conn, &reel.id, "banana", true).is_err());
    }

    #[test]
    fn calendar_and_unscheduled() {
        let db = Db::open_in_memory().unwrap();
        let a = create(&db.conn, "Planned", None).unwrap();
        let b = create(&db.conn, "Unplanned", None).unwrap();

        assert!(set_target_date(&db.conn, &a.id, Some("2026-7-20")).is_err());
        set_target_date(&db.conn, &a.id, Some("2026-07-20")).unwrap();

        let week = calendar_range(&db.conn, "2026-07-20", "2026-07-26").unwrap();
        assert_eq!(week.len(), 1);
        assert_eq!(week[0].code, "R0001");

        let tray = unscheduled(&db.conn).unwrap();
        assert_eq!(tray.len(), 1);
        assert_eq!(tray[0].id, b.id);

        set_target_date(&db.conn, &a.id, None).unwrap();
        assert_eq!(unscheduled(&db.conn).unwrap().len(), 2);
    }
}
