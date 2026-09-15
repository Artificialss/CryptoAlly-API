//! Postgres-backed `AssetRepository` + `AssetResolver`. Every query here is
//! parameterized (`QueryBuilder::push_bind` / `sqlx::query_as` positional binds) --
//! no user input is ever interpolated into SQL text, which is what actually rules out
//! SQL injection, not any particular crate choice by itself.

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::domain::models::{Asset, AssetLookup, AssetQuery};
use crate::domain::ports::{AssetRepository, AssetResolver};
use crate::domain::types::AssetId;

pub struct PgAssetRepository {
    pool: &'static PgPool,
}

impl PgAssetRepository {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetRepository for PgAssetRepository {
    async fn list(&self, query: &AssetQuery) -> Result<Vec<Asset>, sqlx::Error> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT a.id, a.symbol, a.name, a.asset_type::text AS asset_type, a.currency, \
                    m.slug AS market, COALESCE(m.name ->> ",
        );
        qb.push_bind(query.lang.code());
        qb.push(", m.name ->> 'en') AS market_name, a.index_name \
             FROM assets a JOIN markets m ON m.id = a.market_id WHERE a.is_active");

        if let Some(asset_type) = &query.filter.asset_type {
            qb.push(" AND a.asset_type = ").push_bind(asset_type).push("::asset_type");
        }
        if let Some(market) = &query.filter.market {
            qb.push(" AND m.slug = ").push_bind(market);
        }
        if let Some(search) = &query.filter.search {
            let pattern = format!("%{}%", search);
            qb.push(" AND (a.symbol ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR a.name ILIKE ")
                .push_bind(pattern)
                .push(")");
        }

        qb.push(" ORDER BY a.symbol LIMIT ")
            .push_bind(query.pagination.limit)
            .push(" OFFSET ")
            .push_bind(query.pagination.offset);

        qb.build_query_as::<AssetRow>().fetch_all(self.pool).await.map(|rows| {
            rows.into_iter().map(Asset::from).collect()
        })
    }
}

#[async_trait]
impl AssetResolver for PgAssetRepository {
    async fn resolve(&self, lookup: &AssetLookup) -> Result<Option<AssetId>, sqlx::Error> {
        match lookup {
            // A bare `asset_id` must still be checked against the table -- an
            // unvalidated id would let e.g. /api/prices?asset_id=999999 silently
            // return an empty-but-200 result instead of a 404, unlike the
            // market+symbol path below (which was always a real existence query).
            AssetLookup::Id(id) => {
                let row: Option<(AssetId,)> =
                    sqlx::query_as("SELECT id FROM assets WHERE id = $1")
                        .bind(id)
                        .fetch_optional(self.pool)
                        .await?;
                Ok(row.map(|(id,)| id))
            }
            AssetLookup::MarketSymbol { market, symbol } => {
                let row: Option<(AssetId,)> = sqlx::query_as(
                    "SELECT a.id FROM assets a JOIN markets m ON m.id = a.market_id \
                     WHERE m.slug = $1 AND a.external_id = $2",
                )
                .bind(market)
                .bind(symbol)
                .fetch_optional(self.pool)
                .await?;
                Ok(row.map(|(id,)| id))
            }
        }
    }
}

/// Raw row shape as it comes back from the query above -- kept separate from the
/// `Asset` domain model so the sqlx `FromRow` derive stays an infrastructure detail,
/// not something the domain layer needs to know about.
#[derive(sqlx::FromRow)]
struct AssetRow {
    id: AssetId,
    symbol: String,
    name: String,
    asset_type: String,
    currency: String,
    market: String,
    market_name: Option<String>,
    index_name: Option<String>,
}

impl From<AssetRow> for Asset {
    fn from(row: AssetRow) -> Self {
        Asset {
            id: row.id,
            symbol: row.symbol,
            name: row.name,
            asset_type: row.asset_type,
            currency: row.currency,
            market: row.market,
            market_name: row.market_name,
            index_name: row.index_name,
        }
    }
}
