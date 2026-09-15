<div align="center">
  <img src=".github/logo.svg" alt="CryptoAlly logo" width="120" />

  # CryptoAlly API
</div>

<p align="center">
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="Rust" src="https://img.shields.io/badge/rust-2021-orange.svg">
  <img alt="Status" src="https://img.shields.io/badge/status-live-brightgreen.svg">
  <a href="https://www.cryptoally.dev"><img alt="API status" src="https://img.shields.io/badge/api-www.cryptoally.dev-informational.svg"></a>
</p>

Historical price and catalog data for **1,864 assets** — crypto, stablecoins, US
stocks/ETFs, commodities, and 27 international stock markets — served as a REST/JSON
API. Built in Rust, deployed on [Vercel's official Rust runtime](https://vercel.com/docs/functions/runtimes/rust).

This repository is the **public engineering reference** for the API: its real source
(architecture, routing, auth, database access), MIT-licensed. It does not contain the
database schema, the data-ingestion pipeline, or any proprietary data — every example
below is a real, live response, but this repo itself ships no data. For live usage,
current docs, and to request an API key, see **[www.cryptoally.dev](https://www.cryptoally.dev)**.

## Table of contents

- [Quick start](#quick-start)
- [Authentication](#authentication)
- [Endpoints](#endpoints)
  - [`GET /api/health`](#get-apihealth)
  - [`GET /api/assets`](#get-apiassets)
  - [`GET /api/prices`](#get-apiprices)
  - [`GET /api/catalog`](#get-apicatalog)
- [Markets covered](#markets-covered)
- [Architecture](#architecture)
- [Usage & access](#usage--access)
- [License](#license)
- [About](#about)

## Quick start

```bash
curl -H "x-api-key: YOUR_KEY" \
  "https://www.cryptoally.dev/api/prices?market=crypto&symbol=bitcoin&from=2024-01-01&to=2024-01-03"
```

```json
[
  {
    "date": "2024-01-01",
    "open": "42280.23437500",
    "high": "44175.43750000",
    "low": "42214.97656250",
    "close": "44167.33203125",
    "volume": 18426978443,
    "close_usd": "44167.33203125"
  },
  {
    "date": "2024-01-02",
    "open": "44187.14062500",
    "high": "45899.70703125",
    "low": "44176.94921875",
    "close": "44957.96875000",
    "volume": 39335274536,
    "close_usd": "44957.96875000"
  }
]
```

Don't have a key yet? See [Usage & access](#usage--access).

## Authentication

Every endpoint except `/api/health` requires an `x-api-key` header. A missing or
invalid key gets a `401` before any database query runs:

```bash
curl "https://www.cryptoally.dev/api/assets"
```

```json
{ "error": "missing or invalid API key (x-api-key header)" }
```

```
HTTP/2 401
```

Keys are opaque, random tokens; only their SHA-256 hash is ever stored server-side —
the plaintext is shown once, at issuance, and never persisted. See
[Usage & access](#usage--access) to request one.

## Endpoints

All responses are JSON. Standard status codes throughout: `200` success, `400`
malformed request, `401` missing/invalid key, `404` no such asset/entry, `500`/`503`
server- or database-side failure.

### `GET /api/health`

Liveness + database connectivity check. The only public endpoint — no API key
required.

```bash
curl "https://www.cryptoally.dev/api/health"
```

```json
{ "status": "ok", "database": "ok" }
```

### `GET /api/assets`

Screener-style asset list — filter, search, paginate, and localize the market name.

| Param | Type | Description |
|---|---|---|
| `type` | string | `crypto` \| `stablecoin` \| `stock` \| `etf` \| `index` \| `commodity` |
| `market` | string | A market slug, e.g. `japan`, `crypto`, `us` |
| `search` | string | Matches against symbol or name |
| `limit` | int | Default `50`, max `500` |
| `offset` | int | Default `0` |
| `lang` | string | `en` (default) \| `es` \| `pt` \| `ja` \| `zh` — localizes `market_name` |

```bash
curl -H "x-api-key: YOUR_KEY" \
  "https://www.cryptoally.dev/api/assets?market=crypto&search=bitcoin"
```

```json
[
  {
    "id": 21,
    "symbol": "BCH",
    "name": "Bitcoin Cash",
    "asset_type": "crypto",
    "currency": "USD",
    "market": "crypto",
    "market_name": "Crypto",
    "index_name": null
  },
  {
    "id": 22,
    "symbol": "BTC",
    "name": "Bitcoin",
    "asset_type": "crypto",
    "currency": "USD",
    "market": "crypto",
    "market_name": "Crypto",
    "index_name": null
  }
]
```

### `GET /api/prices`

Daily OHLCV + USD-converted close for one asset. Resolve the asset either with
`asset_id` directly, or with both `market` and `symbol` — a ticker alone isn't unique
across markets (the same symbol can exist on more than one exchange).

| Param | Type | Description |
|---|---|---|
| `asset_id` | int | Direct asset id *(or use `market` + `symbol`)* |
| `market` | string | A market slug — required if using `symbol` |
| `symbol` | string | A ticker (`AAPL`) or crypto id (`bitcoin`) — required if using `market` |
| `from` / `to` | date | `YYYY-MM-DD`, default full history |
| `limit` | int | Default `2000`, max `10000` |

```bash
curl -H "x-api-key: YOUR_KEY" \
  "https://www.cryptoally.dev/api/prices?market=crypto&symbol=bitcoin&from=2024-01-01&to=2024-01-03"
```

```json
[
  {
    "date": "2024-01-01",
    "open": "42280.23437500",
    "high": "44175.43750000",
    "low": "42214.97656250",
    "close": "44167.33203125",
    "volume": 18426978443,
    "close_usd": "44167.33203125"
  },
  {
    "date": "2024-01-02",
    "open": "44187.14062500",
    "high": "45899.70703125",
    "low": "44176.94921875",
    "close": "44957.96875000",
    "volume": 39335274536,
    "close_usd": "44957.96875000"
  },
  {
    "date": "2024-01-03",
    "open": "44961.60156250",
    "high": "45503.24218750",
    "low": "40813.53515625",
    "close": "42848.17578125",
    "volume": 46342323118,
    "close_usd": "42848.17578125"
  }
]
```

A well-formed request for an asset that doesn't exist returns `404`, not an empty
`200` — that distinction is enforced at the resolver, not left to callers to infer
from an empty array.

### `GET /api/catalog`

Founders, founding date, a short history, and official links for one asset. Same
asset-resolution rule as `/api/prices` (`asset_id`, or `market` + `symbol`).

| Param | Type | Description |
|---|---|---|
| `asset_id` | int | Direct asset id *(or use `market` + `symbol`)* |
| `market` / `symbol` | string | Same pairing rule as `/api/prices` |
| `lang` | string | `en` (default) \| `es` \| `pt` \| `ja` \| `zh` |

```bash
curl -H "x-api-key: YOUR_KEY" \
  "https://www.cryptoally.dev/api/catalog?market=crypto&symbol=bitcoin"
```

```json
{
  "asset_id": 22,
  "founders": ["Satoshi Nakamoto (pseudonymous, identity never confirmed)"],
  "founded": "2009-01-03 (genesis block)",
  "history": "The first decentralized cryptocurrency. Introduced in an October 2008 whitepaper as a peer-to-peer electronic cash system; the network launched with the genesis block on 2009-01-03. Uses proof-of-work mining and a fixed 21 million coin supply cap.",
  "official_website": "https://bitcoin.org",
  "logo_url": "https://coin-images.coingecko.com/coins/images/1/large/bitcoin.png",
  "info_url": "https://www.coingecko.com/en/coins/bitcoin",
  "issuer": null,
  "index_provider": null,
  "inception_date": null
}
```

`founded`/`history` are resolved server-side to a single string in the requested
`?lang=`, falling back to English for any asset not yet translated into that language
— translation coverage fills in incrementally, so this is never a `404`, only a
graceful fallback.

## Markets covered

1,864 assets across 36 markets — 27 international stock exchanges, US equities/ETFs,
crypto, and commodities:

| Market | Assets |
|---|---:|
| Japan | 225 |
| South Korea | 200 |
| Australia | 196 |
| United States | 182 |
| United Kingdom | 100 |
| Brazil | 90 |
| Crypto | 62 |
| United Arab Emirates | 61 |
| Saudi Arabia | 60 |
| Canada | 60 |
| India | 50 |
| Taiwan | 50 |
| Colombia | 50 |
| China | 50 |
| New Zealand | 48 |
| Peru | 43 |
| South Africa | 33 |
| Mexico | 32 |
| Chile | 31 |
| Singapore | 30 |
| Egypt | 27 |
| Poland | 20 |
| Austria | 20 |
| Switzerland | 20 |
| Hungary | 20 |
| Argentina | 19 |
| Germany | 17 |
| Portugal | 17 |
| France | 15 |
| Czech Republic | 14 |
| Netherlands | 6 |
| Italy | 5 |
| Commodities | 4 |
| Spain | 4 |
| Belgium | 2 |
| Finland | 1 |
| **Total** | **1,864** |

Query any of these via `/api/assets?market=<slug>` — slugs are lowercase, e.g. `japan`,
`south_korea`, `uae`, `crypto`.

## Architecture

- **Clean architecture, dependency-inversion layers**:
  - `domain/` — entities, newtypes (`AssetId`, `MarketSlug`, `Lang`, `ApiKeyHash`, …),
    and repository traits ("ports"). No `sqlx`, no `axum` — this layer knows nothing
    about HTTP or Postgres.
  - `application/` — thin services orchestrating the ports.
  - `infrastructure/` — the real Postgres-backed repository implementations.
  - `http/` — the axum `Router`, handlers, auth middleware, error mapping.
- **One axum `Router`, one Vercel function.** Every route is matched for real inside
  axum (not per-file bins) — this is what makes auth a single middleware `Layer`
  instead of copy-pasted per handler, and what makes the endpoints properly testable.
- **SQL injection is structurally ruled out, not just avoided.** Every query goes
  through `sqlx` with parameterized, compile-time-checked binds — no query is ever
  built by string interpolation.
- **Newtype pattern.** `AssetId(i64)`, `MarketSlug(String)`, `ApiKeyPlaintext` vs.
  `ApiKeyHash` — wrapping primitives so the type system, not convention, prevents
  mixing up e.g. a plaintext key and a stored hash.
- **Least-privilege database access.** The API's runtime credential is read-only and
  distinct from any credential used by data ingestion, which runs entirely
  out-of-band from this API. There is no write endpoint, authenticated or otherwise.
- **API-key auth**, SHA-256 hash lookup against a hashed-credential store — never
  plaintext at rest, never a JWT (no signed/stateless token here — a request is
  checked against the database on every call, which is what makes a key instantly
  revocable).
- **Tested against fakes, not a live database.** `domain::ports` traits mean the HTTP
  layer — routing, auth, status codes, JSON shape — is exercised end-to-end in tests
  through in-memory fake repositories, with zero database dependency in the test
  process.

## Usage & access

This API and the data it serves are **free to use with attribution**. An API key is
required for every endpoint except `/api/health`, and keys are currently issued on
request rather than self-service.

If you display, publish, or build on data from this API, please credit
**[Artificialss](https://artificialss.ai)**:

```
Data provided by Artificialss (https://artificialss.ai)
```

Prefer a badge? Use this markdown:

```
[![Powered by Artificialss](https://img.shields.io/badge/powered%20by-Artificialss-blueviolet.svg)](https://artificialss.ai)
```

[![Powered by Artificialss](https://img.shields.io/badge/powered%20by-Artificialss-blueviolet.svg)](https://artificialss.ai)

To request a key, visit **[cryptoally.app](https://cryptoally.app)**. Full, current
usage terms live at **[www.cryptoally.dev/#usage](https://www.cryptoally.dev/#usage)**
— that page is the source of truth, not this repository.

## What's intentionally not here

This repo does not include: database schema or migrations, the data-ingestion
pipeline, any raw or processed dataset, deployment credentials/environment
configuration, or the API's homepage/UI. Those live in a separate, private repository.

## License

The source code in this repository is released under the [MIT License](LICENSE). This
covers the code only — it does not grant any rights to the CryptoAlly data or API
service itself; see [Usage & access](#usage--access) above for those terms.

## About

CryptoAlly is built by **[Artificialss](https://artificialss.ai)**. This repository is
the public engineering reference for the API; the product itself — screeners, charts,
and the rest of the CryptoAlly experience — lives at
**[cryptoally.app](https://cryptoally.app)**, with the live API and its full docs at
**[www.cryptoally.dev](https://www.cryptoally.dev)**.

Questions, feedback, or an API key request? Reach out via
**[artificialss.ai](https://artificialss.ai)**.

---

<p align="center">Built by <a href="https://artificialss.ai">Artificialss</a></p>
