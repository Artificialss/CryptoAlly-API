use axum::middleware;
use axum::routing::get;
use axum::Router;

use super::auth::require_api_key;
use super::handlers::{assets, catalog, fx, health, prices};
use super::state::AppState;

/// Builds the full router: `/api/health` public, everything else behind
/// `require_api_key`. A single `Router<AppState>`, deployed as one Vercel function
/// (see `api/gateway.rs`) -- this is what lets auth live as one real middleware layer
/// instead of being copy-pasted into every handler.
pub fn build(state: AppState) -> Router {
    let public = Router::new().route("/api/health", get(health::health));

    let protected = Router::new()
        .route("/api/assets", get(assets::list_assets))
        .route("/api/prices", get(prices::get_prices))
        .route("/api/catalog", get(catalog::get_catalog))
        .route("/api/fx", get(fx::get_fx))
        .layer(middleware::from_fn_with_state(state.clone(), require_api_key));

    public.merge(protected).with_state(state)
}
