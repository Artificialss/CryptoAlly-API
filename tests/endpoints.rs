//! Endpoint tests against the real axum `Router`, driven through fake in-memory
//! repositories instead of a live Postgres database. This is the payoff of the
//! ports-and-adapters split in `domain::ports`: the HTTP layer (routing, auth
//! middleware, status codes, JSON shape) gets exercised for real, without a database
//! anywhere in the test process.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::NaiveDate;
use http_body_util::BodyExt;
use tower::ServiceExt;

use cryptoally_api::application::services::{AssetService, AuthService, CatalogService, PriceService};
use cryptoally_api::domain::models::{Asset, AssetLookup, AssetQuery, CatalogEntry, PriceBar};
use cryptoally_api::domain::ports::{ApiKeyRepository, AssetRepository, AssetResolver, CatalogRepository, PriceRepository};
use cryptoally_api::domain::types::{ApiKeyHash, AssetId, Lang};
use cryptoally_api::http::{routes, AppState};

const TEST_KEY: &str = "test-key-plaintext";

// --- Fakes: same ports the real Postgres repositories implement -----------------

struct FakeAssetRepo {
    assets: Vec<Asset>,
}

#[async_trait]
impl AssetRepository for FakeAssetRepo {
    async fn list(&self, query: &AssetQuery) -> Result<Vec<Asset>, sqlx::Error> {
        Ok(self
            .assets
            .iter()
            .filter(|a| query.filter.market.as_deref().is_none_or(|m| a.market == m))
            .filter(|a| query.filter.asset_type.as_deref().is_none_or(|t| a.asset_type == t))
            .cloned()
            .collect())
    }
}

#[async_trait]
impl AssetResolver for FakeAssetRepo {
    async fn resolve(&self, lookup: &AssetLookup) -> Result<Option<AssetId>, sqlx::Error> {
        match lookup {
            AssetLookup::Id(id) => Ok(self.assets.iter().find(|a| a.id == *id).map(|a| a.id)),
            AssetLookup::MarketSymbol { market, symbol } => Ok(self
                .assets
                .iter()
                .find(|a| &a.market == market && &a.symbol == symbol)
                .map(|a| a.id)),
        }
    }
}

struct FakePriceRepo {
    bars: HashMap<i64, Vec<PriceBar>>,
}

#[async_trait]
impl PriceRepository for FakePriceRepo {
    async fn history(
        &self,
        asset_id: AssetId,
        _from: NaiveDate,
        _to: NaiveDate,
        limit: i64,
    ) -> Result<Vec<PriceBar>, sqlx::Error> {
        Ok(self
            .bars
            .get(&asset_id.0)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .take(limit as usize)
            .collect())
    }
}

struct FakeCatalogRepo {
    entries: HashMap<i64, CatalogEntry>,
}

#[async_trait]
impl CatalogRepository for FakeCatalogRepo {
    async fn get(&self, asset_id: AssetId, _lang: Lang) -> Result<Option<CatalogEntry>, sqlx::Error> {
        Ok(self.entries.get(&asset_id.0).cloned())
    }
}

struct FakeApiKeyRepo {
    valid_hash: ApiKeyHash,
}

#[async_trait]
impl ApiKeyRepository for FakeApiKeyRepo {
    async fn authenticate(&self, hash: &ApiKeyHash) -> Result<bool, sqlx::Error> {
        Ok(*hash == self.valid_hash)
    }
}

// --- Test app construction --------------------------------------------------------

fn asset(id: i64, symbol: &str, market: &str, asset_type: &str) -> Asset {
    Asset {
        id: AssetId(id),
        symbol: symbol.to_string(),
        name: format!("{symbol} Name"),
        asset_type: asset_type.to_string(),
        currency: "USD".to_string(),
        market: market.to_string(),
        market_name: Some(market.to_string()),
        index_name: None,
    }
}

