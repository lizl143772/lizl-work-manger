pub mod task_cmd;
pub mod project_cmd;

use serde::Serialize;
use todo_core::TodoError;

#[derive(Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<TodoError> for CommandError {
    fn from(err: TodoError) -> Self {
        let (code, message) = match &err {
            TodoError::ValidationError(msg) => ("VALIDATION_ERROR", msg.clone()),
            TodoError::NotFound(msg) => ("NOT_FOUND", msg.clone()),
            TodoError::Conflict(msg) => ("CONFLICT", msg.clone()),
            TodoError::DatabaseError(e) => ("STORAGE_ERROR", e.to_string()),
        };
        CommandError {
            code: code.to_string(),
            message,
        }
    }
}

pub type CommandResult<T> = std::result::Result<T, CommandError>;
