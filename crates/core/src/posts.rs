use std::path::Path;

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::{activity, ids, reels, CoreError, Result};

pub const PUBLISH_DIR: &str = "06_PUBLISH";
pub const PLATFORMS: &[&str] = &["instagram", "tiktok", "youtube"];
const POST_STATUSES: &[&str] = &[
    "draft", "queued", "pushed", "scheduled", "published", "verified", "failed", "canceled",
];

fn platform_short(platform: &str) -> &'static str {
    match platform {
        "instagram" => "IG",
        "tiktok" => "TT",
        "youtube" => "YT",
        _ => "??",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostView {
    pub id: String,
    pub reel_id: String,
    pub reel_code: String,
    pub reel_title: String,
    pub reel_status: String,
    pub target_date: Option<String>,
    pub platform: String,
    pub caption: Option<String>,
    pub hashtags: Vec<String>,
    pub scheduled_at: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub final_filename: Option<String>,
}

/// Create missing draft posts for a reel on the given platforms (idempotent).
pub fn ensure_posts(conn: &Connection, reel_id: &str, platforms: &[String]) -> Result<()> {
    reels::get_detail(conn, reel_id)?; // 404 check
    for platform in platforms {
        if !PLATFORMS.contains(&platform.as_str()) {
            return Err(CoreError::Invalid(format!("unknown platform: {platform}")));
        }
        conn.execute(
            "INSERT INTO posts (id, reel_id, platform) VALUES (?1, ?2, ?3)
             ON CONFLICT(reel_id, platform) DO NOTHING",
            rusqlite::params![ids::new_id(), reel_id, platform],
        )?;
    }
    Ok(())
}

pub fn list_posts(conn: &Connection, status: Option<&str>) -> Result<Vec<PostView>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.reel_id, r.code, r.title, r.status, r.target_date,
                p.platform, p.caption, p.hashtags, p.scheduled_at, p.status, p.error,
                a.filename
         FROM posts p
         JOIN reels r ON r.id = p.reel_id
         LEFT JOIN assets a ON a.id = r.final_asset_id
         WHERE (?1 IS NULL OR p.status = ?1)
         ORDER BY p.scheduled_at IS NULL DESC, p.scheduled_at, r.code, p.platform",
    )?;
    let rows = stmt.query_map([status], |r| {
        let hashtags: Option<String> = r.get(8)?;
        Ok(PostView {
            id: r.get(0)?,
            reel_id: r.get(1)?,
            reel_code: r.get(2)?,
            reel_title: r.get(3)?,
            reel_status: r.get(4)?,
            target_date: r.get(5)?,
            platform: r.get(6)?,
            caption: r.get(7)?,
            hashtags: hashtags
                .and_then(|h| serde_json::from_str(&h).ok())
                .unwrap_or_default(),
            scheduled_at: r.get(9)?,
            status: r.get(10)?,
            error: r.get(11)?,
            final_filename: r.get(12)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn update_post(
    conn: &Connection,
    id: &str,
    caption: Option<&str>,
    hashtags: &[String],
    scheduled_at: Option<&str>,
) -> Result<()> {
    if let Some(s) = scheduled_at {
        // naive local datetime "YYYY-MM-DDTHH:MM"
        if chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").is_err() {
            return Err(CoreError::Invalid(format!(
                "invalid schedule time (want YYYY-MM-DDTHH:MM): {s}"
            )));
        }
    }
    let changed = conn.execute(
        "UPDATE posts SET caption = ?1, hashtags = ?2, scheduled_at = ?3 WHERE id = ?4",
        rusqlite::params![caption, serde_json::to_string(hashtags).unwrap(), scheduled_at, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("post {id}")));
    }
    Ok(())
}

/// Manual status moves (the tracking board). Also syncs the reel's status.
pub fn set_post_status(conn: &Connection, id: &str, status: &str, error: Option<&str>) -> Result<()> {
    if !POST_STATUSES.contains(&status) {
        return Err(CoreError::Invalid(format!("invalid post status: {status}")));
    }
    let ts_col = match status {
        "published" => "published_at",
        "verified" => "verified_at",
        "pushed" | "scheduled" => "pushed_at",
        _ => "",
    };
    let sql = if ts_col.is_empty() {
        "UPDATE posts SET status = ?1, error = ?2 WHERE id = ?3".to_string()
    } else {
        format!("UPDATE posts SET status = ?1, error = ?2, {ts_col} = '{}' WHERE id = ?3", ids::now_iso())
    };
    let changed = conn.execute(&sql, rusqlite::params![status, error, id])?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("post {id}")));
    }
    let reel_id: String =
        conn.query_row("SELECT reel_id FROM posts WHERE id = ?1", [id], |r| r.get(0))?;
    sync_reel(conn, &reel_id)
}

