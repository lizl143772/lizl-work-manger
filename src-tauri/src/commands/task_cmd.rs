use tauri::State;
use crate::AppState;
use crate::commands::CommandResult;
use todo_core::{Task, CreateTaskInput, TaskQuery, TaskStatus, DailyActivity};
use std::str::FromStr;

#[tauri::command]
pub fn list_tasks(state: State<AppState>, query: TaskQuery) -> CommandResult<todo_core::TaskPage> {
    let db = state.db.lock().unwrap();
    let res = db.list_tasks(query)?;
    Ok(res)
}

#[tauri::command]
pub fn create_task(state: State<AppState>, input: CreateTaskInput) -> CommandResult<Task> {
    let db = state.db.lock().unwrap();
    let res = db.create_task(input)?;
    Ok(res)
}

#[tauri::command]
pub fn update_task_status(state: State<AppState>, task_id: String, status: String) -> CommandResult<Task> {
    let db = state.db.lock().unwrap();
    let st = TaskStatus::from_str(&status)?;
    let res = db.update_task_status(&task_id, st)?;
    Ok(res)
}

#[tauri::command]
pub fn update_task_priority(state: State<AppState>, task_id: String, priority: u8) -> CommandResult<Task> {
    let db = state.db.lock().unwrap();
    let res = db.update_task_priority(&task_id, priority)?;
    Ok(res)
}

#[tauri::command]
pub fn update_task_details(state: State<'_, AppState>, id: String, input: todo_core::models::task::TaskUpdateInput) -> CommandResult<Task> {
    println!("update_task_details received id: {}, input attachments is some? {}", id, input.attachments.is_some());
    let db = state.db.lock().unwrap();
    let res = db.update_task_details(&id, input)?;
    Ok(res)
}

#[tauri::command]
pub fn move_task(state: State<AppState>, task_id: String, project_id: String) -> CommandResult<Task> {
    let db = state.db.lock().unwrap();
    let res = db.move_task(&task_id, &project_id)?;
    Ok(res)
}

#[tauri::command]
pub fn delete_task(state: State<AppState>, task_id: String) -> CommandResult<Task> {
    let db = state.db.lock().unwrap();
    let res = db.delete_task(&task_id)?;
    Ok(res)
}

#[tauri::command]
pub fn restore_task(state: State<AppState>, task_id: String) -> CommandResult<Task> {
    let db = state.db.lock().unwrap();
    let res = db.restore_task(&task_id)?;
    Ok(res)
}

/// 日历视图：取 [from, to]（本地日期 YYYY-MM-DD，闭区间）内每天的活跃度汇总
#[tauri::command]
pub fn get_daily_activity(state: State<AppState>, from: String, to: String) -> CommandResult<Vec<DailyActivity>> {
    let db = state.db.lock().unwrap();
    let res = db.get_daily_activity(&from, &to)?;
    Ok(res)
}

// ---------- 回收站 ----------

#[tauri::command]
pub fn list_deleted_tasks(
    state: State<AppState>,
    keyword: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> CommandResult<todo_core::TaskPage> {
    let db = state.db.lock().unwrap();
    let res = db.list_deleted_tasks(keyword, page.unwrap_or(1), page_size.unwrap_or(200))?;
    Ok(res)
}

#[tauri::command]
pub fn purge_task(state: State<AppState>, task_id: String) -> CommandResult<()> {
    let db = state.db.lock().unwrap();
    db.purge_task(&task_id)?;
    Ok(())
}

#[tauri::command]
pub fn purge_deleted_tasks(state: State<AppState>) -> CommandResult<u64> {
    let db = state.db.lock().unwrap();
    let n = db.purge_deleted_tasks()?;
    Ok(n)
}

#[tauri::command]
pub fn purge_expired_deleted_tasks(state: State<AppState>, retention_days: i64) -> CommandResult<u64> {
    let db = state.db.lock().unwrap();
    let n = db.purge_expired_deleted_tasks(retention_days)?;
    Ok(n)
}

// ---------- 统计 ----------

#[tauri::command]
pub fn get_task_stats(state: State<AppState>) -> CommandResult<todo_core::TaskStats> {
    let db = state.db.lock().unwrap();
    let res = db.get_task_stats()?;
    Ok(res)
}
