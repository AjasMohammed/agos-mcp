# Step 06 — Verify

## Inputs
- The fully built site
- `BUILD-STATUS.md`

## Outputs
- Final BUILD-STATUS.md with all rows checked or annotated
- A short summary message to the user

## Procedure
1. Run `npx tsc --noEmit` on the whole project. Must pass.
2. Run `npm run build`. Must succeed. Capture warnings in BUILD-STATUS § "Build warnings".
3. Start `npm run dev` in background, hit each generated page once, confirm 200 + no
   console errors. Stop the dev server.
4. Confirm `/robots.txt`, `/sitemap.xml` still serve.
5. Update BUILD-STATUS § "Verify" — every row.
6. Post a final summary to chat:
   - Pages generated: N
   - Brand tokens applied
   - SEO files: robots.ts, sitemap.ts, JsonLd.tsx
   - Open issues / content gaps: count + section to read in BUILD-STATUS
   - Dev server URL

## Done
The agent stops here. Subsequent dashboard changes trigger a new sync and the user re-runs
`/build-site` (which resumes from BUILD-STATUS.md) or `/seo-deploy` for SEO-only refreshes.
