# Step 05 — SEO

## Inputs
- `.sitecraft/seo.json`
- `.sitecraft/customer.json`
- All pages from step 04

## Outputs
- `app/robots.ts`, `app/sitemap.ts`, `components/seo/JsonLd.tsx`
- `app/layout.tsx#metadata` populated
- Per-page `generateMetadata()` for entries in `seo.json#pages`
- BUILD-STATUS.md § "SEO" checked

## Procedure
1. Invoke skill `seo-publisher`.
2. After write, hit `http://localhost:<port>/robots.txt` and `/sitemap.xml` in dev — both
   must return 200 and valid output.
3. Validate the rendered JSON-LD on the home page using `JSON.parse` on the script tag
   contents — must parse, must contain at least `@context` and `@type`.

## Next
→ `06-verify.md`
