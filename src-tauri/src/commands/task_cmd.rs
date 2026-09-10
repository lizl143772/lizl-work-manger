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
