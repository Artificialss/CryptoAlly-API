use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::ports::ApiKeyRepository;
use crate::domain::types::ApiKeyHash;

pub struct PgApiKeyRepository {
    pool: &'static PgPool,
}

impl PgApiKeyRepository {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiKeyRepository for PgApiKeyRepository {
    async fn authenticate(&self, hash: &ApiKeyHash) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM api_keys WHERE key_hash = $1 AND is_active AND revoked_at IS NULL",
        )
        .bind(hash)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some((id,)) => {
                // Best-effort: recording usage never fails the auth check itself --
                // the one narrow write this read-mostly role is granted, for its own
                // bookkeeping only.
                let _ = sqlx::query("UPDATE api_keys SET last_used_at = now() WHERE id = $1")
                    .bind(id)
                    .execute(self.pool)
                    .await;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}
