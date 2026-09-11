use serde::Serialize;
use tauri::State;
use crate::AppState;
use crate::commands::CommandResult;

#[derive(Serialize)]
pub struct AppInfo {
    /// 当前使用的 SQLite 文件绝对路径（设置页展示用）
    pub db_path: String,
    pub version: String,
    /// 数据库 schema 版本
    pub schema_version: i32,
}

#[tauri::command]
pub fn get_app_info(state: State<AppState>) -> CommandResult<AppInfo> {
    let db = state.db.lock().unwrap();
    Ok(AppInfo {
        db_path: state.db_path.to_string_lossy().to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version: db.schema_version()?,
    })
}
