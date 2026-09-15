//! GET /robots.txt (rewritten from /robots.txt -- see vercel.json)

use vercel_runtime::{run, service_fn, Error, Request, Response, ResponseBody};

const BODY: &str = "User-agent: *\nAllow: /\n\nSitemap: https://www.cryptoally.dev/sitemap.xml\n";

pub async fn handler(_req: Request) -> Result<Response<ResponseBody>, Error> {
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain; charset=utf-8")
        .header("cache-control", "public, max-age=86400")
        .body(ResponseBody::from(BODY))?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
