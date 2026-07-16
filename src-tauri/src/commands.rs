use std::collections::HashMap;

use contentos_core::{
    activity, batches, components, davinci, ideas, jobs, library, pillars, posts, reels, roots,
    search, settings,
};
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::AppState;

type CmdResult<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[derive(Serialize)]
pub struct AppInfo {
    version: String,
    db_path: String,
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        db_path: state.db_path.display().to_string(),
    }
}

#[derive(Serialize)]
pub struct DashboardStats {
    reels_by_status: HashMap<String, i64>,
    problems: i64,
    roots_total: i64,
    roots_online: i64,
    scheduled_this_week: i64,
    stuck_count: i64,
}

#[tauri::command]
pub fn dashboard_stats(state: State<'_, AppState>) -> CmdResult<DashboardStats> {
    let db = state.db.lock().map_err(err)?;

    let mut reels_by_status = HashMap::new();
    let mut stmt = db
        .conn
        .prepare("SELECT status, COUNT(*) FROM reels GROUP BY status")
        .map_err(err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .map_err(err)?;
    for row in rows {
        let (status, count) = row.map_err(err)?;
        reels_by_status.insert(status, count);
    }

    let problems = jobs::count_failed(&db.conn).map_err(err)?;
    let all_roots = roots::list_roots(&db.conn).map_err(err)?;
    let roots_online = all_roots.iter().filter(|r| r.online).count() as i64;
    let scheduled_this_week = reels::scheduled_this_week(&db.conn).map_err(err)?;
    let stuck_count = reels::stuck(&db.conn, 4).map_err(err)?.len() as i64;

    Ok(DashboardStats {
        reels_by_status,
        problems,
        roots_total: all_roots.len() as i64,
        roots_online,
        scheduled_this_week,
        stuck_count,
    })
}

#[tauri::command]
pub fn reels_stuck(state: State<'_, AppState>, days: i64) -> CmdResult<Vec<reels::ReelSummary>> {
    let db = state.db.lock().map_err(err)?;
    reels::stuck(&db.conn, days).map_err(err)
}

/// Open a folder in the OS file manager. Falls back to the nearest existing
/// ancestor so the button never dead-ends (e.g. an inbox not created yet
/// opens the media root instead).
#[tauri::command]
pub fn reveal_path(path: String) -> CmdResult<()> {
    let mut target = std::path::PathBuf::from(&path);
    while !target.exists() {
        match target.parent() {
            Some(p) if p != target => target = p.to_path_buf(),
            _ => return Err(format!("Folder not found and no parent exists: {path}")),
        }
    }
    #[cfg(target_os = "windows")]
    let spawned = std::process::Command::new("explorer").arg(&target).spawn();
    #[cfg(target_os = "macos")]
    let spawned = std::process::Command::new("open").arg(&target).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let spawned = std::process::Command::new("xdg-open").arg(&target).spawn();

    // explorer.exe returns a non-zero code even on success, so we only care
    // that the process launched, not its exit status.
    spawned.map(|_| ()).map_err(|e| format!("Couldn't open the file manager: {e}"))
}

#[tauri::command]
pub fn backup_now(state: State<'_, AppState>) -> CmdResult<String> {
    let db = state.db.lock().map_err(err)?;
    let backups_dir = state
        .db_path
        .parent()
        .ok_or("no data dir")?
        .join("backups");
    let path = contentos_core::backup::backup_now(&db.conn, &backups_dir).map_err(err)?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn list_storage_roots(state: State<'_, AppState>) -> CmdResult<Vec<roots::StorageRoot>> {
    let db = state.db.lock().map_err(err)?;
    roots::list_roots(&db.conn).map_err(err)
}

#[tauri::command]
pub fn add_storage_root(
    state: State<'_, AppState>,
    name: String,
    path: String,
    kind: String,
) -> CmdResult<roots::StorageRoot> {
    let db = state.db.lock().map_err(err)?;
    let root = roots::add_root(&db.conn, &name, &path, &kind).map_err(err)?;
    activity::log(&db.conn, "storage_root", &root.id, "added", None).map_err(err)?;
    Ok(root)
}

#[tauri::command]
pub fn remove_storage_root(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    roots::remove_root(&db.conn, &id).map_err(err)?;
    activity::log(&db.conn, "storage_root", &id, "removed", None).map_err(err)
}

#[tauri::command]
pub fn relocate_storage_root(
    state: State<'_, AppState>,
    id: String,
    new_path: String,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    roots::relocate_root(&db.conn, &id, &new_path).map_err(err)?;
    activity::log(&db.conn, "storage_root", &id, "relocated", None).map_err(err)
}

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> CmdResult<Option<Value>> {
    let db = state.db.lock().map_err(err)?;
    settings::get(&db.conn, &key).map_err(err)
}

#[tauri::command]
pub fn set_setting(state: State<'_, AppState>, key: String, value: Value) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    settings::set(&db.conn, &key, &value).map_err(err)
}

#[tauri::command]
pub fn list_jobs(state: State<'_, AppState>) -> CmdResult<Vec<jobs::Job>> {
    let db = state.db.lock().map_err(err)?;
    jobs::list_recent(&db.conn, 100).map_err(err)
}

#[tauri::command]
pub fn retry_job(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    jobs::retry(&db.conn, &id).map_err(err)
}

// ── pillars ──────────────────────────────────────────────────────

#[tauri::command]
pub fn pillars_list(state: State<'_, AppState>, include_archived: bool) -> CmdResult<Vec<pillars::Pillar>> {
    let db = state.db.lock().map_err(err)?;
    pillars::list(&db.conn, include_archived).map_err(err)
}

#[tauri::command]
pub fn pillars_create(
    state: State<'_, AppState>,
    name: String,
    color: String,
    target_per_week: i64,
) -> CmdResult<pillars::Pillar> {
    let db = state.db.lock().map_err(err)?;
    pillars::create(&db.conn, &name, &color, target_per_week).map_err(err)
}

#[tauri::command]
pub fn pillars_update(
    state: State<'_, AppState>,
    id: String,
    name: String,
    color: String,
    target_per_week: i64,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    pillars::update(&db.conn, &id, &name, &color, target_per_week).map_err(err)
}

#[tauri::command]
pub fn pillars_archive(state: State<'_, AppState>, id: String, archived: bool) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    pillars::set_archived(&db.conn, &id, archived).map_err(err)
}

// ── ideas ────────────────────────────────────────────────────────

#[tauri::command]
pub fn ideas_list(state: State<'_, AppState>, status: Option<String>) -> CmdResult<Vec<ideas::Idea>> {
    let db = state.db.lock().map_err(err)?;
    ideas::list(&db.conn, status.as_deref()).map_err(err)
}

#[tauri::command]
pub fn ideas_create(
    state: State<'_, AppState>,
    title: String,
    pillar_id: Option<String>,
) -> CmdResult<ideas::Idea> {
    let db = state.db.lock().map_err(err)?;
    ideas::create(&db.conn, &title, pillar_id.as_deref()).map_err(err)
}

#[tauri::command]
pub fn ideas_update(
    state: State<'_, AppState>,
    id: String,
    title: String,
    notes: Option<String>,
    pillar_id: Option<String>,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    ideas::update(&db.conn, &id, &title, notes.as_deref(), pillar_id.as_deref()).map_err(err)
}

#[tauri::command]
pub fn ideas_kill(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    ideas::kill(&db.conn, &id).map_err(err)
}

#[tauri::command]
pub fn ideas_promote(state: State<'_, AppState>, id: String) -> CmdResult<reels::ReelDetail> {
    let db = state.db.lock().map_err(err)?;
    ideas::promote(&db.conn, &id).map_err(err)
}

// ── components ───────────────────────────────────────────────────

#[tauri::command]
pub fn components_list(
    state: State<'_, AppState>,
    kind: Option<String>,
    include_archived: bool,
    query: Option<String>,
) -> CmdResult<Vec<components::Component>> {
    let db = state.db.lock().map_err(err)?;
    components::list(&db.conn, kind.as_deref(), include_archived, query.as_deref()).map_err(err)
}

#[tauri::command]
pub fn components_create(
    state: State<'_, AppState>,
    kind: String,
    text: String,
    tags: Vec<String>,
    pillar_id: Option<String>,
) -> CmdResult<components::Component> {
    let db = state.db.lock().map_err(err)?;
    components::create(&db.conn, &kind, &text, &tags, pillar_id.as_deref()).map_err(err)
}

#[tauri::command]
pub fn components_update(
    state: State<'_, AppState>,
    id: String,
    text: String,
    tags: Vec<String>,
    pillar_id: Option<String>,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    components::update(&db.conn, &id, &text, &tags, pillar_id.as_deref()).map_err(err)
}

#[tauri::command]
pub fn components_archive(state: State<'_, AppState>, id: String, archived: bool) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    components::set_archived(&db.conn, &id, archived).map_err(err)
}

// ── reels ────────────────────────────────────────────────────────

#[tauri::command]
pub fn reels_list(
    state: State<'_, AppState>,
    status: Option<String>,
    pillar_id: Option<String>,
    query: Option<String>,
) -> CmdResult<Vec<reels::ReelSummary>> {
    let db = state.db.lock().map_err(err)?;
    reels::list(&db.conn, status.as_deref(), pillar_id.as_deref(), query.as_deref()).map_err(err)
}

#[tauri::command]
pub fn reels_create(
    state: State<'_, AppState>,
    title: String,
    pillar_id: Option<String>,
) -> CmdResult<reels::ReelDetail> {
    let db = state.db.lock().map_err(err)?;
    reels::create(&db.conn, &title, pillar_id.as_deref()).map_err(err)
}

#[tauri::command]
pub fn reels_get(state: State<'_, AppState>, id: String) -> CmdResult<reels::ReelDetail> {
    let db = state.db.lock().map_err(err)?;
    reels::get_detail(&db.conn, &id).map_err(err)
}

#[tauri::command]
pub fn reels_update_meta(
    state: State<'_, AppState>,
    id: String,
    title: String,
    notes: Option<String>,
    pillar_id: Option<String>,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    reels::update_meta(&db.conn, &id, &title, notes.as_deref(), pillar_id.as_deref()).map_err(err)
}

#[tauri::command]
pub fn reels_set_blocks(
    state: State<'_, AppState>,
    id: String,
    blocks: Vec<reels::BlockInput>,
) -> CmdResult<reels::ReelDetail> {
    let db = state.db.lock().map_err(err)?;
    reels::set_blocks(&db.conn, &id, &blocks).map_err(err)?;
    reels::get_detail(&db.conn, &id).map_err(err)
}

#[tauri::command]
pub fn reels_set_status(
    state: State<'_, AppState>,
    id: String,
    status: String,
    overrule: bool,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    reels::set_status(&db.conn, &id, &status, overrule).map_err(err)
}

#[tauri::command]
pub fn reels_set_target_date(
    state: State<'_, AppState>,
    id: String,
    date: Option<String>,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    reels::set_target_date(&db.conn, &id, date.as_deref()).map_err(err)
}

#[tauri::command]
pub fn calendar_range(
    state: State<'_, AppState>,
    start: String,
    end: String,
) -> CmdResult<Vec<reels::CalendarReel>> {
    let db = state.db.lock().map_err(err)?;
    reels::calendar_range(&db.conn, &start, &end).map_err(err)
}

#[tauri::command]
pub fn reels_unscheduled(state: State<'_, AppState>) -> CmdResult<Vec<reels::ReelSummary>> {
    let db = state.db.lock().map_err(err)?;
    reels::unscheduled(&db.conn).map_err(err)
}

#[tauri::command]
pub fn search_all(state: State<'_, AppState>, query: String) -> CmdResult<Vec<search::SearchHit>> {
    let db = state.db.lock().map_err(err)?;
    search::query(&db.conn, &query, 30).map_err(err)
}

// ── shoot batches ────────────────────────────────────────────────

#[tauri::command]
pub fn batches_list(state: State<'_, AppState>) -> CmdResult<Vec<batches::BatchSummary>> {
    let db = state.db.lock().map_err(err)?;
    batches::list(&db.conn).map_err(err)
}

#[tauri::command]
pub fn batch_create(
    state: State<'_, AppState>,
    name: String,
    shoot_date: Option<String>,
) -> CmdResult<batches::BatchSummary> {
    let db = state.db.lock().map_err(err)?;
    batches::create(&db.conn, &name, shoot_date.as_deref()).map_err(err)
}

#[tauri::command]
pub fn batch_add_reels(
    state: State<'_, AppState>,
    id: String,
    reel_ids: Vec<String>,
) -> CmdResult<batches::BatchDetail> {
    let db = state.db.lock().map_err(err)?;
    batches::add_reels(&db.conn, &id, &reel_ids).map_err(err)?;
    batches::detail(&db.conn, &id).map_err(err)
}

#[tauri::command]
pub fn batch_detail(state: State<'_, AppState>, id: String) -> CmdResult<batches::BatchDetail> {
    let db = state.db.lock().map_err(err)?;
    batches::detail(&db.conn, &id).map_err(err)
}

#[tauri::command]
pub fn batch_set_status(state: State<'_, AppState>, id: String, status: String) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    batches::set_status(&db.conn, &id, &status).map_err(err)
}

#[tauri::command]
pub fn shot_set_status(state: State<'_, AppState>, id: String, status: String) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    batches::set_shot_status(&db.conn, &id, &status).map_err(err)
}

#[tauri::command]
pub fn take_select(state: State<'_, AppState>, shot_id: String, take_id: String) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    batches::select_take(&db.conn, &shot_id, &take_id).map_err(err)
}

#[tauri::command]
pub fn take_rate(
    state: State<'_, AppState>,
    take_id: String,
    rating: Option<i64>,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    batches::set_take_rating(&db.conn, &take_id, rating).map_err(err)
}

// ── library ──────────────────────────────────────────────────────

#[tauri::command]
pub fn inbox_scan(state: State<'_, AppState>, root_id: String) -> CmdResult<library::ScanReport> {
    let db = state.db.lock().map_err(err)?;
    library::scan_inbox(&db.conn, &root_id).map_err(err)
}

#[tauri::command]
pub fn inbox_list(state: State<'_, AppState>, root_id: String) -> CmdResult<Vec<library::AssetView>> {
    let db = state.db.lock().map_err(err)?;
    library::list_inbox(&db.conn, &root_id).map_err(err)
}

#[tauri::command]
pub fn assets_list(
    state: State<'_, AppState>,
    kind: Option<String>,
) -> CmdResult<Vec<library::AssetView>> {
    let db = state.db.lock().map_err(err)?;
    library::list_assets(&db.conn, kind.as_deref()).map_err(err)
}

#[tauri::command]
pub fn take_ingest(
    state: State<'_, AppState>,
    shot_id: String,
    asset_id: String,
    rating: Option<i64>,
) -> CmdResult<batches::TakeView> {
    let db = state.db.lock().map_err(err)?;
    library::ingest_take(&db.conn, &shot_id, &asset_id, rating).map_err(err)
}

// ── davinci handoff ──────────────────────────────────────────────

#[tauri::command]
pub fn handoff_generate(state: State<'_, AppState>, batch_id: String) -> CmdResult<davinci::HandoffReport> {
    let db = state.db.lock().map_err(err)?;
    davinci::generate_handoff(&db.conn, &batch_id).map_err(err)
}

#[tauri::command]
pub fn exports_scan(state: State<'_, AppState>, root_id: String) -> CmdResult<davinci::ExportScan> {
    let db = state.db.lock().map_err(err)?;
    davinci::scan_exports(&db.conn, &root_id).map_err(err)
}

#[tauri::command]
pub fn export_confirm(
    state: State<'_, AppState>,
    root_id: String,
    rel_path: String,
    reel_id: String,
) -> CmdResult<davinci::ConfirmedExport> {
    let db = state.db.lock().map_err(err)?;
    davinci::confirm_export(&db.conn, &root_id, &rel_path, &reel_id).map_err(err)
}

// ── publishing ───────────────────────────────────────────────────

#[tauri::command]
pub fn posts_ensure(
    state: State<'_, AppState>,
    reel_id: String,
    platforms: Vec<String>,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    posts::ensure_posts(&db.conn, &reel_id, &platforms).map_err(err)
}

#[tauri::command]
pub fn posts_list(state: State<'_, AppState>, status: Option<String>) -> CmdResult<Vec<posts::PostView>> {
    let db = state.db.lock().map_err(err)?;
    posts::list_posts(&db.conn, status.as_deref()).map_err(err)
}

#[tauri::command]
pub fn post_update(
    state: State<'_, AppState>,
    id: String,
    caption: Option<String>,
    hashtags: Vec<String>,
    scheduled_at: Option<String>,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    posts::update_post(&db.conn, &id, caption.as_deref(), &hashtags, scheduled_at.as_deref())
        .map_err(err)
}

#[tauri::command]
pub fn post_set_status(
    state: State<'_, AppState>,
    id: String,
    status: String,
    error_note: Option<String>,
) -> CmdResult<()> {
    let db = state.db.lock().map_err(err)?;
    posts::set_post_status(&db.conn, &id, &status, error_note.as_deref()).map_err(err)
}

#[tauri::command]
pub fn posts_bulk_fill(
    state: State<'_, AppState>,
    times: std::collections::HashMap<String, String>,
) -> CmdResult<i64> {
    let db = state.db.lock().map_err(err)?;
    posts::bulk_fill_schedule(&db.conn, &times).map_err(err)
}

#[tauri::command]
pub fn publish_export(
    state: State<'_, AppState>,
    root_id: String,
    post_ids: Vec<String>,
) -> CmdResult<posts::ExportBundle> {
    let db = state.db.lock().map_err(err)?;
    posts::export_bundle(&db.conn, &root_id, &post_ids).map_err(err)
}
