//! GET /api/index (rewritten from "/" -- see vercel.json)
//!
//! The API's homepage: what it is, live markets/stocks/stablecoins tables (queried the
//! same way any other endpoint would, via `api_readonly`), fetching rules, usage/
//! attribution, and the endpoint reference. Public -- no `x-api-key` required, same as
//! /api/health.

use cryptoally_api::pool;
use sqlx::PgPool;
use vercel_runtime::{run, service_fn, Error, Request, Response, ResponseBody};

const TEMPLATE: &str = include_str!("../static/index.html");

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

async fn markets_rows(pool: &PgPool) -> String {
    let rows: Result<Vec<(String, String, Option<String>, String, Option<String>, i64)>, _> =
        sqlx::query_as(
            "SELECT m.slug, m.name, m.exchange, m.currency, m.country_code, count(a.id)
             FROM markets m LEFT JOIN assets a ON a.market_id = m.id
             WHERE m.slug != 'crypto'
             GROUP BY m.id ORDER BY count(a.id) DESC, m.name",
        )
        .fetch_all(pool)
        .await;

    match rows {
        Ok(rows) if !rows.is_empty() => rows
            .into_iter()
            .map(|(_, name, exchange, currency, country, count)| {
                format!(
                    "<tr><td><strong>{}</strong></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    escape(&name),
                    exchange.as_deref().map(escape).unwrap_or_else(|| "—".into()),
                    escape(&currency),
                    country.as_deref().map(escape).unwrap_or_else(|| "—".into()),
                    count,
                )
            })
            .collect(),
        Ok(_) => "<tr><td colspan=\"5\" class=\"empty-note\">No markets found.</td></tr>".into(),
        Err(_) => {
            "<tr><td colspan=\"5\" class=\"empty-note\">Unable to load live data right now.</td></tr>"
                .into()
        }
    }
}

async fn stock_rows(pool: &PgPool) -> String {
    let rows: Result<Vec<(String, String, String)>, _> = sqlx::query_as(
        "SELECT a.symbol, a.name, a.asset_type::text
         FROM assets a JOIN markets m ON m.id = a.market_id
         WHERE m.slug = 'us' AND a.is_active
         ORDER BY a.asset_type, a.symbol",
    )
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) if !rows.is_empty() => rows
            .into_iter()
            .map(|(symbol, name, asset_type)| {
                format!(
                    "<tr><td class=\"symbol\">{}</td><td>{}</td><td><span class=\"type-tag type-{}\">{}</span></td></tr>",
                    escape(&symbol),
                    escape(&name),
                    asset_type,
                    asset_type,
                )
            })
            .collect(),
        Ok(_) => "<tr><td colspan=\"3\" class=\"empty-note\">No stocks found.</td></tr>".into(),
        Err(_) => {
            "<tr><td colspan=\"3\" class=\"empty-note\">Unable to load live data right now.</td></tr>"
                .into()
        }
    }
}

async fn stablecoin_rows(pool: &PgPool) -> String {
    let rows: Result<Vec<(String, String)>, _> = sqlx::query_as(
        "SELECT a.symbol, a.name
         FROM assets a JOIN markets m ON m.id = a.market_id
         WHERE m.slug = 'crypto' AND a.asset_type = 'stablecoin' AND a.is_active
         ORDER BY a.symbol",
    )
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) if !rows.is_empty() => rows
            .into_iter()
            .map(|(symbol, name)| {
                format!(
                    "<tr><td class=\"symbol\">{}</td><td>{}</td></tr>",
                    escape(&symbol),
                    escape(&name),
                )
            })
            .collect(),
        Ok(_) => "<tr><td colspan=\"2\" class=\"empty-note\">No stablecoins found.</td></tr>".into(),
        Err(_) => {
            "<tr><td colspan=\"2\" class=\"empty-note\">Unable to load live data right now.</td></tr>"
                .into()
        }
    }
}

pub async fn handler(_req: Request) -> Result<Response<ResponseBody>, Error> {
    let page = match pool().await {
        Ok(p) => {
            let (markets, stocks, stablecoins) =
                tokio::join!(markets_rows(p), stock_rows(p), stablecoin_rows(p));
            TEMPLATE
                .replace("{{MARKETS_ROWS}}", &markets)
                .replace("{{STOCK_ROWS}}", &stocks)
                .replace("{{STABLECOIN_ROWS}}", &stablecoins)
        }
        Err(_) => TEMPLATE
            .replace(
                "{{MARKETS_ROWS}}",
                "<tr><td colspan=\"5\" class=\"empty-note\">Unable to load live data right now.</td></tr>",
            )
            .replace(
                "{{STOCK_ROWS}}",
                "<tr><td colspan=\"3\" class=\"empty-note\">Unable to load live data right now.</td></tr>",
            )
            .replace(
                "{{STABLECOIN_ROWS}}",
                "<tr><td colspan=\"2\" class=\"empty-note\">Unable to load live data right now.</td></tr>",
            ),
    };

    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/html; charset=utf-8")
        .header("cache-control", "no-cache")
        .body(ResponseBody::from(page))?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
