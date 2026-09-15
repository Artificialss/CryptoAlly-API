<div align="center">
  <img src=".github/logo.svg" alt="CryptoAlly logo" width="120" />

  # CryptoAlly API
</div>

The Rust API behind [CryptoAlly](https://www.cryptoally.dev) — historical price and
catalog data for 1,864 assets across crypto, stablecoins, US stocks/ETFs, commodities,
and 27 international stock markets. Deployed on Vercel's official Rust runtime
(`vercel_runtime`, public beta). This API is the only thing that ever talks to the
underlying Postgres database directly — no client, browser, or mobile app connects to
it, and every non-public endpoint requires an API key.

This repo is the public face of the project: the API's source, its docs, and the
engineering behind it (homepage, analytics, SEO). The database schema, ingestion
pipeline, and raw data archive that feed this API live in a separate, private repo.

## Layout

Vercel's Rust runtime convention: `Cargo.toml` lives at the directory Vercel builds from
(this one), each file under `api/` is a separate `[[bin]]` target that becomes its own
serverless function, and `src/lib.rs` holds what they share (the DB pool, row types,
response helpers).

```
├── Cargo.toml
├── vercel.json      # rewrites "/" -> "/api/index", "/robots.txt", "/sitemap.xml"
├── SEO.md           # SEO strategy for the homepage
├── ANALYTICS.md     # analytics page map: what's tracked, event names, and why
├── src/lib.rs       # DB pool (cached per warm instance), row types, JSON helpers
├── static/index.html  # the homepage's markup (embedded into api/index.rs at compile time)
└── api/
    ├── index.rs     # GET /                    -- homepage: what this is, endpoints, auth
    ├── health.rs    # GET /api/health          -- liveness + DB connectivity check
    ├── assets.rs    # GET /api/assets          -- list/filter/search assets
    ├── prices.rs    # GET /api/prices          -- daily price history for one asset
    ├── robots.rs    # GET /robots.txt
    └── sitemap.rs   # GET /sitemap.xml
```

## Authentication

`/api/assets` and `/api/prices` require an `x-api-key` header. `/api/health` is
intentionally public (a liveness probe). Keys are hashed (SHA-256) server-side — the
plaintext is shown once, at creation, and never stored, and issuing one isn't
self-service yet. Visit [cryptoally.app](https://cryptoally.app) to request one.

An invalid or missing key gets a `401`.

## Endpoints

**`GET /api/health`** — `{"status":"ok","database":"ok"}`, or 503 if the database is
unreachable. No API key required.

**`GET /api/assets`** — screener-style asset list.
Query params (all optional): `type` (`crypto`/`stablecoin`/`stock`/`etf`/`index`/`commodity`),
`market` (a market slug, e.g. `japan`, `crypto`, `us`), `search` (matches symbol/name),
`limit` (default 50, max 500), `offset`.

**`GET /api/prices`** — daily OHLCV + `close_usd` history for one asset.
Resolve the asset either with `asset_id` directly, or with both `market` (slug) and
`symbol` (the asset's ticker, e.g. `AAPL`, or a crypto id like `bitcoin`). Symbols alone
aren't unique across markets — the same ticker can exist on two different exchanges — so
`market` is required alongside `symbol`. Optional `from`/`to` (`YYYY-MM-DD`, default full
history) and `limit` (default 2000, max 10000).

## Environment

| Var | Used by |
|---|---|
| `API_DATABASE_URL` | The live API's runtime connection — a read-only Postgres role, every environment |

Deliberately not `DATABASE_URL` — that name is reserved for the database's full-privilege
owner role, which this API never uses.

## Local development

```bash
rustup default stable   # if you don't already have a Rust toolchain
cargo build --release   # compiles all functions; catches errors before deploying
vercel dev               # runs the functions locally against Vercel's dev server
```

## Deploy

```bash
vercel deploy --prod
```

Or push to the branch connected to the Vercel project for an automatic deployment.

## Analytics, Speed Insights, and SEO

Web Analytics and Speed Insights are enabled on the Vercel project and wired into the
homepage via the plain-HTML script integration (no npm package — this project has no
Node build step). See [`ANALYTICS.md`](ANALYTICS.md) for the full event map (what's
tracked, event names, and why) and [`SEO.md`](SEO.md) for the SEO strategy behind the
homepage's meta tags, structured data, and `robots.txt`/`sitemap.xml`.

## About

Built by [Artificialss](https://artificialss.ai). CryptoAlly's data is free to use with
attribution — see the [Usage section](https://www.cryptoally.dev/#usage) on the live API
homepage for the full policy.
