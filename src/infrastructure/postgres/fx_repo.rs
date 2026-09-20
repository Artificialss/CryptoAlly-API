use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;

use crate::domain::models::FxRate;
use crate::domain::ports::FxRepository;

pub struct PgFxRepository {
    pool: &'static PgPool,
}

impl PgFxRepository {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FxRepository for PgFxRepository {
    async fn history(
        &self,
        currency: &str,
        from: NaiveDate,
        to: NaiveDate,
        limit: i64,
    ) -> Result<Vec<FxRate>, sqlx::Error> {
        sqlx::query_as::<_, FxRate>(
            "SELECT date, open, high, low, close \
             FROM fx_rates \
             WHERE currency = $1 AND date BETWEEN $2 AND $3 \
             ORDER BY date ASC LIMIT $4",
        )
        .bind(currency)
        .bind(from)
        .bind(to)
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }
}
