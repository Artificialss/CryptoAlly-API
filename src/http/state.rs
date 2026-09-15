use std::sync::Arc;

use sqlx::PgPool;

use crate::application::services::{AssetService, AuthService, CatalogService, PriceService};

/// Shared application state, wired once at startup and cloned (cheaply -- it's all
/// `Arc`s/references) into every request. Handlers depend on this, never on the
/// concrete Postgres repositories directly -- see `domain::ports` for why.
#[derive(Clone)]
pub struct AppState {
    pub assets: Arc<AssetService>,
    pub prices: Arc<PriceService>,
    pub catalog: Arc<CatalogService>,
    pub auth: Arc<AuthService>,
    /// Used directly (not through a repository port) only by the health check --
    /// a liveness probe's entire job is verifying the real connection works, so
    /// injecting a fake here would test nothing meaningful.
    pub pool: &'static PgPool,
}
