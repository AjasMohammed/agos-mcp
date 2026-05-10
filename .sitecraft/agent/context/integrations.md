# Integrations

Outside services the client repo touches. The agent rarely needs to wire these from
scratch — the SiteCraft sync prepares most of it.

## Inputs
- (leaf)

## Consumed by
- Skills that surface external services (e.g. content-generator wiring contact form)

## Services

| Service | Used for | How the agent interacts |
|---|---|---|
| **GitHub** | Hosts the client repo. SiteCraft commits via `sitecraft-bot` GitHub App. | Read-only — the repo is already cloned. |
| **Vercel** | Hosting + preview deploys. | Connect repo once via Vercel UI; `main` auto-deploys. The agent does not call the Vercel API. |
| **Supabase** *(when needed)* | Postgres + auth for client-side features. | Only if `customer.json#pages[]` includes auth-gated content. Use `@supabase/ssr` per Supabase skill. |
| **Inngest** *(SiteCraft side)* | Drives the sync job that put files here. | Not a runtime dep of the client site. |
| **Vercel Analytics & Speed Insights** | Free analytics. | Add `<Analytics />` + `<SpeedInsights />` in `app/layout.tsx`. |
| **Resend / Postmark** *(optional)* | Contact form delivery. | Only if `customer.json#business.email` is present and `app/api/contact/route.ts` is generated. Use env var `EMAIL_PROVIDER_API_KEY`. |

## Env vars expected at build time

```
NEXT_PUBLIC_SITE_URL          # = seo.json#siteUrl, mirrored for client code
EMAIL_PROVIDER_API_KEY        # only if contact form exists
SUPABASE_URL                  # only if Supabase is used
SUPABASE_ANON_KEY
```

The agent writes a `.env.example` listing whichever of these the build actually requires.
