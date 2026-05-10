---
description: Generate or refresh robots.txt, sitemap.xml, and JSON-LD schema from .sitecraft/seo.json without re-running the full build. Use when only SEO data changed in the dashboard.
allowed-tools: Read, Write, Edit, Glob
---

# /seo-deploy

Targeted SEO refresh. Skips scaffolding and content generation.

## Inputs

- `.sitecraft/seo.json`
- `.sitecraft/customer.json` (for canonical URL, business schema)
- `.sitecraft/agent/workflow/05-seo.md`
- `.sitecraft/agent/plugin/skills/seo-publisher/SKILL.md`

## Procedure

Run **only** workflow step `05-seo.md`. Update `BUILD-STATUS.md` § "SEO" only — do not
touch other sections.

## Outputs

- `app/robots.ts` (Next.js Metadata API)
- `app/sitemap.ts`
- `components/seo/JsonLd.tsx` (Organization, LocalBusiness, WebSite as configured)
- `app/layout.tsx` metadata block refreshed
