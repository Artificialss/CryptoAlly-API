use once_cell::sync::OnceCell;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use vercel_runtime::Error;

static POOL: OnceCell<PgPool> = OnceCell::new();

/// Connects using a dedicated, read-only database credential (`API_DATABASE_URL`),
/// scoped to exactly the access this API needs -- distinct from any credential used
/// by data ingestion or schema administration, both of which run entirely out-of-band
/// from this API. Deliberately not `DATABASE_URL`: that name is owned by the hosting
/// Postgres provider's own integration and points at the full-privilege owner role.
pub async fn pool() -> Result<&'static PgPool, Error> {
    if let Some(p) = POOL.get() {
        return Ok(p);
    }
    let url = std::env::var("API_DATABASE_URL").map_err(|_| "API_DATABASE_URL is not set")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&url).await?;
    Ok(POOL.get_or_init(|| pool))
}
