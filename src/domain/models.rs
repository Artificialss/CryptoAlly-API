//! Domain entities and query parameter objects. These are the shapes the application
//! and infrastructure layers speak in -- independent of both the HTTP transport (axum
//! request/response types) and the storage engine (sqlx rows), which is the whole
//! point of a domain layer: swap either one out and these stay the same.

use super::types::{AssetId, Lang, Pagination};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Asset {
    pub id: AssetId,
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub currency: String,
    pub market: String,
    pub market_name: Option<String>,
    pub index_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PriceBar {
    pub date: NaiveDate,
    pub open: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub close: Option<Decimal>,
    pub volume: Option<i64>,
    pub close_usd: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CatalogEntry {
    pub asset_id: AssetId,
    pub founders: Option<Vec<String>>,
    pub founded: Option<String>,
    pub history: Option<String>,
    pub official_website: Option<String>,
    pub logo_url: Option<String>,
    pub info_url: Option<String>,
    pub issuer: Option<String>,
    pub index_provider: Option<String>,
    pub inception_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Default)]
pub struct AssetFilter {
    pub asset_type: Option<String>,
    pub market: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AssetQuery {
    pub filter: AssetFilter,
    pub lang: Lang,
    pub pagination: Pagination,
}

/// How a single asset is targeted in a request: directly by id, or by the
/// (market, symbol) pair -- a ticker alone isn't unique across markets, since the
/// same symbol can exist on more than one exchange.
#[derive(Debug, Clone)]
pub enum AssetLookup {
    Id(AssetId),
    MarketSymbol { market: String, symbol: String },
}
