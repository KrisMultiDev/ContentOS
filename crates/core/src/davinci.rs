use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use crate::{activity, batches, ids, media, reels, CoreError, Result};

pub const HANDOFF_DIR: &str = "02_HANDOFF";
pub const EXPORTS_DIR: &str = "03_EXPORTS";
pub const FINALS_DIR: &str = "04_FINALS";

const TIMELINE_FPS: i64 = 30;
/// Laid-out length for clips whose real duration is unknown (no ffprobe).
/// The editor trims every clip anyway; a too-long clip is visible, a missing
/// one is not.
const ASSUMED_CLIP_SECONDS: i64 = 60;
const FINAL_EXTS: &[&str] = &["mp4", "mov"];

#[derive(Debug, Serialize)]
pub struct HandoffReel {
    pub reel_id: String,
    pub code: String,
    pub title: String,
    pub folder: String,
    pub clips: i64,
}

#[derive(Debug, Serialize)]
pub struct HandoffReport {
    pub batch_code: String,
    pub handoff_path: String,
    pub fcpxml_path: String,
    pub staged: Vec<HandoffReel>,
    pub skipped: Vec<String>,
    pub warnings: Vec<String>,
}

struct StagedClip {
    ordinal: usize,
    kind: String,
    clip_key: String,
    src: PathBuf,
    dest_name: String,
    duration_ms: Option<i64>,
}

/// Stage a batch for DaVinci: per-reel folders of renamed selected takes,
/// a script text per reel, and one FCPXML with a ready timeline per reel.
/// Reels in `shot` advance to `assembled`.
pub fn generate_handoff(conn: &Connection, batch_id: &str) -> Result<HandoffReport> {
    let batch = batches::detail(conn, batch_id)?;
    let reel_ids = batches::reels_in_batch(conn, batch_id)?;
    if reel_ids.is_empty() {
        return Err(CoreError::Invalid(
            "this batch has no reels — add reels to its shot list first".into(),
        ));
    }

    let mut report = HandoffReport {
        batch_code: batch.code.clone(),
        handoff_path: String::new(),
        fcpxml_path: String::new(),
        staged: Vec::new(),
        skipped: Vec::new(),
        warnings: Vec::new(),
    };

    let mut root: Option<(String, PathBuf)> = None; // (root_id, path)
    let mut reel_clips: Vec<(reels::ReelDetail, Vec<StagedClip>)> = Vec::new();

    for reel_id in &reel_ids {
        let detail = reels::get_detail(conn, reel_id)?;
        if !matches!(detail.status.as_str(), "shot" | "assembled") {
            report
                .skipped
                .push(format!("{} — status {}, needs selected takes for every block", detail.code, detail.status));
            continue;
        }

        let mut clips = Vec::new();
        let mut complete = true;
        for (ordinal, block) in detail.blocks.iter().enumerate() {
            match resolve_block_take(conn, block)? {
                Some((asset_root_id, rel_path, filename, duration_ms, clip_key)) => {
                    if root.is_none() {
                        let path: String = conn.query_row(
                            "SELECT path FROM storage_roots WHERE id = ?1",
                            [&asset_root_id],
                            |r| r.get(0),
                        )?;
                        root = Some((asset_root_id.clone(), PathBuf::from(path)));
                    }
                    let (_, root_path) = root.as_ref().unwrap();
                    let ext = Path::new(&filename)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("mp4")
                        .to_lowercase();
                    clips.push(StagedClip {
                        ordinal: ordinal + 1,
                        kind: block.kind.to_uppercase(),
                        clip_key: clip_key.clone(),
                        src: root_path.join(&rel_path),
                        dest_name: format!("{:02}_{}_{}.{ext}", ordinal + 1, block.kind.to_uppercase(), clip_key),
                        duration_ms,
                    });
                }
                None => {
                    complete = false;
                    report.skipped.push(format!(
                        "{} — block {} ({}) has no selected take",
                        detail.code,
                        ordinal + 1,
                        block.kind
                    ));
                    break;
                }
            }
        }
        if complete && !clips.is_empty() {
            reel_clips.push((detail, clips));
        }
    }

    let Some((_, root_path)) = root else {
        return Err(CoreError::Invalid(
            "no reels in this batch are ready — every block needs a selected take".into(),
        ));
    };

    let handoff_root = root_path.join(HANDOFF_DIR).join(&batch.code);
    std::fs::create_dir_all(&handoff_root)?;

    for (detail, clips) in &reel_clips {
        let folder = handoff_root.join(format!("{}_{}", detail.code, detail.slug));
        if folder.exists() {
            std::fs::remove_dir_all(&folder)?; // regenerable staging
        }
        std::fs::create_dir_all(&folder)?;

        for clip in clips {
            let dest = folder.join(&clip.dest_name);
            if std::fs::hard_link(&clip.src, &dest).is_err() {
                std::fs::copy(&clip.src, &dest)?; // cross-volume fallback
            }
            if clip.duration_ms.is_none() {
                report.warnings.push(format!(
                    "{}/{} — unknown duration (ffprobe not available); timeline assumes {}s",
                    detail.code, clip.dest_name, ASSUMED_CLIP_SECONDS
                ));
            }
        }

        let mut script = format!("{} — {}\n\n", detail.code, detail.title);
        for (i, block) in detail.blocks.iter().enumerate() {
            script.push_str(&format!("[{} {}]\n{}\n\n", i + 1, block.kind.to_uppercase(), block.text));
        }
        std::fs::write(folder.join(format!("{}_script.txt", detail.code)), script)?;

        report.staged.push(HandoffReel {
            reel_id: detail.id.clone(),
            code: detail.code.clone(),
            title: detail.title.clone(),
            folder: folder.to_string_lossy().into_owned(),
            clips: clips.len() as i64,
        });
    }

    // one FCPXML for the whole batch: a timeline per reel
    let fcpxml = build_fcpxml(&batch.code, &handoff_root, &reel_clips);
    let fcpxml_path = handoff_root.join("_IMPORT_ME.fcpxml");
    std::fs::write(&fcpxml_path, fcpxml)?;

    for (detail, _) in &reel_clips {
        if detail.status == "shot" {
            reels::set_status(conn, &detail.id, "assembled", false)?;
        }
    }
    activity::log(
        conn,
        "batch",
        &batch.code,
        "handoff-generated",
        Some(&serde_json::json!({ "reels": report.staged.len() })),
    )?;

    report.handoff_path = handoff_root.to_string_lossy().into_owned();
    report.fcpxml_path = fcpxml_path.to_string_lossy().into_owned();
    Ok(report)
}

