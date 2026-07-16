mod commands;

use std::path::PathBuf;
use std::sync::Mutex;

use contentos_core::Db;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Db>,
    pub db_path: PathBuf,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("contentos.db");
            let db = Db::open(&db_path)
                .map_err(|e| format!("failed to open database at {}: {e}", db_path.display()))?;
            // daily safety net — never blocks startup on failure
            let _ = contentos_core::backup::daily_backup(&db.conn, &data_dir.join("backups"));
            app.manage(AppState { db: Mutex::new(db), db_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::dashboard_stats,
            commands::list_storage_roots,
            commands::add_storage_root,
            commands::remove_storage_root,
            commands::relocate_storage_root,
            commands::get_setting,
            commands::set_setting,
            commands::list_jobs,
            commands::retry_job,
            commands::pillars_list,
            commands::pillars_create,
            commands::pillars_update,
            commands::pillars_archive,
            commands::ideas_list,
            commands::ideas_create,
            commands::ideas_update,
            commands::ideas_kill,
            commands::ideas_promote,
            commands::components_list,
            commands::components_create,
            commands::components_update,
            commands::components_archive,
            commands::reels_list,
            commands::reels_create,
            commands::reels_get,
            commands::reels_update_meta,
            commands::reels_set_blocks,
            commands::reels_set_status,
            commands::reels_set_target_date,
            commands::calendar_range,
            commands::reels_unscheduled,
            commands::search_all,
            commands::batches_list,
            commands::batch_create,
            commands::batch_add_reels,
            commands::batch_detail,
            commands::batch_set_status,
            commands::shot_set_status,
            commands::take_select,
            commands::take_rate,
            commands::inbox_scan,
            commands::inbox_list,
            commands::assets_list,
            commands::take_ingest,
            commands::handoff_generate,
            commands::exports_scan,
            commands::export_confirm,
            commands::posts_ensure,
            commands::posts_list,
            commands::post_update,
            commands::post_set_status,
            commands::posts_bulk_fill,
            commands::publish_export,
            commands::reels_stuck,
            commands::backup_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ContentOS");
}