async fn test_app() -> axum::Router {
    let valid_key = cryptoally_api::domain::types::ApiKeyPlaintext(TEST_KEY.to_string());
    let valid_hash = valid_key.hash();

    let assets = vec![
        asset(1, "AAPL", "us", "stock"),
        asset(2, "bitcoin", "crypto", "crypto"),
    ];

    let asset_repo = Arc::new(FakeAssetRepo { assets });

    let mut bars = HashMap::new();
    bars.insert(
        2,
        vec![PriceBar {
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            open: None,
            high: None,
            low: None,
            close: None,
            volume: None,
        }],
    );
    let price_repo = Arc::new(FakePriceRepo { bars });

    let mut entries = HashMap::new();
    entries.insert(
        2,
        CatalogEntry {
            asset_id: AssetId(2),
            founders: Some(vec!["Satoshi Nakamoto".to_string()]),
            founded: Some("2009".to_string()),
            history: Some("The first cryptocurrency.".to_string()),
            official_website: None,
            logo_url: None,
            info_url: None,
            issuer: None,
            index_provider: None,
            inception_date: None,
        },
    );
    let catalog_repo = Arc::new(FakeCatalogRepo { entries });

    let api_key_repo = Arc::new(FakeApiKeyRepo { valid_hash });

    let state = AppState {
        assets: Arc::new(AssetService::new(asset_repo.clone())),
        prices: Arc::new(PriceService::new(asset_repo.clone(), price_repo)),
        catalog: Arc::new(CatalogService::new(asset_repo, catalog_repo)),
        auth: Arc::new(AuthService::new(api_key_repo)),
        pool: test_pool_placeholder(),
    };

    routes::build(state)
}

/// `/api/health` is the only route that touches `state.pool` directly (a real
/// liveness check has no meaningful fake), so it's deliberately excluded from these
/// tests -- a pool that's never connected is fine as long as nothing calls it.
fn test_pool_placeholder() -> &'static sqlx::PgPool {
    use std::sync::OnceLock;
    static POOL: OnceLock<sqlx::PgPool> = OnceLock::new();
    POOL.get_or_init(|| {
        sqlx::PgPool::connect_lazy("postgres://placeholder/placeholder")
            .expect("lazy connect never actually dials out")
    })
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

// --- Tests -------------------------------------------------------------------------

#[tokio::test]
async fn assets_without_api_key_is_unauthorized() {
    let app = test_app().await;
    let req = Request::builder().uri("/api/assets").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn assets_with_wrong_api_key_is_unauthorized() {
    let app = test_app().await;
    let req = Request::builder()
        .uri("/api/assets")
        .header("x-api-key", "not-the-right-key")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn assets_with_valid_key_lists_assets() {
    let app = test_app().await;
    let req = Request::builder()
        .uri("/api/assets")
        .header("x-api-key", TEST_KEY)
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn assets_filters_by_market() {
    let app = test_app().await;
    let req = Request::builder()
        .uri("/api/assets?market=crypto")
        .header("x-api-key", TEST_KEY)
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["symbol"], "bitcoin");
}

#[tokio::test]
async fn prices_without_lookup_params_is_bad_request() {
    let app = test_app().await;
    let req = Request::builder()
        .uri("/api/prices")
        .header("x-api-key", TEST_KEY)
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn prices_for_nonexistent_asset_is_not_found() {
    let app = test_app().await;
    let req = Request::builder()
        .uri("/api/prices?asset_id=999")
        .header("x-api-key", TEST_KEY)
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    // Distinct from the 400 above: params were well-formed, the asset just doesn't
    // exist -- this is the bug the old (pre-clean-architecture) handler had, where
    // both cases returned the same "provide either..." 400.
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn prices_by_market_and_symbol_returns_bars() {
    let app = test_app().await;
    let req = Request::builder()
        .uri("/api/prices?market=crypto&symbol=bitcoin")
        .header("x-api-key", TEST_KEY)
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn catalog_returns_translated_fields() {
    let app = test_app().await;
    let req = Request::builder()
        .uri("/api/catalog?asset_id=2")
        .header("x-api-key", TEST_KEY)
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    assert_eq!(body["history"], "The first cryptocurrency.");
}

#[tokio::test]
async fn catalog_for_asset_with_no_entry_is_not_found() {
    let app = test_app().await;
    let req = Request::builder()
        .uri("/api/catalog?asset_id=1")
        .header("x-api-key", TEST_KEY)
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn catalog_for_nonexistent_asset_id_is_not_found() {
    // Distinct from the test above: asset_id=1 exists but has no catalog row.
    // asset_id=999 doesn't exist as an asset at all -- both share the same
    // AssetResolver::resolve() path as /api/prices, so this locks in the fix for
    // the real Postgres adapter's Id branch, which used to trust any id unchecked.
    let app = test_app().await;
    let req = Request::builder()
        .uri("/api/catalog?asset_id=999")
        .header("x-api-key", TEST_KEY)
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn health_is_public_no_api_key_needed() {
    // health.rs touches the real pool, which the fake app is deliberately not wired
    // to connect anywhere -- assert only that the route matches and does NOT return
    // 401 (the thing auth middleware would produce), not that it returns 200.
    let app = test_app().await;
    let req = Request::builder().uri("/api/health").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::UNAUTHORIZED);
}
