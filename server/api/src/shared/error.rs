use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal database error: {0}")]
    Database(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

pub type Result<T, E = AppError> = std::result::Result<T, E>;
