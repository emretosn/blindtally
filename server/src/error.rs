use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use blindtally_core::errors::AppError;

/// The error type every handler returns. axum turns it into an HTTP
/// response through the `IntoResponse` impl below.
///
/// We can't `impl IntoResponse for AppError` directly: the trait lives in
/// axum and the type in blindtally-core, and Rust's orphan rule forbids
/// implementing a foreign trait for a foreign type. Wrapping it is the fix.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        ApiError {
            status,
            message: message.into(),
        }
    }
}

/// Lets `?` convert core errors inside handlers. Undecodable bodies are the
/// client's fault (400); anything else is ours (500).
impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        let status = match err {
            AppError::Decode(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        ApiError::new(status, err.to_string())
    }
}

/// A `spawn_blocking` task panicked or was cancelled.
impl From<tokio::task::JoinError> for ApiError {
    fn from(err: tokio::task::JoinError) -> Self {
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::warn!(status = %self.status, "{}", self.message);
        (self.status, self.message).into_response()
    }
}
