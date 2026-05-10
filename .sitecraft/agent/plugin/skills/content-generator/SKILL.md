---
name: content-generator
description: Generate page content (Home, About, Services, Contact, plus any pages listed in customer.json) from .sitecraft/customer.json. Use when BUILD-STATUS.md shows step 04-content is unchecked. Renders sections with shadcn/ui primitives wired to brand tokens.
---

## Inputs
- `.sitecraft/customer.json` — copy, services, products, testimonials, contact
- `.sitecraft/branding.json` — for tone-of-voice fields (`brandVoice`, `tagline`)
- `.sitecraft/agent/context/architecture.md` — page route conventions

## Outputs
- `app/page.tsx` (Home)
- `app/about/page.tsx`, `app/services/page.tsx`, `app/contact/page.tsx` as configured
- Any custom routes listed under `customer.json#pages[]`
- `components/sections/` — reusable Hero, FeatureGrid, Testimonials, ContactForm
- BUILD-STATUS.md § "Content" — one row per page, each checked when the page renders without runtime errors

## Procedure
1. Read customer.json. For each entry in `pages[]` (or the default set if absent), pick a
   matching section composition.
2. Use only copy that exists in customer.json. If a section needs copy that isn't there,
   render the section with a TODO placeholder and log it under "Content gaps" in BUILD-STATUS.
3. Wire forms to `app/api/contact/route.ts` (create the route if absent) — do not invent
   external endpoints.
4. After each page, run `tsc --noEmit` on that file. Failures → fix before moving on.

## Hard rules
- Never fabricate testimonials, prices, hours, or contact details.
- Never inline brand colors — use Tailwind tokens from step 03.
