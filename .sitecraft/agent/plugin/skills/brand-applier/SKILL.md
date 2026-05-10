---
name: brand-applier
description: Apply branding tokens (colors, typography, logo, spacing) from .sitecraft/branding.json into Tailwind config, CSS variables, and the root layout. Use when BUILD-STATUS.md shows step 03-branding is unchecked, or when branding.json has been updated since the last sync.
---

## Inputs
- `.sitecraft/branding.json` — full brand spec
- `.sitecraft/agent/context/tech-stack.md` — confirms Tailwind v4 token format

## Outputs
- `tailwind.config.ts` — `theme.extend.colors`, `fontFamily`, `borderRadius` populated
- `app/globals.css` — CSS variables for `--brand-primary`, `--brand-accent`, etc.
- `app/layout.tsx` — `<html>` font class wired up
- `public/brand/` — logo files written from base64 / URLs in branding.json
- BUILD-STATUS.md § "Branding" checked + token diff logged

## Procedure
1. Read branding.json. Validate required fields: `colors.primary`, `colors.accent`, `typography.heading`, `typography.body`, `logo`. Missing → mark `[!]` in status, stop.
2. Map JSON to Tailwind tokens. Use HSL CSS variables so dark mode flips cleanly.
3. Pull the heading + body fonts via `next/font/google` or `next/font/local` based on
   `typography.source`.
4. Drop logo assets into `public/brand/`. Reference them from `components/brand/Logo.tsx`.
5. Update BUILD-STATUS.md.

## Hard rules
- Never hex-code colors directly in components — always reference the Tailwind token.
- Don't change branding.json. If a value seems wrong, log it under § "Deviations".