/// Assign schedule times to unscheduled draft/queued posts from their reel's
/// calendar target date + per-platform default times.
pub fn bulk_fill_schedule(
    conn: &Connection,
    times: &std::collections::HashMap<String, String>,
) -> Result<i64> {
    let posts = list_posts(conn, None)?;
    let mut filled = 0;
    for p in posts {
        if !matches!(p.status.as_str(), "draft" | "queued") || p.scheduled_at.is_some() {
            continue;
        }
        let Some(date) = &p.target_date else { continue };
        let Some(time) = times.get(&p.platform) else { continue };
        let scheduled = format!("{date}T{time}");
        conn.execute(
            "UPDATE posts SET scheduled_at = ?1 WHERE id = ?2",
            rusqlite::params![scheduled, p.id],
        )?;
        filled += 1;
    }
    Ok(filled)
}

/// Forward-only reel status sync from its posts' states.
fn sync_reel(conn: &Connection, reel_id: &str) -> Result<()> {
    let detail = reels::get_detail(conn, reel_id)?;
    let order = |s: &str| reels::STATUS_ORDER.iter().position(|x| *x == s);
    let Some(current) = order(&detail.status) else { return Ok(()) }; // archived/killed: leave
    let statuses: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT status FROM posts WHERE reel_id = ?1 AND status != 'canceled'",
        )?;
        let rows = stmt.query_map([reel_id], |r| r.get(0))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()?
    };
    if statuses.is_empty() {
        return Ok(());
    }
    let all = |s: &str| statuses.iter().all(|x| x == s);
    let scheduled_or_later = |x: &String| {
        matches!(x.as_str(), "pushed" | "scheduled" | "published" | "verified")
    };
    let target = if all("verified") {
        "verified"
    } else if statuses.iter().any(|x| matches!(x.as_str(), "published" | "verified")) {
        "posted"
    } else if statuses.iter().all(scheduled_or_later) {
        "scheduled"
    } else {
        return Ok(());
    };
    if order(target).unwrap() > current {
        reels::set_status(conn, reel_id, target, true)?;
    }
    Ok(())
}

// ── publishing providers ─────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ExportBundle {
    pub folder: String,
    pub csv_path: String,
    pub exported: Vec<String>, // "R0142 · instagram"
    pub skipped: Vec<String>,  // human reasons
}

fn csv_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

