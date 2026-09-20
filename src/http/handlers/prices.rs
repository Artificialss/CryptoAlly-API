use axum::extract::{Query, State};
use axum::Json;
use chrono::NaiveDate;
use serde::Deserialize;

use crate::domain::models::{AssetLookup, PriceBar};
use crate::domain::types::AssetId;

use super::super::error::ApiError;
use super::super::state::AppState;

const DEFAULT_LIMIT: i64 = 2000;
const MAX_LIMIT: i64 = 10_000;

#[derive(Debug, Deserialize)]
pub struct PricesParams {
    asset_id: Option<i64>,
    market: Option<String>,
    symbol: Option<String>,
    from: Option<String>,
    to: Option<String>,
    limit: Option<i64>,
}

fn lookup_from_params(params: &PricesParams) -> Result<AssetLookup, ApiError> {
    if let Some(id) = params.asset_id {
        return Ok(AssetLookup::Id(AssetId(id)));
    }
    if let (Some(market), Some(symbol)) = (&params.market, &params.symbol) {
        return Ok(AssetLookup::MarketSymbol { market: market.clone(), symbol: symbol.clone() });
    }
    Err(ApiError::BadRequest("provide either asset_id, or both market and symbol".to_string()))
}

/// GET /api/prices?asset_id=1&from=2024-01-01&to=2024-12-31
/// GET /api/prices?market=crypto&symbol=bitcoin&from=2024-01-01&to=2024-12-31
///
/// Daily OHLCV history for one asset, in its native currency (see the asset's
/// `currency` field from `/api/assets`). A client wanting USD (or any other
/// currency) converts client-side -- there's no server-computed USD field.
/// Resolve by `asset_id` directly, or by `market` (slug) + `symbol` (external_id)
/// -- symbols alone aren't unique across markets.
pub async fn get_prices(
    State(state): State<AppState>,
    Query(params): Query<PricesParams>,
) -> Result<Json<Vec<PriceBar>>, ApiError> {
    let lookup = lookup_from_params(&params)?;

    let from = params
        .from
        .as_deref()
        .and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let to = params
        .to
        .as_deref()
        .and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);

    match state.prices.history(lookup, from, to, limit).await? {
        Some(bars) => Ok(Json(bars)),
        None => Err(ApiError::NotFound("no such asset".to_string())),
    }
}