/// Selected material for a block: component master take, or the block's own
/// shot's selected take. Returns (root_id, rel_path, filename, duration, clip_key).
#[allow(clippy::type_complexity)]
fn resolve_block_take(
    conn: &Connection,
    block: &reels::BlockView,
) -> Result<Option<(String, String, String, Option<i64>, String)>> {
    let take_id: Option<String> = if let Some(component_id) = &block.component_id {
        conn.query_row(
            "SELECT master_take_id FROM components WHERE id = ?1",
            [component_id],
            |r| r.get(0),
        )?
    } else {
        conn.query_row(
            "SELECT selected_take_id FROM shots WHERE script_block_id = ?1 AND selected_take_id IS NOT NULL",
            [&block.id],
            |r| r.get(0),
        )
        .optional()?
    };
    let Some(take_id) = take_id else { return Ok(None) };

    let row = conn
        .query_row(
            "SELECT a.root_id, a.rel_path, a.filename, a.duration_ms
             FROM takes t JOIN assets a ON a.id = t.asset_id WHERE t.id = ?1",
            [&take_id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                ))
            },
        )
        .optional()?;
    let Some((root_id, rel_path, filename, mut duration_ms)) = row else { return Ok(None) };

    // enrich lazily if ffprobe is available and duration unknown
    if duration_ms.is_none() {
        let root: String =
            conn.query_row("SELECT path FROM storage_roots WHERE id = ?1", [&root_id], |r| r.get(0))?;
        let asset_id: String =
            conn.query_row("SELECT asset_id FROM takes WHERE id = ?1", [&take_id], |r| r.get(0))?;
        if let Some(info) = media::enrich_asset(conn, &asset_id, &Path::new(&root).join(&rel_path))? {
            duration_ms = Some(info.duration_ms);
        }
    }

    let clip_key = block
        .component_code
        .clone()
        .unwrap_or_else(|| format!("S{:02}", block.position + 1));
    Ok(Some((root_id, rel_path, filename, duration_ms, clip_key)))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn file_url(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    let encoded: String = s
        .chars()
        .map(|c| match c {
            '%' => "%25".into(),
            ' ' => "%20".into(),
            '#' => "%23".into(),
            '?' => "%3F".into(),
            c => c.to_string(),
        })
        .collect();
    if encoded.starts_with('/') {
        format!("file://localhost{encoded}")
    } else {
        format!("file://localhost/{encoded}")
    }
}