/// The CSV publishing provider (Metricool free plan): writes a bulk-import
/// CSV plus a folder of the matching finals (named by slot) for manual
/// attachment in Metricool's planner. Exported posts move to `scheduled`.
///
/// The Metricool API provider slots in beside this one when the plan
/// upgrade lands (same inputs, same status flow).
pub fn export_bundle(conn: &Connection, root_id: &str, post_ids: &[String]) -> Result<ExportBundle> {
    let root: String = conn
        .query_row("SELECT path FROM storage_roots WHERE id = ?1", [root_id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| CoreError::NotFound(format!("storage root {root_id}")))?;

    let stamp = ids::now_iso().replace(':', "").replace('-', "");
    let folder = Path::new(&root)
        .join(PUBLISH_DIR)
        .join(format!("bundle_{}", &stamp[..13])); // YYYYMMDDTHHMM
    std::fs::create_dir_all(&folder)?;

    let all = list_posts(conn, None)?;
    let mut bundle = ExportBundle {
        folder: folder.to_string_lossy().into_owned(),
        csv_path: String::new(),
        exported: Vec::new(),
        skipped: Vec::new(),
    };

    let mut csv = String::from("Text,Date,Time,Draft,Instagram,TikTok,YouTube\n");
    for id in post_ids {
        let Some(p) = all.iter().find(|p| &p.id == id) else {
            bundle.skipped.push(format!("post {id} not found"));
            continue;
        };
        let label = format!("{} · {}", p.reel_code, p.platform);
        let Some(scheduled) = &p.scheduled_at else {
            bundle.skipped.push(format!("{label} — no schedule time set"));
            continue;
        };
        let Some(caption) = p.caption.as_deref().filter(|c| !c.trim().is_empty()) else {
            bundle.skipped.push(format!("{label} — caption is empty"));
            continue;
        };

        // final render lookup
        let final_asset: Option<(String, String)> = conn
            .query_row(
                "SELECT a.root_id, a.rel_path FROM reels r JOIN assets a ON a.id = r.final_asset_id
                 WHERE r.id = ?1",
                [&p.reel_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((a_root, a_rel)) = final_asset else {
            bundle.skipped.push(format!("{label} — reel has no final render yet"));
            continue;
        };
        let a_root_path: String =
            conn.query_row("SELECT path FROM storage_roots WHERE id = ?1", [&a_root], |r| r.get(0))?;
        let src = Path::new(&a_root_path).join(&a_rel);
        if !src.is_file() {
            bundle.skipped.push(format!("{label} — final file missing on disk ({a_rel})"));
            continue;
        }

        let (date, time) = scheduled.split_once('T').unwrap_or((scheduled.as_str(), "00:00"));
        let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
        let staged_name = format!(
            "{}_{}_{}_{}.{ext}",
            date,
            time.replace(':', ""),
            platform_short(&p.platform),
            p.reel_code
        );
        let dest = folder.join(&staged_name);
        if !dest.exists() && std::fs::hard_link(&src, &dest).is_err() {
            std::fs::copy(&src, &dest)?;
        }

        let mut text = caption.to_string();
        if !p.hashtags.is_empty() {
            text.push_str("\n\n");
            text.push_str(&p.hashtags.iter().map(|h| {
                if h.starts_with('#') { h.clone() } else { format!("#{h}") }
            }).collect::<Vec<_>>().join(" "));
        }
        csv.push_str(&format!(
            "{},{date},{time},FALSE,{},{},{}\n",
            csv_quote(&text),
            if p.platform == "instagram" { "TRUE" } else { "FALSE" },
            if p.platform == "tiktok" { "TRUE" } else { "FALSE" },
            if p.platform == "youtube" { "TRUE" } else { "FALSE" },
        ));

        set_post_status(conn, &p.id, "scheduled", None)?;
        bundle.exported.push(label);
    }

    if bundle.exported.is_empty() {
        // nothing made it — remove the empty folder, surface reasons
        let _ = std::fs::remove_dir_all(&folder);
        return Err(CoreError::Invalid(format!(
            "nothing exported: {}",
            bundle.skipped.join("; ")
        )));
    }

    let csv_path = folder.join("metricool_import.csv");
    std::fs::write(&csv_path, csv)?;
    bundle.csv_path = csv_path.to_string_lossy().into_owned();
    activity::log(
        conn,
        "publish",
        &stamp[..13],
        "csv-bundle-exported",
        Some(&serde_json::json!({ "posts": bundle.exported.len() })),
    )?;
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{batches, components, davinci, library, roots, Db};
    use std::collections::HashMap;

    /// Full pipeline to an `edited` reel with a final on disk.
    fn edited_fixture(db: &Db, dir: &Path) -> (String, String) {
        let hook = components::create(&db.conn, "hook", "Hook line", &[], None).unwrap();
        let reel = reels::create(&db.conn, "Gym myths #1", None).unwrap();
        reels::set_blocks(&db.conn, &reel.id, &[reels::BlockInput {
            kind: "hook".into(), component_id: Some(hook.id), text: None, est_seconds: None,
        }]).unwrap();
        let root = roots::add_root(&db.conn, "media", dir.to_str().unwrap(), "local").unwrap();
        let batch = batches::create(&db.conn, "Day", None).unwrap();
        batches::add_reels(&db.conn, &batch.id, &[reel.id.clone()]).unwrap();
        std::fs::create_dir_all(dir.join(library::INBOX_DIR)).unwrap();
        std::fs::write(dir.join(library::INBOX_DIR).join("c.mp4"), "clip").unwrap();
        library::scan_inbox(&db.conn, &root.id).unwrap();
        let asset = library::list_inbox(&db.conn, &root.id).unwrap().remove(0);
        let detail = batches::detail(&db.conn, &batch.id).unwrap();
        let take = library::ingest_take(&db.conn, &detail.shots[0].id, &asset.id, None).unwrap();
        batches::select_take(&db.conn, &detail.shots[0].id, &take.id).unwrap();
        davinci::generate_handoff(&db.conn, &batch.id).unwrap();
        std::fs::create_dir_all(dir.join(davinci::EXPORTS_DIR)).unwrap();
        std::fs::write(dir.join(davinci::EXPORTS_DIR).join("R0001_final.mp4"), "render").unwrap();
        let scan = davinci::scan_exports(&db.conn, &root.id).unwrap();
        davinci::confirm_export(&db.conn, &root.id, &scan.matches[0].rel_path, &scan.matches[0].reel_id).unwrap();
        (reel.id, root.id)
    }

    #[test]
    fn posts_lifecycle_and_reel_sync() {
        let db = Db::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let (reel_id, _) = edited_fixture(&db, dir.path());

        ensure_posts(&db.conn, &reel_id, &["instagram".into(), "tiktok".into()]).unwrap();
        ensure_posts(&db.conn, &reel_id, &["instagram".into()]).unwrap(); // idempotent
        let posts = list_posts(&db.conn, Some("draft")).unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].final_filename.as_deref(), Some("R0001_final.mp4"));

        // bulk fill from target date
        reels::set_target_date(&db.conn, &reel_id, Some("2026-07-20")).unwrap();
        let times = HashMap::from([
            ("instagram".to_string(), "09:00".to_string()),
            ("tiktok".to_string(), "12:30".to_string()),
        ]);
        assert_eq!(bulk_fill_schedule(&db.conn, &times).unwrap(), 2);
        let posts = list_posts(&db.conn, None).unwrap();
        assert!(posts.iter().any(|p| p.scheduled_at.as_deref() == Some("2026-07-20T09:00")));

        // status sync: both scheduled → reel scheduled; one published → posted; all verified → verified
        for p in &posts {
            set_post_status(&db.conn, &p.id, "scheduled", None).unwrap();
        }
        assert_eq!(reels::get_detail(&db.conn, &reel_id).unwrap().status, "scheduled");
        set_post_status(&db.conn, &posts[0].id, "published", None).unwrap();
        assert_eq!(reels::get_detail(&db.conn, &reel_id).unwrap().status, "posted");
        set_post_status(&db.conn, &posts[0].id, "verified", None).unwrap();
        set_post_status(&db.conn, &posts[1].id, "verified", None).unwrap();
        assert_eq!(reels::get_detail(&db.conn, &reel_id).unwrap().status, "verified");
    }

    #[test]
    fn csv_bundle_export() {
        let db = Db::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let (reel_id, root_id) = edited_fixture(&db, dir.path());

        ensure_posts(&db.conn, &reel_id, &["instagram".into(), "youtube".into()]).unwrap();
        let posts = list_posts(&db.conn, None).unwrap();
        // caption+time on IG; YT left incomplete on purpose
        let ig = posts.iter().find(|p| p.platform == "instagram").unwrap();
        let yt = posts.iter().find(|p| p.platform == "youtube").unwrap();
        update_post(&db.conn, &ig.id, Some("Stop believing this gym myth."),
            &["gym".into(), "#fitness".into()], Some("2026-07-20T09:00")).unwrap();

        let bundle = export_bundle(&db.conn, &root_id, &[ig.id.clone(), yt.id.clone()]).unwrap();
        assert_eq!(bundle.exported, vec!["R0001 · instagram"]);
        assert_eq!(bundle.skipped.len(), 1, "youtube skipped with reason");

        let csv = std::fs::read_to_string(&bundle.csv_path).unwrap();
        assert!(csv.contains("2026-07-20,09:00,FALSE,TRUE,FALSE,FALSE"));
        assert!(csv.contains("#gym #fitness"));
        let staged = Path::new(&bundle.folder).join("2026-07-20_0900_IG_R0001.mp4");
        assert!(staged.is_file());

        // exported post is now scheduled
        let ig_after = list_posts(&db.conn, Some("scheduled")).unwrap();
        assert_eq!(ig_after.len(), 1);

        // exporting nothing valid errors with reasons
        assert!(export_bundle(&db.conn, &root_id, &[yt.id.clone()]).is_err());
    }
}
