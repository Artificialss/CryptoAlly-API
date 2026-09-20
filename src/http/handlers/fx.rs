use axum::extract::{Query, State};
use axum::Json;
use chrono::NaiveDate;
use serde::Deserialize;

use crate::domain::models::FxRate;
use crate::domain::types::FxCurrency;

use super::super::error::ApiError;
use super::super::state::AppState;

const DEFAULT_LIMIT: i64 = 2000;
const MAX_LIMIT: i64 = 10_000;

#[derive(Debug, Deserialize)]
pub struct FxParams {
    currency: Option<String>,
    from: Option<String>,
    to: Option<String>,
    limit: Option<i64>,
}

/// GET /api/fx?currency=BRL&from=2024-01-01&to=2024-12-31
///
/// Daily USD value of 1 unit of `currency` -- e.g. a `close` of `0.19` for BRL
/// means 1 BRL was worth $0.19 USD that day. Currently supports `BRL`, `CNY`,
/// `JPY`, `EUR`, `KRW`. An unsupported or missing `currency` is a `400`, not a
/// silently empty array.
pub async fn get_fx(
    State(state): State<AppState>,
    Query(params): Query<FxParams>,
) -> Result<Json<Vec<FxRate>>, ApiError> {
    let currency = params
        .currency
        .as_deref()
        .and_then(FxCurrency::parse)
        .ok_or_else(|| {
            ApiError::BadRequest("currency must be one of: BRL, CNY, JPY, EUR, KRW".to_string())
        })?;

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

    let rates = state.fx.history(currency, from, to, limit).await?;
    Ok(Json(rates))
}
