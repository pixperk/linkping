use std:: fmt::Display;

use axum::{response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError{
    DatabaseError(String),
    ValidationError(String),
    NotFound(String),
    InternalServerError(String),
    Unauthorized(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError{
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            AppError::DatabaseError(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::ValidationError(e) => (axum::http::StatusCode::BAD_REQUEST, e),
            AppError::NotFound(e) => (axum::http::StatusCode::NOT_FOUND, e),
            AppError::InternalServerError(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::Unauthorized(e) => (axum::http::StatusCode::UNAUTHORIZED, e),
        };

        let body = Json(ErrorResponse {
            error: msg,
        });
        (status, body).into_response()
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::ValidationError(e) => write!(f, "Validation error: {}", e),
            AppError::NotFound(e) => write!(f, "Not found: {}", e),
            AppError::InternalServerError(e) => write!(f, "Internal server error: {}", e),
            AppError::Unauthorized(e) => write!(f, "Unauthorized: {}", e),
        }
    }
}