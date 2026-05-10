---
name: seo-publisher
description: Generate robots.txt, sitemap.xml, JSON-LD schema, and metadata from .sitecraft/seo.json using Next.js Metadata API. Use when BUILD-STATUS.md shows step 05-seo is unchecked, or when invoked directly via /seo-deploy.
---

## Inputs
- `.sitecraft/seo.json` — meta defaults, robots rules, sitemap routes, schema config
- `.sitecraft/customer.json` — business name, address, geo for LocalBusiness schema
- `.sitecraft/agent/context/tech-stack.md` — confirms Next.js Metadata API conventions

## Outputs
- `app/robots.ts` — exports a `MetadataRoute.Robots` function
- `app/sitemap.ts` — exports a `MetadataRoute.Sitemap` function
- `components/seo/JsonLd.tsx` — renders `<script type="application/ld+json">` for the schemas requested
- `app/layout.tsx` — `metadata` export populated (title template, description, OG, Twitter)
- Per-page `generateMetadata()` where seo.json has page-level overrides
- BUILD-STATUS.md § "SEO" checked

## Procedure
1. Read seo.json. Validate `siteUrl` is set — without it, sitemap and canonical URLs break.
2. `app/robots.ts` — translate `seo.json#robots` rules. Default to `Allow: /` with a
   `Disallow: /api/` if not specified.
3. `app/sitemap.ts` — pull routes from `seo.json#routes[]`, fall back to scanning `app/`
   for `page.tsx` files. Include `lastModified`, `changeFrequency`, `priority`.
4. `JsonLd.tsx` — render Organization + WebSite by default. Add LocalBusiness if
   `customer.json#business.address` is present.
5. Per-page metadata — for each entry in `seo.json#pages`, write `generateMetadata` in
   the matching `app/.../page.tsx`.

## Hard rules
- Never hard-code the production URL — pull from `seo.json#siteUrl`.
- Don't write a static `public/robots.txt` — Next.js's `app/robots.ts` is the contract.
- JSON-LD must validate against schema.org. If a required field is missing in customer.json,
  omit that schema rather than emitting invalid markup.
