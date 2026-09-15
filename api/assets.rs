//! GET /api/assets?type=crypto&market=japan&search=apple&limit=50&offset=0
//!
//! Lists assets, optionally filtered by type, market slug, and a symbol/name search.
//! Backs a screener-style listing use case.

use cryptoally_api::{authenticate, error_response, json_response, pool, query_params, AssetSummary};
use sqlx::{Postgres, QueryBuilder};
use vercel_runtime::{run, service_fn, Error, Request, Response};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 500;

pub async fn handler(req: Request) -> Result<Response<serde_json::Value>, Error> {
    let params = query_params(&req);
    let p = match pool().await {
        Ok(p) => p,
        Err(_) => return error_response(503, "database unreachable"),
    };

    if !authenticate(p, &req).await {
        return error_response(401, "missing or invalid API key (x-api-key header)");
    }

    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let offset = params
        .get("offset")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0)
        .max(0);

    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT a.id, a.symbol, a.name, a.asset_type::text AS asset_type, a.currency, \
                m.slug AS market, a.index_name \
         FROM assets a JOIN markets m ON m.id = a.market_id WHERE a.is_active",
    );

    if let Some(asset_type) = params.get("type") {
        qb.push(" AND a.asset_type = ").push_bind(asset_type).push("::asset_type");
    }
    if let Some(market) = params.get("market") {
        qb.push(" AND m.slug = ").push_bind(market);
    }
    if let Some(search) = params.get("search") {
        let pattern = format!("%{}%", search);
        qb.push(" AND (a.symbol ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR a.name ILIKE ")
            .push_bind(pattern)
            .push(")");
    }

    qb.push(" ORDER BY a.symbol LIMIT ").push_bind(limit).push(" OFFSET ").push_bind(offset);

    match qb.build_query_as::<AssetSummary>().fetch_all(p).await {
        Ok(assets) => json_response(200, &assets),
        Err(e) => {
            eprintln!("assets query failed: {e}");
            error_response(500, "query failed")
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
