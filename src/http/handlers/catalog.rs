use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use crate::domain::models::{AssetLookup, CatalogEntry};
use crate::domain::types::{AssetId, Lang};

use super::super::error::ApiError;
use super::super::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CatalogParams {
    asset_id: Option<i64>,
    market: Option<String>,
    symbol: Option<String>,
    lang: Option<String>,
}

fn lookup_from_params(params: &CatalogParams) -> Result<AssetLookup, ApiError> {
    if let Some(id) = params.asset_id {
        return Ok(AssetLookup::Id(AssetId(id)));
    }
    if let (Some(market), Some(symbol)) = (&params.market, &params.symbol) {
        return Ok(AssetLookup::MarketSymbol { market: market.clone(), symbol: symbol.clone() });
    }
    Err(ApiError::BadRequest("provide either asset_id, or both market and symbol".to_string()))
}

/// GET /api/catalog?asset_id=1&lang=es
/// GET /api/catalog?market=crypto&symbol=bitcoin&lang=es
///
/// Founders, founding date, a short history, and official links for one asset.
/// `founded`/`history` are resolved server-side to a single string in the requested
/// `?lang=` (es/pt/ja/zh; default/fallback en), not the raw i18n object.
pub async fn get_catalog(
    State(state): State<AppState>,
    Query(params): Query<CatalogParams>,
) -> Result<Json<CatalogEntry>, ApiError> {
    let lookup = lookup_from_params(&params)?;
    let lang = Lang::parse_or_default(params.lang.as_deref());

    match state.catalog.get(lookup, lang).await? {
        Some(entry) => Ok(Json(entry)),
        None => Err(ApiError::NotFound("no catalog entry for this asset".to_string())),
    }
}
