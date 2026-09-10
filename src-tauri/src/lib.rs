pub mod commands;

use tauri::Manager;
use std::sync::Mutex;
use todo_core::Database;
use std::fs;
use std::path::PathBuf;

pub struct AppState {
    pub db: Mutex<Database>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            let db_path = PathBuf::from("../data/tasks.dev.db");
            
            #[cfg(not(debug_assertions))]
            let db_path = {
                let data_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("data"));
                let p = data_dir.join("data");
                fs::create_dir_all(&p).expect("failed to create data dir");
                p.join("tasks.db")
            };
            
            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent).unwrap_or(());
            }

            let db = Database::new(db_path).expect("Failed to initialize database");
            app.manage(AppState {
                db: Mutex::new(db),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::task_cmd::list_tasks,
            commands::task_cmd::create_task,
            commands::task_cmd::update_task_status,
            commands::task_cmd::update_task_priority,
            commands::task_cmd::update_task_details,
            commands::task_cmd::move_task,
            commands::task_cmd::delete_task,
            commands::task_cmd::restore_task,
            commands::task_cmd::get_daily_activity,
            commands::project_cmd::list_projects,
            commands::project_cmd::create_project,
            commands::project_cmd::update_project,
            commands::project_cmd::archive_project,
            commands::project_cmd::delete_project,
            commands::project_cmd::get_task_counts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
