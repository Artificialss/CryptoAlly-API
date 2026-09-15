use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::models::CatalogEntry;
use crate::domain::ports::CatalogRepository;
use crate::domain::types::{AssetId, Lang};

pub struct PgCatalogRepository {
    pool: &'static PgPool,
}

impl PgCatalogRepository {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatalogRepository for PgCatalogRepository {
    async fn get(&self, asset_id: AssetId, lang: Lang) -> Result<Option<CatalogEntry>, sqlx::Error> {
        sqlx::query_as::<_, CatalogEntry>(
            "SELECT asset_id, founders, \
                    COALESCE(founded ->> $2, founded ->> 'en') AS founded, \
                    COALESCE(history ->> $2, history ->> 'en') AS history, \
                    official_website, logo_url, info_url, issuer, index_provider, inception_date \
             FROM asset_catalog WHERE asset_id = $1",
        )
        .bind(asset_id)
        .bind(lang.code())
        .fetch_optional(self.pool)
        .await
    }
}
