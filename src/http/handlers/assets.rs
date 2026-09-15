use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use crate::domain::models::{Asset, AssetFilter, AssetQuery};
use crate::domain::types::{Lang, Pagination};

use super::super::error::ApiError;
use super::super::state::AppState;

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 500;

#[derive(Debug, Deserialize)]
pub struct AssetsParams {
    #[serde(rename = "type")]
    asset_type: Option<String>,
    market: Option<String>,
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    lang: Option<String>,
}

/// GET /api/assets?type=crypto&market=japan&search=apple&limit=50&offset=0&lang=es
///
/// Screener-style asset list. `market_name` is localized per `?lang=` (es/pt/ja/zh;
/// default/fallback en) -- the `market` slug itself is unaffected by language.
pub async fn list_assets(
    State(state): State<AppState>,
    Query(params): Query<AssetsParams>,
) -> Result<Json<Vec<Asset>>, ApiError> {
    let query = AssetQuery {
        filter: AssetFilter {
            asset_type: params.asset_type,
            market: params.market,
            search: params.search,
        },
        lang: Lang::parse_or_default(params.lang.as_deref()),
        pagination: Pagination::new(params.limit, params.offset, DEFAULT_LIMIT, MAX_LIMIT),
    };

    let assets = state.assets.list(query).await?;
    Ok(Json(assets))
}
