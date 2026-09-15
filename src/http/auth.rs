use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::domain::types::ApiKeyPlaintext;

use super::error::ApiError;
use super::state::AppState;

/// Applied as a `Router` layer to every route except `/api/health`. Computes
/// SHA-256(key) in memory and checks it against the hashed-credential store in
/// Postgres -- a request with a missing or invalid key never reaches the handler (or
/// its database query) it would otherwise have triggered.
pub async fn require_api_key(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let Some(key) = key else {
        return Err(ApiError::Unauthorized);
    };

    if state.auth.verify(&ApiKeyPlaintext(key)).await {
        Ok(next.run(req).await)
    } else {
        Err(ApiError::Unauthorized)
    }
}