fn frames(duration_ms: Option<i64>) -> i64 {
    let ms = duration_ms.unwrap_or(ASSUMED_CLIP_SECONDS * 1000);
    ((ms * TIMELINE_FPS) as f64 / 1000.0).round() as i64
}

/// Rational seconds aligned to the timeline frame rate: N frames → "N*100/3000s".
fn rational(frames: i64) -> String {
    format!("{}/{}s", frames * 100, TIMELINE_FPS * 100)
}

/// FCPXML 1.8 with one <project> (timeline) per reel — vertical 1080×1920,
/// selected takes laid end-to-end in block order. DaVinci Resolve:
/// File ▸ Import ▸ Timeline.
fn build_fcpxml(
    batch_code: &str,
    handoff_root: &Path,
    reel_clips: &[(reels::ReelDetail, Vec<StagedClip>)],
) -> String {
    let mut resources = String::new();
    let mut projects = String::new();
    resources.push_str(&format!(
        "    <format id=\"r1\" name=\"FFVideoFormat1080x1920p{TIMELINE_FPS}\" frameDuration=\"100/{}s\" width=\"1080\" height=\"1920\"/>\n",
        TIMELINE_FPS * 100
    ));

    let mut asset_id = 1;
    for (detail, clips) in reel_clips {
        let folder = handoff_root.join(format!("{}_{}", detail.code, detail.slug));
        let mut spine = String::new();
        let mut offset = 0i64;
        let mut clip_assets = String::new();

        for clip in clips {
            asset_id += 1;
            let id = format!("a{asset_id}");
            let f = frames(clip.duration_ms);
            let url = file_url(&folder.join(&clip.dest_name));
            let name = xml_escape(clip.dest_name.trim_end_matches(|c| c != '.').trim_end_matches('.'));
            clip_assets.push_str(&format!(
                "    <asset id=\"{id}\" name=\"{name}\" src=\"{url}\" start=\"0s\" duration=\"{}\" hasVideo=\"1\" hasAudio=\"1\" format=\"r1\"/>\n",
                rational(f)
            ));
            spine.push_str(&format!(
                "            <asset-clip ref=\"{id}\" offset=\"{}\" name=\"{name}\" start=\"0s\" duration=\"{}\"/>\n",
                rational(offset),
                rational(f)
            ));
            offset += f;
        }

        resources.push_str(&clip_assets);
        projects.push_str(&format!(
            "      <project name=\"{}_{}\">\n        <sequence format=\"r1\" duration=\"{}\" tcStart=\"0s\" tcFormat=\"NDF\">\n          <spine>\n{spine}          </spine>\n        </sequence>\n      </project>\n",
            xml_escape(&detail.code),
            xml_escape(&detail.slug),
            rational(offset)
        ));
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE fcpxml>\n<fcpxml version=\"1.8\">\n  <resources>\n{resources}  </resources>\n  <library>\n    <event name=\"{}\">\n{projects}    </event>\n  </library>\n</fcpxml>\n",
        xml_escape(batch_code)
    )
}

// ── exports back in ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ExportMatch {
    pub rel_path: String,
    pub filename: String,
    pub reel_id: String,
    pub reel_code: String,
    pub reel_title: String,
    pub reel_status: String,
}

#[derive(Debug, Serialize)]
pub struct ExportScan {
    pub matches: Vec<ExportMatch>,
    pub unmatched: Vec<String>,
}

/// Find `R####` anywhere in a filename.
pub fn parse_reel_code(name: &str) -> Option<String> {
    let bytes = name.as_bytes();
    for (i, _) in name.char_indices() {
        if bytes[i] == b'R' && i + 4 < bytes.len() {
            let digits = &name[i + 1..i + 5];
            if digits.chars().all(|c| c.is_ascii_digit())
                && !bytes.get(i + 5).is_some_and(|b| b.is_ascii_digit())
            {
                return Some(format!("R{digits}"));
            }
        }
    }
    None
}

