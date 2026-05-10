# Dashboard field reference — every field, what it is, where it comes from

> **Purpose.** A complete map of every field captured in the SiteCraft dashboard, so the agent
> (and the people filling the dashboard) know exactly what each value is for, who supplies it,
> and which part of the built site it powers.
>
> Source of truth: [`prisma/schema.prisma`](../../prisma/schema.prisma). When schema changes,
> update this doc in the same PR.

## Inputs
- [`data-contracts.md`](./data-contracts.md) — JSON shape after serialization
- `prisma/schema.prisma` — DB models

## Consumed by
- All skills (`plugin/skills/*`)
- The dashboard form components in `app/(dashboard)/projects/[projectId]/`
- The customer onboarding intake

## Source legend

| Tag | Meaning |
|---|---|
| **C** | From the **client** (intake call, questionnaire, brand book) |
| **R** | From **research** the SiteCraft team does (competitor scan, keyword tools) |
| **A** | From **automation / API** (GSC, PSI, Ahrefs, Google Places, AI generators) |
| **S** | **System / internal** (IDs, timestamps, sync state) |

## Use legend (where the field shows up in the built site)

`design` · `marketing` · `website-copy` · `seo` · `schema` · `local-seo` · `analytics` · `ops`

## Pushed to client repo?

Each section header is annotated:

- 📤 **Pushed** — serialized into `.sitecraft/*.json` and shipped to the client repo. Agents can read it.
- 🔒 **Internal** — lives only in the dashboard. Used for analytics, RBAC, ops. Never lands in the client repo.

---

## 0. `Organization` / `User` / `Membership` — tenancy 🔒

These power Clerk-based multi-tenancy and RBAC. **Never serialized.** Agents inside client
repos only see the resulting `organizationId` echoed in `manifest.json`.

### `Organization`
| Field | Source | Use | What & why |
|---|---|---|---|
| `id` | S | ops | Internal CUID — every Project links to one. |
| `clerkOrgId` | A (Clerk) | ops | Clerk organization ID; primary key for SSO. |
| `name` | C | ops, dashboard | Org display name in the dashboard. |
| `slug` | S | ops | URL-safe handle. |
| `createdAt` | S | ops | Audit. |

### `User`
| Field | Source | Use | What & why |
|---|---|---|---|
| `id` | S | ops | Internal CUID. |
| `clerkUserId` | A (Clerk) | ops | Clerk user ID. |
| `email` | A (Clerk) | ops | Login email. |
| `name`, `avatarUrl` | A (Clerk) | dashboard | Header avatar / mention rendering. |
| `createdAt` | S | ops | Audit. |

### `Membership`
| Field | Source | Use | What & why |
|---|---|---|---|
| `userId`, `organizationId` | S | ops | Composite scope. |
| `role` | C | ops | Enum — see Appendix A. Drives every `requireRole()` check. |
| `projectIds[]` | C | ops | Empty = access to all projects in the org. Populated = scoped client viewer/editor. |

---

## 1. `Project` — top-level container 📤 (subset)

