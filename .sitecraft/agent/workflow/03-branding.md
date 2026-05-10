# Step 03 — Branding

## Inputs
- `.sitecraft/branding.json`
- `tailwind.config.ts`, `app/globals.css` (from step 02)

## Outputs
- Tailwind tokens populated, CSS variables set, fonts wired, logo assets in `public/brand/`
- BUILD-STATUS.md § "Branding" checked

## Procedure
1. Invoke skill `brand-applier`.
2. Run `npm run dev` briefly (background) and confirm `app/page.tsx` (still empty) renders
   without console errors. Stop the dev server.

## Next
→ `04-content.md`
