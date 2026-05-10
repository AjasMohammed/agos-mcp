# BUILD-STATUS

> Single source of truth for this client repo's build progress.
> Owned by the SiteCraft agent kit. Humans may read it; only the `status-tracker` skill writes it.

**Last updated:** _never_
**Agent kit version:** 0.1.0
**Source manifest:** `.sitecraft/manifest.json`

---

## Phases

### 01 — Bootstrap
- [ ] manifest.json present and valid
- [ ] branding.json / customer.json / seo.json validated
- [ ] BUILD-STATUS.md initialized
- [ ] AGENTS.md initialized

### 02 — Scaffold
- [ ] package.json + tsconfig + next.config + tailwind.config written
- [ ] Folder skeleton created
- [ ] `lib/sitecraft.ts` typed loader written
- [ ] `npm install` succeeded
- [ ] `tsc --noEmit` passes

### 03 — Branding
- [ ] Tailwind tokens populated from branding.json
- [ ] CSS variables in `globals.css`
- [ ] Fonts wired via `next/font`
- [ ] Logo assets in `public/brand/`
- [ ] Dev server renders without console errors

### 04 — Content
<!-- One row per page in customer.json#pages[] — status-tracker adds these dynamically -->
- [ ] (pages will be listed here as they generate)

### 05 — SEO
- [ ] `app/robots.ts` written
- [ ] `app/sitemap.ts` written
- [ ] `components/seo/JsonLd.tsx` written
- [ ] `app/layout.tsx` metadata populated
- [ ] Per-page `generateMetadata` written
- [ ] `/robots.txt` and `/sitemap.xml` return 200 in dev

### 06 — Verify
- [ ] `tsc --noEmit` passes (full project)
- [ ] `npm run build` succeeds
- [ ] All pages return 200 in dev with no console errors
- [ ] Final summary posted

---

## Activity log
<!-- one line per skill action: `- YYYY-MM-DD HH:MM <skill> — <what happened>` -->

## Blockers
<!-- one line per `[!]` row above, with the reason -->

## Deviations
<!-- libraries, versions, or conventions that diverged from tech-stack.md, with reason -->

## Content gaps
<!-- sections that needed copy not present in customer.json -->

## Build warnings
<!-- captured from `npm run build` output -->
