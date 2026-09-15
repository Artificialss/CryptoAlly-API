# SEO Strategy

This is a single-page API product site, not a content site — the strategy is scoped to
what actually applies: making that one page maximally findable and correctly described,
plus positioning the underlying data for discovery through channels other than classic
web search (Google Dataset Search, API directories, AI answer engines).

## What's implemented (`static/index.html`, `api/robots.rs`, `api/sitemap.rs`)

- **Title/description** target the actual query intent: people searching for
  `crypto api`, `stock market data api`, `historical price data api`, `free financial
  data api`. Title leads with the product name (brand recognition) then the value prop
  (`Free Crypto, Stock & Market Data API`) — matches how competitors (CoinGecko API,
  Alpha Vantage, Polygon.io) title their own docs pages.
- **Canonical tag** — `https://www.cryptoally.dev/`, since content is also reachable via
  the `.vercel.app` alias; prevents duplicate-content ambiguity.
- **Open Graph + Twitter Card** — so links shared in Slack/Discord/X render a proper
  card instead of a bare URL. This matters more than usual here: developer discovery of
  APIs happens heavily through link shares in chat, not search.
- **JSON-LD structured data, `schema.org/Dataset`** — this is the single highest-leverage
  move available. This product *is* a dataset with an access API, which is exactly
  what [Google Dataset Search](https://datasetsearch.research.google.com) indexes.
  Getting listed there is a much less contested channel than ranking for generic
  "crypto api" queries against CoinGecko/CMC. `creator` links to Artificialss for
  entity association.
- **`robots.txt` + `sitemap.xml`** — served as real endpoints (`/api/robots.rs`,
  `/api/sitemap.rs`), not static files, consistent with how every other route here
  works. One URL in the sitemap: this is a single page, and the `#section` anchors
  aren't independently crawlable documents.

## Keyword targets

Primary (title/description/JSON-LD `keywords`): `crypto api`, `stock market data api`,
`historical price data api`, `financial data api`, `market data api`.
Secondary (naturally present in body copy already — Markets/Stocks/Stablecoins sections,
Fetching Rules): `stablecoin data`, `OHLCV data`, `international stock market data`,
`free financial api`.

Deliberately not targeting: coin-specific queries ("bitcoin price api") — that's
CoinGecko/CMC's turf and this isn't a coin-tracking product; broad "crypto news/prices"
consumer queries — wrong audience (this is a developer-facing API, not a consumer app).

## Content strategy

The page's copy already does double duty as documentation and as search-facing text —
that's intentional and should stay that way rather than splitting into a separate
"marketing" page and "docs" page. Two concrete levers if this needs to grow later:

1. **Per-market or per-asset-class pages** (`/markets/japan`, `/crypto`, `/stocks`) would
   let each rank for its own long-tail query ("japan stock market api",
   "stablecoin price history api") instead of competing with itself on one URL. Not
   built now — would need real content per page, not just a filtered table, to avoid
   thin-content penalties.
2. **A changelog/blog is not recommended** for this product. API changelogs rarely drive
   organic traffic and add maintenance surface; the ingestion audit history already
   lives in `Seed/README.md` for anyone who needs it.

## Technical checklist (status)

- [x] Unique, descriptive `<title>` and `<meta description>`
- [x] Canonical URL
- [x] Open Graph + Twitter Card
- [x] Structured data (`Dataset`)
- [x] `robots.txt`, `sitemap.xml`
- [x] Single `<h1>` per page (the hero title), proper `<h2>` per section
- [x] Mobile-responsive (already required by the artifact/page conventions this was built under)
- [ ] Favicon — not set; low priority, cosmetic only
- [ ] Backlinks from `cryptoally.app` (the marketing site) and the GitHub repo (once
      public) — both should link to `www.cryptoally.dev`; this is worth more than any
      on-page change at this stage, since the site has zero external links pointing at
      it today
- [ ] Submit to Google Search Console + Google Dataset Search once the public repo and
      any backlinks exist, so there's something for a crawler to actually discover

## What actually moves the needle here

For a brand-new, zero-backlink API product page, on-page SEO is necessary but not
sufficient — search engines need a reason to find and trust the page at all. The two
highest-impact next actions, in order: **(1)** link to `www.cryptoally.dev` from
`cryptoally.app` and the GitHub repo once it's public, **(2)** submit the URL to Google
Search Console and Dataset Search directly rather than waiting for organic crawl
discovery.
