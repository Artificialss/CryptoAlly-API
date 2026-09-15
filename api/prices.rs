//! GET /api/prices?asset_id=1&from=2024-01-01&to=2024-12-31
//! GET /api/prices?market=crypto&symbol=bitcoin&from=2024-01-01&to=2024-12-31
//!
//! Daily price history for one asset. Resolve by `asset_id` directly, or by
//! `market` (slug) + `symbol` (a ticker or asset identifier) -- symbols alone aren't
//! unique across markets, since the same ticker can exist on more than one exchange.

use chrono::NaiveDate;
use cryptoally_api::{authenticate, error_response, json_response, pool, query_params, PricePoint};
use vercel_runtime::{run, service_fn, Error, Request, Response};

const DEFAULT_LIMIT: i64 = 2000;
const MAX_LIMIT: i64 = 10_000;

async fn resolve_asset_id(
    p: &sqlx::PgPool,
    params: &std::collections::HashMap<String, String>,
) -> Result<Option<i64>, sqlx::Error> {
    if let Some(id) = params.get("asset_id").and_then(|v| v.parse::<i64>().ok()) {
        return Ok(Some(id));
    }
    if let (Some(market), Some(symbol)) = (params.get("market"), params.get("symbol")) {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT a.id FROM assets a JOIN markets m ON m.id = a.market_id \
             WHERE m.slug = $1 AND a.external_id = $2",
        )
        .bind(market)
        .bind(symbol)
        .fetch_optional(p)
        .await?;
        return Ok(row.map(|(id,)| id));
    }
    Ok(None)
}

pub async fn handler(req: Request) -> Result<Response<serde_json::Value>, Error> {
    let params = query_params(&req);
    let p = match pool().await {
        Ok(p) => p,
        Err(_) => return error_response(503, "database unreachable"),
    };

    if !authenticate(p, &req).await {
        return error_response(401, "missing or invalid API key (x-api-key header)");
    }

    let asset_id = match resolve_asset_id(p, &params).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return error_response(400, "provide either asset_id, or both market and symbol")
        }
        Err(e) => {
            eprintln!("asset lookup failed: {e}");
            return error_response(500, "lookup failed");
        }
    };

    let from: NaiveDate = params
        .get("from")
        .and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let to: NaiveDate = params
        .get("to")
        .and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);

    let result = sqlx::query_as::<_, PricePoint>(
        "SELECT date, open, high, low, close, volume, close_usd \
         FROM daily_prices \
         WHERE asset_id = $1 AND date BETWEEN $2 AND $3 \
         ORDER BY date ASC LIMIT $4",
    )
    .bind(asset_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(p)
    .await;

    match result {
        Ok(prices) => json_response(200, &prices),
        Err(e) => {
            eprintln!("prices query failed: {e}");
            error_response(500, "query failed")
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
