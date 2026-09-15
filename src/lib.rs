//! Shared plumbing for the CryptoAlly API's Vercel Rust functions.
//!
//! Each file under `api/` compiles to its own binary/serverless function (Vercel's
//! Rust runtime convention), so this crate holds everything they share: the DB pool
//! and the row types. The pool is cached in a `OnceCell` so a warm Fluid Compute
//! instance reuses the same connections across invocations instead of reconnecting
//! every request.

use once_cell::sync::OnceCell;
use serde::Serialize;
use sha2::Digest;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use vercel_runtime::{Error, Response};

static POOL: OnceCell<PgPool> = OnceCell::new();

/// Connects using a dedicated, read-only database credential, scoped to exactly the
/// access this API needs -- distinct from any credential used by data ingestion or
/// schema administration, both of which run entirely out-of-band from this API.
pub async fn pool() -> Result<&'static PgPool, Error> {
    if let Some(p) = POOL.get() {
        return Ok(p);
    }
    let url = std::env::var("API_DATABASE_URL").map_err(|_| "API_DATABASE_URL is not set")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&url).await?;
    Ok(POOL.get_or_init(|| pool))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AssetSummary {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub currency: String,
    pub market: String,
    pub index_name: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct PricePoint {
    pub date: chrono::NaiveDate,
    pub open: Option<rust_decimal::Decimal>,
    pub high: Option<rust_decimal::Decimal>,
    pub low: Option<rust_decimal::Decimal>,
    pub close: Option<rust_decimal::Decimal>,
    pub volume: Option<i64>,
    pub close_usd: Option<rust_decimal::Decimal>,
}

/// Builds a JSON response. `status` is a plain status code (200, 404, ...) --
/// `vercel_runtime` doesn't re-export `http::StatusCode`, and its response builder
/// accepts a bare `u16` directly, so there's no need to add `http`/`hyper` as an
/// explicit dependency just for the type.
///
/// No `cache-control` header here deliberately: this is used by every endpoint,
/// including the ones gated by `authenticate()`. A shared/CDN cache keys purely on
/// URL, not headers -- a `public` cache-control would let a request with a missing or
/// wrong `x-api-key` be served a cached response from someone else's authenticated
/// request for the same URL, silently bypassing auth. Callers that know their response
/// is safe to cache publicly (nothing behind auth) add the header themselves.
pub fn json_response<T: Serialize>(
    status: u16,
    body: &T,
) -> Result<Response<serde_json::Value>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("cache-control", "private, no-store")
        .body(serde_json::to_value(body)?)?)
}

/// Like `json_response`, but marks the response publicly cacheable. Only use this for
/// responses that don't depend on `authenticate()` -- see the safety note on
/// `json_response`.
pub fn json_response_public<T: Serialize>(
    status: u16,
    body: &T,
) -> Result<Response<serde_json::Value>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("cache-control", "public, max-age=60, stale-while-revalidate=300")
        .body(serde_json::to_value(body)?)?)
}

#[derive(Serialize)]
pub struct ErrorBody<'a> {
    pub error: &'a str,
}

pub fn error_response(status: u16, message: &str) -> Result<Response<serde_json::Value>, Error> {
    json_response(status, &ErrorBody { error: message })
}

/// Checks the request's `x-api-key` header against a hashed-credential store (SHA-256
/// digest, never plaintext at rest). Records last-used-at on success, using the one
/// narrow write this read-mostly role is granted for its own bookkeeping. Best-effort:
/// a failure to record usage doesn't fail the request.
pub async fn authenticate(pool: &PgPool, req: &vercel_runtime::Request) -> bool {
    let Some(key) = req.headers().get("x-api-key").and_then(|v| v.to_str().ok()) else {
        return false;
    };

    let digest = sha2::Sha256::digest(key.as_bytes());
    let hash: String = digest.iter().map(|b| format!("{b:02x}")).collect();

    let row: Result<Option<(i64,)>, _> = sqlx::query_as(
        "SELECT id FROM api_keys WHERE key_hash = $1 AND is_active AND revoked_at IS NULL",
    )
    .bind(&hash)
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some((id,))) => {
            let _ = sqlx::query("UPDATE api_keys SET last_used_at = now() WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await;
            true
        }
        _ => false,
    }
}

/// Parses the request's query string into key/value pairs.
pub fn query_params(req: &vercel_runtime::Request) -> std::collections::HashMap<String, String> {
    req.uri()
        .query()
        .map(|q| url::form_urlencoded::parse(q.as_bytes()).into_owned().collect())
        .unwrap_or_default()
}
