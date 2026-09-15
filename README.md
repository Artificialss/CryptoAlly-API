<div align="center">
  <img src=".github/logo.svg" alt="CryptoAlly logo" width="120" />

  # CryptoAlly API
</div>

A Rust API serving historical price and catalog data for 1,864 assets — crypto,
stablecoins, US stocks/ETFs, commodities, and 27 international stock markets — deployed
on Vercel's official Rust runtime.

This repository is a **public reference** for the API's engineering: its architecture,
stack, and general request/response shape. It is not the project's primary repository,
and it does not contain database internals, ingestion logic, or any proprietary or
sensitive data — every response shown here is illustrative, not a live data dump. For
live usage, docs, and to request access, see **[www.cryptoally.dev](https://www.cryptoally.dev)**.

## Stack

- **Language:** Rust, 2021 edition
- **Runtime:** [Vercel's official Rust runtime](https://vercel.com/docs/functions/runtimes/rust) (`vercel_runtime`, public beta) — each function is an independent serverless binary, cold-started on demand and reused across requests on warm Fluid Compute instances
- **Database client:** `sqlx` (compile-time-checked, parameterized queries against Postgres — no string-built SQL)
- **Async runtime:** `tokio`

## Architecture

- **One binary per endpoint.** Vercel's Rust convention: each file under `api/`
  compiles to its own `[[bin]]` target, deployed as an independent serverless function.
  There's no monolithic router — routing is Vercel's filesystem-based convention.
- **Shared library crate** for everything the endpoints have in common: the database
  connection pool, response/error helpers, and shared row types. Each binary depends on
  it as an ordinary Rust library.
- **Connection pooling, not per-request connections.** The pool is initialized once per
  warm function instance (via a lazily-initialized static) and reused across
  invocations, rather than opening a fresh Postgres connection on every request.
- **Least-privilege database access.** The API's runtime credential is scoped to
  read-only access — it cannot write, and it's a distinct credential from anything used
  by data ingestion, which runs entirely out-of-band from this API.
- **API-key authentication**, checked per-request against a hashed-credential store
  (never plaintext at rest); a request with a missing or invalid key never reaches the
  database query it would have triggered.
- **No public write surface.** This API is read-only by design — there is no endpoint,
  authenticated or otherwise, that mutates data.

## Request shape

Endpoints are conventional REST-style `GET` requests returning JSON, authenticated via
an `x-api-key` header. A minimal example (illustrative only — see
[www.cryptoally.dev](https://www.cryptoally.dev) for the real, current endpoint
reference):

```
GET /api/<resource>?<filters>
x-api-key: <your key>

200 OK
Content-Type: application/json

[{ "...": "..." }]
```

Standard HTTP status codes throughout: `200` on success, `400` for a malformed request,
`401` for a missing/invalid key, `500`/`503` for server- or database-side failures.

## Usage & access

This API and the data it serves are **free to use with attribution**. An API key is
required for every endpoint except the public health check, and keys are currently
issued on request rather than self-service. Full usage terms, attribution requirements,
and how to request a key are documented at
**[www.cryptoally.dev](https://www.cryptoally.dev/#usage)** — that page is the source of
truth, not this repository.

## What's intentionally not here

This repo does not include: database schema or migrations, the data ingestion
pipeline, any raw or processed dataset, deployment credentials/environment
configuration, or the API's homepage/UI. Those live in a separate, private repository.

## License

The source code in this repository is released under the [MIT License](LICENSE). This
covers the code only — it does not grant any rights to the CryptoAlly data or API
service itself; see [Usage & access](#usage--access) above for those terms.

## About

Built by [Artificialss](https://artificialss.ai).