/// Scan 03_EXPORTS for rendered finals and propose reel matches by code.
pub fn scan_exports(conn: &Connection, root_id: &str) -> Result<ExportScan> {
    let root: String =
        conn.query_row("SELECT path FROM storage_roots WHERE id = ?1", [root_id], |r| r.get(0))?;
    let exports = Path::new(&root).join(EXPORTS_DIR);
    std::fs::create_dir_all(&exports)?;

    let mut scan = ExportScan { matches: Vec::new(), unmatched: Vec::new() };
    for entry in std::fs::read_dir(&exports)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let filename = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase)
            .unwrap_or_default();
        if !FINAL_EXTS.contains(&ext.as_str()) {
            continue;
        }
        let rel_path = format!("{EXPORTS_DIR}/{filename}");

        let Some(code) = parse_reel_code(&filename) else {
            scan.unmatched
                .push(format!("{filename} — no R#### code in the name; render with the timeline name"));
            continue;
        };
        let reel = conn
            .query_row(
                "SELECT id, code, title, status FROM reels WHERE code = ?1",
                [&code],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?;
        match reel {
            None => scan.unmatched.push(format!("{filename} — no reel {code} exists")),
            Some((id, code, title, status)) => scan.matches.push(ExportMatch {
                rel_path,
                filename,
                reel_id: id,
                reel_code: code,
                reel_title: title,
                reel_status: status,
            }),
        }
    }
    Ok(scan)
}

#[derive(Debug, Serialize)]
pub struct ConfirmedExport {
    pub reel_code: String,
    pub final_path: String,
}

