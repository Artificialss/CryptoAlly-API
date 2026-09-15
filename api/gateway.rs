//! The single Vercel function backing every route (`/api/health`, `/api/assets`,
//! `/api/prices`, `/api/catalog`). A `vercel.json` rewrite sends each of those paths
//! here; internally, one axum `Router` matches on the real request path (routing +
//! auth middleware defined in `cryptoally_api::http`), adapted to `vercel_runtime` via
//! `VercelLayer`.

use std::sync::Arc;

use cryptoally_api::application::services::{AssetService, AuthService, CatalogService, PriceService};
use cryptoally_api::http::{routes, AppState};
use cryptoally_api::infrastructure::postgres::{
    pool, PgApiKeyRepository, PgAssetRepository, PgCatalogRepository, PgPriceRepository,
};
use tower::ServiceBuilder;
use vercel_runtime::axum::VercelLayer;
use vercel_runtime::{run, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let pool = pool::pool().await?;

    let asset_repo = Arc::new(PgAssetRepository::new(pool));
    let price_repo = Arc::new(PgPriceRepository::new(pool));
    let catalog_repo = Arc::new(PgCatalogRepository::new(pool));
    let api_key_repo = Arc::new(PgApiKeyRepository::new(pool));

    let state = AppState {
        assets: Arc::new(AssetService::new(asset_repo.clone())),
        prices: Arc::new(PriceService::new(asset_repo.clone(), price_repo)),
        catalog: Arc::new(CatalogService::new(asset_repo, catalog_repo)),
        auth: Arc::new(AuthService::new(api_key_repo)),
        pool,
    };

    let router = routes::build(state);
    let service = ServiceBuilder::new().layer(VercelLayer::new()).service(router);

    run(service).await
}
