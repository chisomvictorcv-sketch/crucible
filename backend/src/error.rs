//! Custom error types for the Crucible backend.
//!
//! Provides a unified [`AppError`] type that maps internal errors into
//! appropriate HTTP status codes and JSON error responses.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use tracing::error;

/// Structured error response returned to API clients.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Machine-readable error code (e.g., `"database_error"`, `"not_found"`).
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

/// Application-level error type that unifies all possible error sources.
///
/// Each variant maps to an HTTP status code and produces a consistent
/// JSON error response via the [`IntoResponse`] implementation.
#[derive(Debug, Error)]
pub enum AppError {
    /// 404 — The requested resource was not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// 400 — The request was malformed or contained invalid data.
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// 401 — Authentication is required or failed.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// 403 — The authenticated user lacks permission.
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// 409 — The request conflicts with the current state.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// 422 — The request body was well-formed but semantically invalid.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// 500 — An internal database error occurred.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// 500 — An internal Redis error occurred.
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// 500 — A serialization error occurred.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// 500 — A catch-all for unexpected internal errors.
    #[error("Internal error: {0}")]
    InternalError(String),

    /// 502 — External Stellar integration failed.
    #[error("Stellar operation failed: {0}")]
    StellarError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            AppError::ValidationError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error",
                msg.clone(),
            ),
            AppError::Database(e) => {
                error!(error = ?e, "Database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "An internal database error occurred".to_string(),
                )
            }
            AppError::Redis(e) => {
                error!(error = ?e, "Redis error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "redis_error",
                    "A cache error occurred".to_string(),
                )
            }
            AppError::Serialization(e) => {
                error!(error = ?e, "Serialization error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "serialization_error",
                    "A serialization error occurred".to_string(),
                )
            }
            AppError::InternalError(msg) => {
                error!(error = %msg, "Internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred".to_string(),
                )
            }
            AppError::StellarError(msg) => {
                error!(error = %msg, "Stellar integration error");
                (
                    StatusCode::BAD_GATEWAY,
                    "stellar_error",
                    "A Stellar network error occurred".to_string(),
                )
            }
        };

        let body = Json(ErrorResponse { code: code.to_string(), message });
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error_display() {
        let err = AppError::NotFound("Contract not found".into());
        assert_eq!(err.to_string(), "Not found: Contract not found");
    }

    #[test]
    fn test_bad_request_error_display() {
        let err = AppError::BadRequest("Invalid address format".into());
        assert_eq!(err.to_string(), "Bad request: Invalid address format");
    }

    #[test]
    fn test_validation_error_display() {
        let err = AppError::ValidationError("name is required".into());
        assert_eq!(err.to_string(), "Validation error: name is required");
    }

    #[test]
    fn test_internal_error_display() {
        let err = AppError::InternalError("unexpected state".into());
        assert_eq!(err.to_string(), "Internal error: unexpected state");
    }
}
