//! Repository traits (the "ports" in ports-and-adapters/clean architecture) -- the
//! application layer depends only on these, never on `sqlx` or Postgres directly.
//! `infrastructure::postgres` provides the real implementation; tests provide
//! in-memory fakes implementing the same trait, so endpoint tests never need a live
//! database.

use async_trait::async_trait;

use super::models::{Asset, AssetLookup, AssetQuery, CatalogEntry, PriceBar};
use super::types::{ApiKeyHash, AssetId, Lang};

#[async_trait]
pub trait AssetRepository: Send + Sync {
    async fn list(&self, query: &AssetQuery) -> Result<Vec<Asset>, sqlx::Error>;
}

/// Resolves an `AssetLookup` to a concrete `AssetId`. Its own trait (rather than a
/// method on `PriceRepository`/`CatalogRepository`) because both of those need the
/// exact same resolution logic and shouldn't each reimplement or duplicate it.
#[async_trait]
pub trait AssetResolver: Send + Sync {
    async fn resolve(&self, lookup: &AssetLookup) -> Result<Option<AssetId>, sqlx::Error>;
}

#[async_trait]
pub trait PriceRepository: Send + Sync {
    async fn history(
        &self,
        asset_id: AssetId,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
        limit: i64,
    ) -> Result<Vec<PriceBar>, sqlx::Error>;
}

#[async_trait]
pub trait CatalogRepository: Send + Sync {
    async fn get(&self, asset_id: AssetId, lang: Lang) -> Result<Option<CatalogEntry>, sqlx::Error>;
}

#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    /// Checks whether `hash` matches an active, non-revoked key. Implementations
    /// should record usage (e.g. `last_used_at`) as a best-effort side effect that
    /// never fails the check itself.
    async fn authenticate(&self, hash: &ApiKeyHash) -> Result<bool, sqlx::Error>;
}
