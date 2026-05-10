---
name: site-scaffolder
description: Scaffold a fresh Next.js 15 App Router project structure following SiteCraft conventions. Use when BUILD-STATUS.md shows step 02-scaffold is unchecked, or when the repo has only .sitecraft/ data files and no app/ directory yet. Triggers on phrases like "scaffold the site", "set up the Next.js project", or "create the file structure".
---

## Inputs
- `.sitecraft/agent/context/tech-stack.md` — locked tech choices
- `.sitecraft/agent/context/architecture.md` — folder layout
- `.sitecraft/customer.json` — site name, route hints

## Outputs
- `package.json` with pinned versions from tech-stack.md
- `app/`, `components/`, `lib/`, `public/` directories
- `tailwind.config.ts`, `postcss.config.mjs`, `tsconfig.json`, `next.config.ts`
- An empty `app/page.tsx` (filled by content-generator skill)
- `BUILD-STATUS.md` § "Scaffold" checked off

## Procedure
1. Read tech-stack.md and architecture.md. Do not deviate from pinned versions.
2. Initialize `package.json`. Run `npm install` only after all config files are written
   (avoids partial installs on failure).
3. Create the folder skeleton. Use `git mv` if migrating from a starter — never `rm -rf`.
4. Write Tailwind config with brand tokens **commented as TODO** — the brand-applier skill
   fills them in step 03.
5. Update BUILD-STATUS.md and hand off to step 03.

## Hard rules
- Never run `npx create-next-app` — it overwrites and pulls unpinned versions.
- Never install packages not listed in tech-stack.md without writing a note in BUILD-STATUS.md
  § "Deviations".
