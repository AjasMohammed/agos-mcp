# Step 02 — Scaffold

## Inputs
- `.sitecraft/agent/context/tech-stack.md`
- `.sitecraft/agent/context/architecture.md`

## Outputs
- Folder skeleton (`app/`, `components/`, `lib/`, `public/`)
- `package.json`, `tsconfig.json`, `next.config.ts`, `tailwind.config.ts`, `postcss.config.mjs`, `app/globals.css`
- `lib/sitecraft.ts` — typed loader for the three JSON files
- BUILD-STATUS.md § "Scaffold" rows checked

## Procedure
1. Invoke skill `site-scaffolder`.
2. After all files are written, run `npm install`. If it fails, mark `[!]`, stop.
3. Run `npx tsc --noEmit`. Must pass. Failures → fix typings before moving on.

## Next
→ `03-branding.md`
