---
description: Build a first-cut Next.js site from synced SiteCraft data (.sitecraft/*.json). Reads branding, customer, and SEO data, scaffolds the site, applies the brand, generates content, configures SEO, and writes BUILD-STATUS.md.
argument-hint: "[--mode=fresh|resume]  (default: resume — picks up from BUILD-STATUS.md)"
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
---

# /build-site

You are about to run the SiteCraft build workflow on this client repo.

## Inputs you must read first

1. `.sitecraft/branding.json` — colors, fonts, logos
2. `.sitecraft/customer.json` — client info, copy, products/services
3. `.sitecraft/seo.json` — meta, robots, sitemap, schema config
4. `.sitecraft/agent/README.md` — kit entry point
5. `.sitecraft/agent/brain-graph.md` — file map
6. `BUILD-STATUS.md` (if it exists) — resume point

## Procedure

1. If `--mode=fresh` or `BUILD-STATUS.md` does not exist, copy
   `.sitecraft/agent/templates/BUILD-STATUS.template.md` → `BUILD-STATUS.md` at repo root.
2. Walk the workflow in order — `.sitecraft/agent/workflow/01-bootstrap.md` through `06-verify.md`.
3. After **every** sub-step, update `BUILD-STATUS.md` (mark the checkbox, append a one-line note
   under "Activity log", set "Last updated" timestamp).
4. If a step fails, mark the box `[!]`, write the error under "Blockers", and stop.
5. On success of step 06, post a short summary in chat: pages created, brand tokens applied,
   SEO files generated, dev server URL.

## Hard rules

- Never edit `.sitecraft/*.json` or `.sitecraft/agent/**` — they are owned by the dashboard.
- Never invent customer copy not derivable from `customer.json`. If a field is missing, mark
  the section TODO in `BUILD-STATUS.md` instead of hallucinating.
- Use the tech stack in `.sitecraft/agent/context/tech-stack.md` exactly — don't substitute
  alternative libraries.
- All file changes must go through Edit/Write — never `cat >` or `sed` via Bash.

## Outputs

- A working `app/` directory with home + key pages
- `tailwind.config.ts` with brand tokens
- `app/robots.ts`, `app/sitemap.ts`, JSON-LD components
- `BUILD-STATUS.md` fully updated
- `AGENTS.md` written from `templates/AGENTS.template.md` (first run only)
