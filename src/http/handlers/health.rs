use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use super::super::state::AppState;

#[derive(Serialize)]
struct Health {
    status: &'static str,
    database: &'static str,
}

/// GET /api/health -- public, no API key required (a liveness probe).
pub async fn health(State(state): State<AppState>) -> Response {
    match sqlx::query("SELECT 1").execute(state.pool).await {
        Ok(_) => (
            [(header::CACHE_CONTROL, "public, max-age=60, stale-while-revalidate=300")],
            Json(Health { status: "ok", database: "ok" }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(Health { status: "error", database: "query failed" }),
        )
            .into_response(),
    }
}
