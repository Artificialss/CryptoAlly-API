use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;

use crate::domain::models::PriceBar;
use crate::domain::ports::PriceRepository;
use crate::domain::types::AssetId;

pub struct PgPriceRepository {
    pool: &'static PgPool,
}

impl PgPriceRepository {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PriceRepository for PgPriceRepository {
    async fn history(
        &self,
        asset_id: AssetId,
        from: NaiveDate,
        to: NaiveDate,
        limit: i64,
    ) -> Result<Vec<PriceBar>, sqlx::Error> {
        sqlx::query_as::<_, PriceBar>(
            "SELECT date, open, high, low, close, volume \
             FROM daily_prices \
             WHERE asset_id = $1 AND date BETWEEN $2 AND $3 \
             ORDER BY date ASC LIMIT $4",
        )
        .bind(asset_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }
}
