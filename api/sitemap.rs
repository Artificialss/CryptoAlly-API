//! GET /sitemap.xml (rewritten from /sitemap.xml -- see vercel.json)
//!
//! One-URL sitemap -- this is a single-page site. Anchor sections (#markets, #usage,
//! ...) aren't separately crawlable pages, so they don't get their own <url> entries.

use vercel_runtime::{run, service_fn, Error, Request, Response, ResponseBody};

const BODY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://www.cryptoally.dev/</loc>
    <changefreq>daily</changefreq>
    <priority>1.0</priority>
  </url>
</urlset>
"#;

pub async fn handler(_req: Request) -> Result<Response<ResponseBody>, Error> {
    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/xml; charset=utf-8")
        .header("cache-control", "public, max-age=86400")
        .body(ResponseBody::from(BODY))?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
