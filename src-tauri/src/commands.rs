use std::collections::HashMap;

use contentos_core::{activity, jobs, roots, settings};
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

    Ok(DashboardStats {
        reels_by_status,
        problems,
        roots_total: all_roots.len() as i64,
        roots_online,
    })
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
