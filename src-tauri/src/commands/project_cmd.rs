use tauri::State;
use crate::AppState;
use crate::commands::CommandResult;
use todo_core::{Project, CreateProjectInput, UpdateProjectInput, TaskCountsSummary};

#[tauri::command]
pub fn list_projects(state: State<AppState>, include_archived: bool) -> CommandResult<Vec<Project>> {
    let db = state.db.lock().unwrap();
    let res = db.list_projects(include_archived)?;
    Ok(res)
}

#[tauri::command]
pub fn create_project(state: State<AppState>, input: CreateProjectInput) -> CommandResult<Project> {
    let db = state.db.lock().unwrap();
    let res = db.create_project(input)?;
    Ok(res)
}

#[tauri::command]
pub fn update_project(state: State<AppState>, project_id: String, input: UpdateProjectInput) -> CommandResult<Project> {
    let db = state.db.lock().unwrap();
    let res = db.update_project(&project_id, input)?;
    Ok(res)
}

#[tauri::command]
pub fn archive_project(state: State<AppState>, project_id: String) -> CommandResult<Project> {
    let db = state.db.lock().unwrap();
    let res = db.archive_project(&project_id)?;
    Ok(res)
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, project_id: String) -> CommandResult<()> {
    let mut db = state.db.lock().unwrap();
    db.delete_project(&project_id)?;
    Ok(())
}

#[tauri::command]
pub fn get_task_counts(state: State<AppState>) -> CommandResult<TaskCountsSummary> {
    let db = state.db.lock().unwrap();
    let res = db.get_task_counts()?;
    Ok(res)
}
