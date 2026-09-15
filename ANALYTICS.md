# Analytics

Vercel Web Analytics (pageviews + custom events) and Speed Insights (Core Web Vitals),
both enabled via `vercel project web-analytics` / `vercel project speed-insights` and
wired into `static/index.html` using the plain-HTML integration (no npm package --
there's no Node build step in this project, so the `@vercel/analytics` package doesn't
apply; see [Vercel's HTML integration docs](https://vercel.com/docs/analytics/quickstart)).

```html
<script>window.va = window.va || function () { (window.vaq = window.vaq || []).push(arguments); };</script>
<script defer src="/_vercel/insights/script.js"></script>

<script>window.si = window.si || function () { (window.siq = window.siq || []).push(arguments); };</script>
<script defer src="/_vercel/speed-insights/script.js"></script>
```

Pageviews and Core Web Vitals are collected automatically once these scripts load --
nothing further to wire up for those. View both in the Vercel dashboard under
**Analytics** / **Speed Insights** for the `cryptoally-api` project.

## Page map (single page, section + interaction tracked)

The site is one page (`/`, i.e. `api/index.rs`) with anchored sections. There's no
route-based navigation to track, so the map below is section + interaction, not
page-to-page.

| Section (`#anchor`) | What's tracked | Event name | Data |
|---|---|---|---|
| Top nav | Click any nav link | `nav_click` | `section`: `usage` \| `api-docs` \| `markets` \| `stocks` \| `stablecoins` \| `fetching-rules` |
| Top nav / API Docs | Click "Status" (either button) | `status_check_opened` | `trigger`: `nav` \| `api-docs-cta` |
| Usage | Click "Request an API key" | `api_key_request_clicked` | `method`: `email` |
| Usage | Click "Visit cryptoally.app" | `cryptoally_app_visited` | `source`: `usage-section` |
| Fetching Rules (Usage card) | Click the `cryptoally.app` link | `cryptoally_app_visited` | `source`: `fetching-rules` |
| API Docs | Click "View source on GitHub" | `github_source_clicked` | -- |
| Footer | Click an icon (CryptoAlly / Artificialss / AI Invest) | `footer_icon_clicked` | `target`: `cryptoally.app` \| `artificialss.ai` \| `ai-invest.market` |
| Markets / Stocks / Stablecoins | Click a sortable column header | `table_sort` | `table`: `markets-table` \| `stocks-table` \| `stablecoins-table`, `column`: header text, `direction`: `asc` \| `desc` |

Implementation: every trackable element carries `data-va-event` (+ any extra `data-*`
attributes to pass through as event data) in the markup; one delegated listener in
`static/index.html` reads those attributes and calls `va('event', { name, data })` on
click. Adding a new tracked element is a markup-only change -- no new JS needed unless
the interaction isn't a plain click (the table-sort handler is the one exception, since
it needs the resolved sort direction, not just a static attribute).

## What's deliberately not tracked

- **Individual endpoint calls** (`/api/assets`, `/api/prices`) -- those are API traffic,
  not page interactions; they're server-side and not part of Web Analytics' model here.
  If request-level metrics are needed later, that's a different system (e.g. logging
  `ingestion_runs`-style rows, or Vercel's function invocation metrics), not this.
- **Scroll depth / section-view (impression) tracking** -- adds meaningful JS complexity
  (IntersectionObserver, dedup logic) for a single page with six short sections; click
  tracking on the nav already gives a reasonable proxy for section interest.
- **The attribution link inside the Usage `<pre>` snippet** -- it's a code sample, not a
  real link (plain text), so there's nothing to instrument.
