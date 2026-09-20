//! Thin orchestration over the domain ports. Each service depends only on the trait
//! objects it needs (`Arc<dyn ...>`), never on a concrete Postgres type -- swapping the
//! real repository for a test fake means constructing the service differently, not
//! changing its code.

use std::sync::Arc;

use chrono::NaiveDate;

use crate::domain::models::{Asset, AssetLookup, AssetQuery, CatalogEntry, FxRate, PriceBar};
use crate::domain::ports::{
    ApiKeyRepository, AssetRepository, AssetResolver, CatalogRepository, FxRepository, PriceRepository,
};
use crate::domain::types::{ApiKeyPlaintext, FxCurrency};

pub struct AssetService {
    repo: Arc<dyn AssetRepository>,
}

impl AssetService {
    pub fn new(repo: Arc<dyn AssetRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(&self, query: AssetQuery) -> Result<Vec<Asset>, sqlx::Error> {
        self.repo.list(&query).await
    }
}

pub struct PriceService {
    resolver: Arc<dyn AssetResolver>,
    repo: Arc<dyn PriceRepository>,
}

impl PriceService {
    pub fn new(resolver: Arc<dyn AssetResolver>, repo: Arc<dyn PriceRepository>) -> Self {
        Self { resolver, repo }
    }

    /// `Ok(None)` means the lookup didn't resolve to any asset (a 404 at the HTTP
    /// layer); `Ok(Some(vec![]))` means the asset exists but has no bars in range.
    pub async fn history(
        &self,
        lookup: AssetLookup,
        from: NaiveDate,
        to: NaiveDate,
        limit: i64,
    ) -> Result<Option<Vec<PriceBar>>, sqlx::Error> {
        let Some(asset_id) = self.resolver.resolve(&lookup).await? else {
            return Ok(None);
        };
        Ok(Some(self.repo.history(asset_id, from, to, limit).await?))
    }
}

pub struct CatalogService {
    resolver: Arc<dyn AssetResolver>,
    repo: Arc<dyn CatalogRepository>,
}

impl CatalogService {
    pub fn new(resolver: Arc<dyn AssetResolver>, repo: Arc<dyn CatalogRepository>) -> Self {
        Self { resolver, repo }
    }

    pub async fn get(
        &self,
        lookup: AssetLookup,
        lang: crate::domain::types::Lang,
    ) -> Result<Option<CatalogEntry>, sqlx::Error> {
        let Some(asset_id) = self.resolver.resolve(&lookup).await? else {
            return Ok(None);
        };
        self.repo.get(asset_id, lang).await
    }
}

pub struct FxService {
    repo: Arc<dyn FxRepository>,
}

impl FxService {
    pub fn new(repo: Arc<dyn FxRepository>) -> Self {
        Self { repo }
    }

    pub async fn history(
        &self,
        currency: FxCurrency,
        from: NaiveDate,
        to: NaiveDate,
        limit: i64,
    ) -> Result<Vec<FxRate>, sqlx::Error> {
        self.repo.history(currency.code(), from, to, limit).await
    }
}

pub struct AuthService {
    repo: Arc<dyn ApiKeyRepository>,
}

impl AuthService {
    pub fn new(repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repo }
    }

    pub async fn verify(&self, plaintext: &ApiKeyPlaintext) -> bool {
        let hash = plaintext.hash();
        self.repo.authenticate(&hash).await.unwrap_or(false)
    }
}
