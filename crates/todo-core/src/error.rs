use thiserror::Error;

#[derive(Error, Debug)]
pub enum TodoError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

pub type Result<T> = std::result::Result<T, TodoError>;
