use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug)]
pub enum ApiError {
    Unauthorized,
    BadRequest(String),
    NotFound(String),
    Internal,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "missing or invalid API key (x-api-key header)".to_string())
            }
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
        };
        (status, Json(ErrorBody { error: &message })).into_response()
    }
}

/// Database errors are logged server-side with detail but never echoed to the client
/// -- the response only ever says "query failed" / "internal error", never a raw
/// sqlx/Postgres error string.
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        eprintln!("database error: {err}");
        ApiError::Internal
    }
}
