use cryptoally_api::{error_response, json_response_public, pool};
use serde::Serialize;
use vercel_runtime::{run, service_fn, Error, Request, Response};

#[derive(Serialize)]
struct Health {
    status: &'static str,
    database: &'static str,
}

pub async fn handler(_req: Request) -> Result<Response<serde_json::Value>, Error> {
    match pool().await {
        Ok(p) => match sqlx::query("SELECT 1").execute(p).await {
            Ok(_) => json_response_public(200, &Health { status: "ok", database: "ok" }),
            Err(_) => error_response(503, "database query failed"),
        },
        Err(_) => error_response(503, "database unreachable"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
