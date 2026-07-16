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
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("contentos.db");
            let db = Db::open(&db_path)
                .map_err(|e| format!("failed to open database at {}: {e}", db_path.display()))?;
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running ContentOS");
}
