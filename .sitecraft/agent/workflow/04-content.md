# Step 04 — Content

## Inputs
- `.sitecraft/customer.json`
- `.sitecraft/branding.json` (for `tagline`, `brandVoice`)
- Brand tokens from step 03

## Outputs
- One `app/<slug>/page.tsx` per entry in `customer.json#pages[]`
- `components/sections/` library
- `app/api/contact/route.ts` if a contact page is configured
- BUILD-STATUS.md § "Content" — one row per page, each checked

## Procedure
1. Invoke skill `content-generator`.
2. For each page generated, run `tsc --noEmit` on the file. Failures → fix.
3. Log any "content gaps" (sections needing copy not in customer.json) under
   BUILD-STATUS § "Content gaps".

## Next
→ `05-seo.md`