| Field | Source | Use | What & why |
|---|---|---|---|
| `id` | S | ops | Internal CUID. Used as the foreign key from every other table. |
| `organizationId` | S | ops | Multi-tenant scope. Every query is filtered by it (CLAUDE.md rule #3). |
| `name` | C | website-copy, schema | Human label of the project (often the brand name). |
| `slug` | C/S | ops | URL-safe key, used in dashboard routes and as repo folder name. |
| `productionUrl` | C | seo, schema | Canonical live URL — feeds `seo.json#siteUrl`, OG urls, sitemap, JSON-LD. |
| `previewUrl` | A | ops | Latest Vercel preview URL — shown in the dashboard, not pushed. |
| `githubInstallationId` | S | ops | GitHub App install used by `sitecraft-bot` to commit. |
| `githubRepoFullName` | C/S | ops | `owner/repo` of the client website repo. |
| `githubDefaultBranch` | C | ops | Branch the bot commits to (default `main`). |
| `syncMode` | C | ops | `DIRECT` commits or `PULL_REQUEST` for review-first workflows. |
| `vercelProjectId` | C/A | ops | Lets the dashboard pull deploy status. |
| `defaultCurrency` | C | website-copy, schema | Used by service/product price formatting. |
| `syncEnabled` | C | ops | Kill switch on the sync engine. |
| `lastSyncAt` / `lastSyncCommitSha` / `lastSyncStatus` | S | ops | Sync state shown in the UI and in `manifest.json`. |
| `createdAt` / `updatedAt` | S | ops | Audit columns. |

---

## 2. `Customer` — who the client is, where they are, what they sell 📤

### 2.1 Core identity (scalar — backwards-compatible with `customer.json`)

| Field | Source | Use | What & why |
|---|---|---|---|
| `legalName` | C | schema, website-copy | The legal company name. Powers Organization JSON-LD `legalName` and the footer. |
| `displayName` | C | website-copy | Friendlier name used in the H1 / hero / nav. |
| `tagline` | C | website-copy, marketing | One-line positioning. Shown under the logo and in OG cards. |
| `description` | C | website-copy, seo, schema | 1–2 sentence company description. Used for meta description fallback and Organization `description`. |
| `founded` | C | schema, website-copy | Founding date. Powers Organization `foundingDate` and "Since YYYY" trust copy. |

### 2.2 Location (scalar — used for LocalBusiness)

| Field | Source | Use | What & why |
|---|---|---|---|
| `addressLine1`, `addressLine2`, `city`, `state`, `postalCode`, `country` | C | local-seo, schema, website-copy | NAP block. Powers LocalBusiness `address`, footer, Contact page. NAP must match Google Business Profile letter-for-letter. |
| `latitude`, `longitude` | C/A | local-seo, schema | Used for `geo` in LocalBusiness JSON-LD and embedded map links. Auto-fillable from `googlePlaceId`. |
| `googlePlaceId` | C/A | local-seo | Stable Google Place ID. Drives "Get directions" links and review fetch. |
| `serviceAreas` | C | local-seo, schema | Cities/regions served. Powers LocalBusiness `areaServed` and service-area landing pages. |

### 2.3 Hours & socials (existing JSON columns)

| Field | Source | Use | What & why |
|---|---|---|---|
| `hours` | C | local-seo, schema | Array of `{day, open, close, alwaysOpen, closed}`. Powers `openingHoursSpecification` and the contact/footer hours block. |
| `socials` | C | website-copy, schema, marketing | Object with `website, instagram, facebook, twitter, linkedin, youtube, whatsapp, tiktok, tripadvisor, googleMaps, yelp, threads, pinterest`. Powers footer icons and Organization `sameAs`. |

### 2.4 Extended identity (`identity` JSON)

| Sub-field | Source | Use | What & why |
|---|---|---|---|
| `elevatorPitch` | C | website-copy | 30-second pitch — used in Hero subhead and About intro. |
| `mission` | C | website-copy, marketing | Mission statement on About page. |
| `vision` | C | website-copy | Long-term vision; About page. |
| `values[]` | C | website-copy, design | Drives "Our values" section and tone-of-voice for AI copy. |
| `stage` | C | marketing | `seed`, `growth`, `mature` — informs which trust signals to surface. |
| `languages[]` | C | seo, website-copy | Site languages for `lang` attribute, hreflang, alternateLocales in SEO. |
| `dba` | C | schema | "Doing business as" — used as alternate name. |

### 2.5 Audience & positioning (`audience` JSON)

| Sub-field | Source | Use | What & why |
|---|---|---|---|
| `personas[]` | C/R | marketing, website-copy | Target buyer personas. Each section can be tailored per persona. |
| `usps[]` | C | marketing, website-copy | Unique selling points — feature grid, "Why choose us". |
| `differentiators[]` | C/R | marketing | Comparative claims used cautiously in copy. |
| `pricingTier` | C | website-copy, schema | `value`, `mid`, `premium`, `luxury` — drives `priceRange` field. |

### 2.6 Services / products catalog (`services` JSON)

Each entry: `{name, slug, desc, price, category, featured, bookingUrl}`

| Sub-field | Source | Use | What & why |
|---|---|---|---|
| `name`, `desc` | C | website-copy, seo | Service card content + meta data. |
| `slug` | C/S | ops | Internal route or anchor. |
| `price` | C | website-copy, schema | Shown in cards; powers Service `offers.price`. |
| `category` | C | website-copy | Groups in service grid. |
| `featured` | C | website-copy | Surfaces on home page hero/cta sections. |
| `bookingUrl` | C | website-copy, marketing | Links the CTA on each service card. |

### 2.7 Trust signals (`trust` JSON)

| Sub-field | Source | Use | What & why |
|---|---|---|---|
| `certifications[]` | C | website-copy, marketing | Logo strip + About page. |
| `awards[]` | C | website-copy, marketing | "Awards" section. |
| `press[]` | C | website-copy, marketing | "As seen in" logo bar. |
| `displayMetrics{customers, projects, years}` | C | website-copy, marketing | Counters in hero/about. |

---

## 3. `Branding` — how the site looks and sounds 📤

### 3.1 Core (scalar — backwards-compatible with `branding.json`)

| Field | Source | Use | What & why |
|---|---|---|---|
| `name` | C | design, website-copy | Brand display name. |
| `logoPrimary` | C | design | Main logo file (URL/asset key). Header. |
| `logoMark` | C | design | Icon-only mark. Favicons, mobile nav, OG fallback. |
| `logoWordmark` | C | design | Text-only logo. Email, print. |
| `favicon` | C | design | Browser tab icon. Generated to `/favicon.ico` + apple-touch. |

### 3.2 Color / type / voice / imagery (existing JSON)

| Field | Source | Use | What & why |
|---|---|---|---|
| `colors` | C | design | `{primary, secondary, accent, neutral, background, foreground}` — Tailwind tokens + CSS variables. |
| `typography` | C | design | `{headingFont, bodyFont, monoFont, baseSize, scale}` — `next/font` wiring. |
| `voice` | C | website-copy, marketing | `{tone[], readingGrade, doNotSay[], alwaysSay[]}` — guides AI copy. |
| `imagery` | C | design, marketing | `{style, colorTreatment, subjectMatter, notesForAI}` — prompts for image gen + photographer brief. |

### 3.3 Brand essence (`essence` JSON)

| Sub-field | Source | Use | What & why |
|---|---|---|---|
| `brandPromise` | C | website-copy, marketing | One-line promise — Hero subhead candidate. |
| `archetype` | C | marketing, website-copy | Jungian archetype (`hero`, `caregiver`, …) — colors voice/imagery. |
| `personalitySliders{formal, classic, serious, premium}` | C | marketing | 0–100 sliders. AI tone uses these directly. |
| `story` | C | website-copy | Long-form origin story — About page. |

### 3.4 Messaging framework (`messaging` JSON)

| Sub-field | Source | Use | What & why |
|---|---|---|---|
| `valueProp` | C | website-copy, marketing | Primary value proposition — Hero. |
| `pillars[]` | C | website-copy | 3–5 messaging pillars — feature grid headings. |
| `taglines{short, medium, long}` | C | website-copy, marketing | Channel-fit tagline variants. |
| `boilerplate{50, 100, 200}` | C | website-copy, marketing | Word-count variants for press, footer, OG. |
| `ctas[]` | C | website-copy, marketing | Approved CTA copy ("Book a call", "Get started"). |

### 3.5 Guardrails (`guardrails` JSON)

| Sub-field | Source | Use | What & why |
|---|---|---|---|
| `visual[{do, dont, imageUrl}]` | C | design | Visual do/don't with example images. |
| `verbal[{do, dont}]` | C | website-copy, marketing | Word-level guardrails. |
| `cobrandingRules` | C | design, marketing | Rules for partner logos / co-marketing. |

### 3.6 Asset manifest (`assets` JSON)

Each entry: `{key, url, kind, version, width, height, size, uploadedAt}`. Used by the agent to look up assets by key (e.g. `key: "hero-bg"`).

---

## 4. `Campaign` — time-boxed marketing 🔒 (dashboard-only today; promote to push when client sites need promo banners)

| Field | Source | Use | What & why |
|---|---|---|---|
| `name`, `slug`, `type`, `status`, `goal` | C | marketing | Campaign metadata. `type` drives template choice; `goal` drives KPIs. |
| `startsAt`, `endsAt` | C | marketing | Schedule. Drives countdown timers and auto-end logic. |
| `offer{headline, subhead, code, discountPct, terms, expiry}` | C | marketing, website-copy | Offer banner / promo card / coupon code. |
| `briefMd` | C/R | marketing | Markdown brief — also fed to AI image gen and copywriters. |
| `audiences[]` | C | marketing | Persona names — chooses which personas see this campaign. |
| `channels[]` | C | marketing | `instagram, google-ads, email, sms, blog, …`. |
| `budget`, `currency` | C | ops, marketing | Reporting only. |
| `kpis[{name, target, current}]` | C/A | marketing, analytics | Goal tracking; surfaces in dashboard. |
| `creatives[{url, channel, format, status}]` | C/R | marketing | Asset library for the campaign. |
| `externalIds` | C/A | marketing | Meta/Google Ads/Klaviyo IDs for cross-platform reporting. |
| `utmTemplate` | C | marketing, analytics | UTM string template for links. |
| `notes` | C | ops | Internal. |

---

## 5. `SEOConfig` — site-wide SEO defaults 📤

| Field | Source | Use | What & why |
|---|---|---|---|
| `titleTemplate` | C/R | seo | e.g. `%s — Acme Co.`. Must contain `%s`. Used by Next.js metadata `title.template`. |
| `titleDefault` | C/R | seo | Fallback `<title>` for pages without their own. Max 70 chars. |
| `description` | C/R | seo | Default meta description. Max 160 chars. |
| `canonicalHost` | C | seo, schema | Canonical hostname URL (e.g. `https://www.example.com`) — fixes split rankings, drives canonicals. |
| `locale` | C | seo | Primary site language. Default `en`. |
| `alternateLocales[]` | C | seo | Additional locales for hreflang. |
| `openGraph` | C | seo, marketing | `{siteName, type:"website"\|"article", imageUrl, imageWidth, imageHeight, imageAlt}`. Default OG card. |
| `twitter` | C | seo, marketing | `{card:"summary"\|"summary_large_image", handle, imageUrl}`. |
| `verification` | C | seo | `{google, bing}` — site-verification meta tag content. Add only the codes the client gave you. |
| `analytics` | C | analytics | `{gaMeasurementId, gtmContainerId}`. Anything else (Plausible/PostHog) needs a schema extension first. |
| `twitterHandle` | C | schema, marketing | Brand Twitter handle (separate scalar, used in Organization `sameAs`). |
| `facebookPageUrl`, `linkedinPageUrl` | C | schema, marketing | Powers Organization `sameAs`. URL-validated. |
| `sitemapEnabled` | C | seo | Toggles `app/sitemap.ts` generation. |
| `sitemapLastBuilt` | S | ops | When the bot last regenerated the sitemap. |
| `gscConnected`, `gscSiteUrl` | C/A | seo, analytics | Google Search Console linkage. |
| `gscAccessToken`, `gscRefreshToken`, `gscTokenExpiry` | A | ops | Encrypted OAuth tokens. Never serialized to client repos. |
| `gscLastFetchAt` | S | ops | When GSC daily ingest last ran. |

---

## 6. `RobotsConfig` — robots.txt 📤

| Field | Source | Use | What & why |
|---|---|---|---|
| `rules` | C | seo | Array of `{userAgent, allow[], disallow[], crawlDelay?}`. Drives `app/robots.ts`. |
| `sitemaps[]` | C/S | seo | Sitemap URLs to advertise. |
| `host` | C | seo | Optional `Host:` directive. |

---

## 7. `SchemaConfig` & `SchemaMarkup` — JSON-LD 📤

| `SchemaConfig.entries` | C/A | schema | Object keyed by schema name (`organization`, `localBusiness`, `blogPost`, …). Each entry is the JSON-LD payload to emit. |
| `SchemaMarkup.name` | C | schema | Friendly label. |
| `SchemaMarkup.schemaType` | C | schema | `Organization`, `LocalBusiness`, `Article`, `Product`, etc. |
| `SchemaMarkup.applyTo` | C | schema | `all`, a route pattern, or a slug — controls which pages render this schema. |
| `SchemaMarkup.data` | C/A | schema | The JSON-LD object. |
| `SchemaMarkup.rendered` | A | schema | Pre-rendered string (cache). |
| `SchemaMarkup.isActive`, `lastPushed` | C/S | ops | Activation toggle + push tracking. |

---

## 8. `Redirect` — 301/302s 📤

| Field | Source | Use | What & why |
|---|---|---|---|
| `source`, `destination` | C/R | seo | The redirect map. Written to `next.config.ts` redirects array. |
| `permanent` | C | seo | `true` → 308, `false` → 307. |
| `notes` | C | ops | Why it exists (e.g. "old service URL — preserves 2024 backlinks"). |

---

## 9. `Blog` — articles 📤 (per published post; analytics fields stripped)

### 9.1 Content
`slug, title, excerpt, coverImage, coverAlt, mdxBody, status, author, publishedAt, readingMinutes, wordCount` — **C / A** for content; **A** for word count. Used for the post page and listing.

### 9.2 SEO targeting
| Field | Source | Use | What & why |
|---|---|---|---|
| `primaryKeyword` | R | seo, content | The single keyword the post is built to rank for. |
| `secondaryKeywords[]` | R | seo, content | Supporting keywords; drive H2s and internal links. |
| `lsiKeywords[]` | R/A | seo, content | Latent semantic terms — ensures topical depth. |
| `intent` | R | seo, content | `INFORMATIONAL`, `COMMERCIAL`, `TRANSACTIONAL`, `NAVIGATIONAL`. |
| `funnelStage` | R | seo, marketing | `AWARENESS`, `CONSIDERATION`, `DECISION`. |
| `categories[]` | C | website-copy, seo | Drives category routes and breadcrumbs. |

### 9.3 Per-post SEO
`metaTitle`, `metaDescription`, `canonical`, `schemaEntries` — overrides for SEOConfig defaults.

### 9.4 AI provenance
`aiAssisted`, `aiMeta` — `aiMeta` records `{provider, model, promptTokens, completionTokens, costUsd}` for transparency.

### 9.5 Internal/external links
`internalLinks[]`, `externalLinks[]` — agent ensures these resolve before publish.

### 9.6 Performance (read-only, ingested from GSC)
`gscClicks`, `gscImpressions`, `gscCtr`, `gscAvgPosition`, `gscUpdatedAt` — surfaced in the dashboard. Not pushed to the client repo.

---

## 10. `Page` — static marketing pages 📤 (when status=PUBLISHED)

`slug, title, mdxBody, metaTitle, metaDescription, status` — same shape as Blog but for non-blog routes (About, Services, custom landing pages).

---

## 11. `Keyword` & `KeywordPosition` — rank tracking 🔒

| Field | Source | Use | What & why |
|---|---|---|---|
| `term` | C/R | seo | The search query being tracked. |
| `intent` | R | seo, content | Same enum as Blog. |
| `volume`, `difficulty`, `cpc`, `trafficPotential` | A (Ahrefs/SerpApi/DataForSEO) | seo | Pulled by `inngest/functions/ingest-keywords.ts`. |
| `currentPosition`, `previousPosition`, `bestPosition`, `positionChange` | A (rank tracker) | seo | Updated by `inngest/functions/rank-tracker.ts`. |
| `targetUrl` | C/R | seo | Which page the term should rank with. |
| `notes`, `priority`, `tracked`, `status` | C | ops | Triage metadata. |
| `KeywordPosition` rows | A | seo | Time series — historical rank chart. |

Not pushed to client repos (analytical only).

---

## 12. `Competitor` — competitive intel 🔒

| Field | Source | Use | What & why |
|---|---|---|---|
| `name`, `domain` | C/R | seo, marketing | Tracked competitors. |
| `notes` | C/R | marketing | Free-form positioning notes. |
| `domainRating`, `organicKeywords`, `organicTraffic`, `backlinks`, `refDomains` | A (Ahrefs) | seo, marketing | Refreshed weekly. Drives gap analysis. |
| `ahrefsFetchedAt` | S | ops | Freshness check. |

---

## 13. `BusinessProfile` — Google Business Profile mirror 📤 (powers LocalBusiness JSON-LD + on-page NAP/hours/services/reviews)

NAP block (`businessName`, `phone`, `email`, `additionalPhones[]`, full address, `latitude/longitude`) — must match the Customer NAP exactly. **Local-SEO critical.**

### 13.1 Identity & categorization
| Field | Source | Use | What & why |
|---|---|---|---|
| `shortName` | C | local-seo | GBP short name (g.page/<short>). |
| `logoUrl`, `coverPhotoUrl` | C | local-seo, design | GBP photos. |
| `businessType` | C | local-seo, schema | `LocalBusiness` subtype (Restaurant, Plumber, etc.). |
| `description` | C | local-seo | GBP description (≤750 chars). |
| `foundingYear` | C | local-seo, schema | "Established YYYY". |
| `priceRange` | C | local-seo, schema | `$`–`$$$$`. |
| `languages[]`, `paymentMethods[]` | C | local-seo | GBP attributes. |
| `primaryCategory`, `categories[]` | C | local-seo | GBP categories. **Most important local-SEO field.** |

### 13.2 Hours
| Field | Source | Use | What & why |
|---|---|---|---|
| `openingHours` | C | local-seo, schema | Standard weekly hours. |
| `specialHours` | C | local-seo | Holiday overrides. |
| `moreHours` | C | local-seo | Extra hour types (delivery, drive-through). |
| `attributes` | C | local-seo | Accessibility, amenities, payments, identity, serviceOptions, health. |

### 13.3 Action links
`appointmentUrl`, `menuUrl`, `orderOnlineUrl`, `reservationUrl`, `bookingUrl`, `quoteUrl` — **C**. Drive GBP CTAs and "Book / Order / Reserve" buttons on the website.

### 13.4 Chat
`chatChannel` (`whatsapp` / `sms`), `chatTarget` — **C**. Powers click-to-chat. (Google native chat ended 2024-07-31.)

### 13.5 External links
`googleMapsUrl`, `googlePlaceId`, `yelpUrl`, `facebookUrl` — **C/A**. Sameas + map embeds + review fetch.

### 13.6 Service area
`serviceArea[]`, `serviceRadius` — **C**. For service-area businesses without a storefront.

### 13.7 Compliance
`verificationStatus`, `verificationMethod`, `suspensionRiskScore`, `suspensionRiskFlags`, `evidencePackUrl`, `region` — **A/S**. Local-SEO ops only; never serialized.

### 13.8 Children of BusinessProfile

| Model | Source | Use | What & why |
|---|---|---|---|
| `BusinessService` | C | local-seo, website-copy, schema | Services with price/duration/photo. Powers service grid + Service JSON-LD. |
| `BusinessProduct` | C | local-seo, website-copy, schema | Products. Powers product grid + Product JSON-LD. |
| `BusinessPhoto` | C | design, local-seo | Categorized photos (`LOGO`, `COVER`, `INTERIOR`, `EXTERIOR`, `TEAM`, `AT_WORK`, `FOOD_DRINK`, `PRODUCT`, `OTHER`). |
| `BusinessPost` | C | marketing, local-seo | GBP posts (`UPDATE`, `OFFER`, `EVENT`, `PRODUCT`) with CTA + offer fields. |
| `BusinessFAQ` | C/R | website-copy, schema | FAQ items. Powers FAQ section + FAQ JSON-LD. |
| `BusinessReview` | A (Google) | local-seo, marketing | Synced reviews + responses. Drives reviews carousel + response-time metrics. |

---

## 14. SEO platform — analytical 🔒 (never serialized)

These tables back the SEO dashboard and are **read-only** for the agent. Never push to the
client repo — they're internal observability.

| Model | Source | Use | What & why |
|---|---|---|---|
| `ProjectSeoConfig` | C/S | ops | Tier (`FREE`/`LOW`/`PRO`), provider keys (encrypted), feature flags. |
| `SeoAuditRun` | A (PSI + custom) | seo, analytics | Audit grade, category scores, Core Web Vitals (`cwv`), issue counts, full findings. |
| `SeoGscDaily` | A (GSC) | seo, analytics | Daily impressions/clicks/CTR/position by page+query. |
| `SeoAiCitation` | A (LLM probes) | seo, marketing | Whether each engine (`CHATGPT`, `PERPLEXITY`, `GEMINI`, `CLAUDE`, `COPILOT`) cites the brand. |
| `SeoApiUsage` | S | ops | Per-provider cost tracking. |
| `SeoRankSnapshot` | A | seo | Per-keyword rank history (richer than `KeywordPosition`). |
| `SeoBacklink` | A (Ahrefs/etc.) | seo | Live/lost/broken backlinks with source URL, anchor text, DR/UR, dofollow flag. |
| `SEOAudit` | A (legacy) | seo | Older audit table — being replaced by `SeoAuditRun`. |

---

## 15. AI usage & sync ops 🔒

| Model | Source | Use | What & why |
|---|---|---|---|
| `AIUsageEvent` | S | ops | Logged per AI call (CLAUDE.md rule #4). Powers cost dashboard + per-feature usage. |
| `GithubInstallation` | S | ops | App install record. |
| `SyncLog` | S | ops | One row per sync attempt — status, commit, files, errors. |
| `Deployment` | A (Vercel) | ops | Deployment status mirror. |

---

## 16. Where each field shows up at a glance

| Site surface | Source models / fields |
|---|---|
| **Hero / Home H1** | `Customer.displayName`, `Branding.essence.brandPromise`, `Branding.messaging.valueProp`, `Customer.identity.elevatorPitch` |
| **Footer NAP** | `Customer.legalName`, address scalars, `Customer.hours`, `Customer.socials` |
| **About page** | `Customer.identity.{mission,vision,values,story}`, `Branding.essence.story`, `Customer.trust.*` |
| **Services / Products** | `Customer.services` (legacy) **or** `BusinessProfile.services/products` (richer) |
| **Contact page** | NAP + `BusinessProfile.{appointment,menu,order,reservation,booking,quote}Url` + `chatChannel/chatTarget` |
| **FAQ** | `BusinessFAQ` |
| **Reviews carousel** | `BusinessReview` |
| **Blog index / posts** | `Blog.*` |
| **Static pages (custom)** | `Page.*` |
| **`<head>` metadata** | `SEOConfig.*` + per-page `metaTitle/metaDescription/canonical` on `Blog` / `Page` |
| **`robots.txt`** | `RobotsConfig` |
| **`sitemap.xml`** | published `Blog`, `Page`, `BusinessService`, `BusinessProduct` URLs |
| **JSON-LD** | `SchemaConfig.entries` + `SchemaMarkup` rows + auto-derived from `Customer` / `BusinessProfile` |
| **Tailwind tokens / CSS variables** | `Branding.colors`, `Branding.typography` |
| **Image generation prompts** | `Branding.imagery`, `Branding.guardrails.visual`, persona descriptors from `Customer.audience.personas` |
| **AI copy prompts** | `Branding.voice`, `Branding.essence.personalitySliders`, `Branding.guardrails.verbal`, `Branding.messaging.*` |
| **Promo banners / offers** | `Campaign.offer` |
| **Verification meta tags** | `SEOConfig.verification` |
| **Analytics scripts** | `SEOConfig.analytics` |
| **`sameAs` in Organization JSON-LD** | `Customer.socials` + `SEOConfig.{twitterHandle, facebookPageUrl, linkedinPageUrl}` |

---

## 17. Field-fill checklist for a new client

Use this when onboarding. Anything starred is a **release blocker**.

**Day 1 — intake call**
- [ ] `Project.name`, `slug`, `productionUrl` *
- [ ] `Customer.{legalName, displayName, tagline, description}` *
- [ ] Full NAP * (must match GBP exactly)
- [ ] `Customer.hours` *
- [ ] `Customer.socials`
- [ ] `Branding.{logoPrimary, favicon}` *
- [ ] `Branding.colors` *
- [ ] `Branding.typography`
- [ ] `Branding.voice.tone`

**Day 2 — brand workshop**
- [ ] `Branding.essence.{brandPromise, archetype, personalitySliders, story}`
- [ ] `Branding.messaging.{valueProp, pillars, taglines, boilerplate, ctas}`
- [ ] `Branding.guardrails.{visual, verbal}`
- [ ] `Customer.identity.{mission, vision, values, elevatorPitch}`
- [ ] `Customer.audience.{personas, usps, differentiators, pricingTier}`

**Day 3 — catalog & trust**
- [ ] `Customer.services` (or `BusinessProfile.services`) *
- [ ] `Customer.trust.{certifications, awards, press, displayMetrics}`
- [ ] `BusinessProfile.{primaryCategory, categories, attributes}` (if local) *
- [ ] `BusinessProfile` action links

**Day 4 — SEO**
- [ ] `SEOConfig.{titleTemplate, titleDefault, description, canonicalHost}` *
- [ ] `SEOConfig.{openGraph.defaultImage}` *
- [ ] `SEOConfig.verification` codes
- [ ] `SEOConfig.analytics` IDs
- [ ] `RobotsConfig.rules` (default is fine for most)
- [ ] `Keyword[]` — initial tracked set (R)
- [ ] `Competitor[]` — top 3–5 (C/R)
- [ ] `Redirect[]` — only if migrating from an old site

**Day 5 — content**
- [ ] `Page` rows for About, Services, Contact at minimum
- [ ] `BusinessFAQ` — 6+ entries for Schema FAQ payoff
- [ ] `BusinessPhoto` — at least LOGO, COVER, plus 5+ category photos

---

## Appendix A — full enum reference

Authoritative copy of every enum in `prisma/schema.prisma`. The agent should treat these
as closed sets — never invent values not listed here.

| Enum | Values | Used by |
|---|---|---|
| `Role` | `OWNER`, `OPERATOR`, `EDITOR`, `CLIENT_VIEWER`, `CLIENT_EDITOR` | `Membership.role` |
| `SyncMode` | `DIRECT`, `PULL_REQUEST` | `Project.syncMode` |
| `SyncStatus` | `PENDING`, `SUCCESS`, `FAILED`, `CONFLICT` | `Project.lastSyncStatus` |
| `SyncLogStatus` | `STARTED`, `SUCCESS`, `FAILED`, `CONFLICT` | `SyncLog.status` |
| `CampaignType` | `LAUNCH`, `SALE`, `SEASONAL`, `ALWAYS_ON`, `RETENTION`, `BRAND` | `Campaign.type` |
| `CampaignStatus` | `DRAFT`, `SCHEDULED`, `LIVE`, `PAUSED`, `ENDED` | `Campaign.status` |
| `CampaignGoal` | `AWARENESS`, `LEADS`, `SALES`, `RETENTION`, `APP_INSTALLS` | `Campaign.goal` |
| `ContentStatus` | `DRAFT`, `IN_REVIEW`, `SCHEDULED`, `PUBLISHED`, `ARCHIVED` | `Blog.status`, `Page.status` |
| `SearchIntent` | `INFORMATIONAL`, `COMMERCIAL`, `TRANSACTIONAL`, `NAVIGATIONAL` | `Blog.intent`, `Keyword.intent` |
| `FunnelStage` | `AWARENESS`, `CONSIDERATION`, `DECISION` | `Blog.funnelStage` |
| `AIKind` | `TEXT`, `IMAGE`, `EMBEDDING`, `AUDIT` | `AIUsageEvent.kind` |
| `PhotoCategory` | `LOGO`, `COVER`, `INTERIOR`, `EXTERIOR`, `TEAM`, `AT_WORK`, `FOOD_DRINK`, `PRODUCT`, `OTHER` | `BusinessPhoto.category` |
| `PostType` | `UPDATE`, `OFFER`, `EVENT`, `PRODUCT` | `BusinessPost.type` |
| `PostStatus` | `DRAFT`, `SCHEDULED`, `PUBLISHED`, `EXPIRED` | `BusinessPost.status` |
| `PostCta` | `NONE`, `BOOK`, `ORDER`, `LEARN_MORE`, `SIGN_UP`, `CALL`, `BUY` | `BusinessPost.cta` |
| `SeoTier` | `FREE`, `LOW`, `PRO` | `ProjectSeoConfig.tier` |
| `AiEngine` | `CHATGPT`, `PERPLEXITY`, `GEMINI`, `CLAUDE`, `COPILOT` | `SeoAiCitation.engine` |
| `BacklinkStatus` | `LIVE`, `LOST`, `BROKEN`, `NEW` | `SeoBacklink.status` |

---

## Appendix B — model coverage matrix

Sanity check that this doc covers every model in the schema. **36 models / 18 enums total** (verified with `grep -c "^model " prisma/schema.prisma`).

| # | Model | Documented in § |
|---|---|---|
| 1 | `Organization` | 0 |
| 2 | `User` | 0 |
| 3 | `Membership` | 0 |
| 4 | `Project` | 1 |
| 5 | `Customer` | 2 |
| 6 | `Branding` | 3 |
| 7 | `Campaign` | 4 |
| 8 | `SEOConfig` | 5 |
| 9 | `RobotsConfig` | 6 |
| 10 | `SchemaConfig` | 7 |
| 11 | `SchemaMarkup` | 7 |
| 12 | `Redirect` | 8 |
| 13 | `Blog` | 9 |
| 14 | `Page` | 10 |
| 15 | `Keyword` | 11 |
| 16 | `KeywordPosition` | 11 |
| 17 | `Competitor` | 12 |
| 18 | `BusinessProfile` | 13 |
| 19 | `BusinessService` | 13.8 |
| 20 | `BusinessProduct` | 13.8 |
| 21 | `BusinessPhoto` | 13.8 |
| 22 | `BusinessPost` | 13.8 |
| 23 | `BusinessFAQ` | 13.8 |
| 24 | `BusinessReview` | 13.8 |
| 25 | `ProjectSeoConfig` | 14 |
| 26 | `SeoAuditRun` | 14 |
| 27 | `SeoGscDaily` | 14 |
| 28 | `SeoAiCitation` | 14 |
| 29 | `SeoApiUsage` | 14 |
| 30 | `SeoRankSnapshot` | 14 |
| 31 | `SeoBacklink` | 14 |
| 32 | `SEOAudit` (legacy) | 14 |
| 33 | `AIUsageEvent` | 15 |
| 34 | `GithubInstallation` | 15 |
| 35 | `SyncLog` | 15 |
| 36 | `Deployment` | 15 |

If a new model is added to `prisma/schema.prisma`, add a row here and document it above
**in the same PR**. CI should grep this file for the new model name and fail if missing.