/// File a confirmed export: move to 04_FINALS/YYYY-MM/, index as a `final`
/// asset, link as the reel's final render, and mark the reel `edited`.
pub fn confirm_export(
    conn: &Connection,
    root_id: &str,
    rel_path: &str,
    reel_id: &str,
) -> Result<ConfirmedExport> {
    let root: String =
        conn.query_row("SELECT path FROM storage_roots WHERE id = ?1", [root_id], |r| r.get(0))?;
    let src = Path::new(&root).join(rel_path);
    if !src.is_file() {
        return Err(CoreError::NotFound(format!("export file {rel_path}")));
    }
    let reel = reels::get_detail(conn, reel_id)?;

    let month = &ids::now_iso()[..7]; // YYYY-MM
    let finals_dir = Path::new(&root).join(FINALS_DIR).join(month);
    std::fs::create_dir_all(&finals_dir)?;
    let filename = src.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let dest = finals_dir.join(&filename);
    if dest.exists() {
        return Err(CoreError::Invalid(format!(
            "{filename} already exists in {FINALS_DIR}/{month} — rename the export (e.g. _v2) first"
        )));
    }
    std::fs::rename(&src, &dest)?;
    let new_rel = format!("{FINALS_DIR}/{month}/{filename}");

    let meta = std::fs::metadata(&dest)?;
    let asset_id = ids::new_id();
    conn.execute(
        "INSERT INTO assets (id, root_id, rel_path, filename, kind, size_bytes, imported_at)
         VALUES (?1, ?2, ?3, ?4, 'final', ?5, ?6)",
        rusqlite::params![asset_id, root_id, new_rel, filename, meta.len() as i64, ids::now_iso()],
    )?;
    let _ = media::enrich_asset(conn, &asset_id, &dest);

    conn.execute(
        "UPDATE reels SET final_asset_id = ?1 WHERE id = ?2",
        rusqlite::params![asset_id, reel_id],
    )?;
    if !matches!(reel.status.as_str(), "edited" | "scheduled" | "posted" | "verified") {
        reels::set_status(conn, reel_id, "edited", true)?; // assembled → edited skips 'editing'
    }
    activity::log(
        conn,
        "reel",
        &reel.code,
        "final-matched",
        Some(&serde_json::json!({ "file": filename })),
    )?;

    Ok(ConfirmedExport {
        reel_code: reel.code,
        final_path: dest.to_string_lossy().into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components, library, roots, Db};

    fn shot_ready_fixture(db: &Db, dir: &Path) -> (String, String) {
        // one modular + standalone reel, batch, ingested + selected takes
        let hook = components::create(&db.conn, "hook", "Hook line", &[], None).unwrap();
        let reel = reels::create(&db.conn, "Gym myths #1", None).unwrap();
        reels::set_blocks(&db.conn, &reel.id, &[
            reels::BlockInput { kind: "hook".into(), component_id: Some(hook.id), text: None, est_seconds: None },
            reels::BlockInput { kind: "body".into(), component_id: None, text: Some("Body text".into()), est_seconds: None },
        ]).unwrap();
        let root = roots::add_root(&db.conn, "media", dir.to_str().unwrap(), "local").unwrap();
        let batch = batches::create(&db.conn, "Day 1", None).unwrap();
        batches::add_reels(&db.conn, &batch.id, &[reel.id.clone()]).unwrap();

        std::fs::create_dir_all(dir.join(library::INBOX_DIR)).unwrap();
        let detail = batches::detail(&db.conn, &batch.id).unwrap();
        for (i, shot) in detail.shots.iter().enumerate() {
            std::fs::write(dir.join(library::INBOX_DIR).join(format!("c{i}.mp4")), format!("clip{i}")).unwrap();
            library::scan_inbox(&db.conn, &root.id).unwrap();
            let asset = library::list_inbox(&db.conn, &root.id).unwrap()
                .into_iter().find(|a| a.filename.contains(&format!("c{i}"))).unwrap();
            let take = library::ingest_take(&db.conn, &shot.id, &asset.id, None).unwrap();
            batches::select_take(&db.conn, &shot.id, &take.id).unwrap();
        }
        (batch.id, root.id)
    }

    #[test]
    fn handoff_stages_files_and_fcpxml() {
        let db = Db::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let (batch_id, _) = shot_ready_fixture(&db, dir.path());

        let report = generate_handoff(&db.conn, &batch_id).unwrap();
        assert_eq!(report.staged.len(), 1);
        assert_eq!(report.staged[0].clips, 2);

        let reel_dir = dir.path().join(HANDOFF_DIR).join("B001").join("R0001_gym-myths-1");
        assert!(reel_dir.join("01_HOOK_HK0001.mp4").is_file());
        assert!(reel_dir.join("02_BODY_S02.mp4").is_file());
        assert!(reel_dir.join("R0001_script.txt").is_file());

        let xml = std::fs::read_to_string(dir.path().join(HANDOFF_DIR).join("B001").join("_IMPORT_ME.fcpxml")).unwrap();
        assert!(xml.contains("<project name=\"R0001_gym-myths-1\">"));
        assert!(xml.contains("01_HOOK_HK0001"));
        assert!(xml.matches("<asset-clip").count() == 2);
        assert!(xml.contains("width=\"1080\" height=\"1920\""));

        // reel advanced, warnings mention assumed durations (no ffprobe here)
        let reel_status: String = db.conn.query_row("SELECT status FROM reels", [], |r| r.get(0)).unwrap();
        assert_eq!(reel_status, "assembled");

        // regeneration is idempotent
        let report2 = generate_handoff(&db.conn, &batch_id).unwrap();
        assert_eq!(report2.staged.len(), 1);
    }

    #[test]
    fn export_matching_and_confirm() {
        let db = Db::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let (batch_id, root_id) = shot_ready_fixture(&db, dir.path());
        generate_handoff(&db.conn, &batch_id).unwrap();

        let exports = dir.path().join(EXPORTS_DIR);
        std::fs::create_dir_all(&exports).unwrap();
        std::fs::write(exports.join("R0001_gym-myths-1.mp4"), "final render").unwrap();
        std::fs::write(exports.join("random-clip.mp4"), "???").unwrap();

        let scan = scan_exports(&db.conn, &root_id).unwrap();
        assert_eq!(scan.matches.len(), 1);
        assert_eq!(scan.matches[0].reel_code, "R0001");
        assert_eq!(scan.unmatched.len(), 1);

        let confirmed = confirm_export(&db.conn, &root_id, &scan.matches[0].rel_path, &scan.matches[0].reel_id).unwrap();
        assert_eq!(confirmed.reel_code, "R0001");
        assert!(!exports.join("R0001_gym-myths-1.mp4").exists(), "moved out of exports");

        let (status, final_asset): (String, Option<String>) = db.conn.query_row(
            "SELECT status, final_asset_id FROM reels WHERE code = 'R0001'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(status, "edited");
        assert!(final_asset.is_some());
    }

    #[test]
    fn reel_code_parsing() {
        assert_eq!(parse_reel_code("R0142_gym-myths_FINAL.mp4"), Some("R0142".into()));
        assert_eq!(parse_reel_code("final R0001.mov"), Some("R0001".into()));
        assert_eq!(parse_reel_code("R01423-not-a-code.mp4"), None);
        assert_eq!(parse_reel_code("no-code.mp4"), None);
    }
}
