# AGENTS.md

> This file points any IDE coding agent (Claude Code, Cursor, Antigravity, Windsurf, Cline) at the
> SiteCraft build kit. **Read `.sitecraft/agent/README.md` before doing anything else.**

## What this repo is

A SiteCraft-managed client website. The dashboard at SiteCraft owns:

- `.sitecraft/branding.json` — brand tokens
- `.sitecraft/customer.json` — copy, services, pages
- `.sitecraft/seo.json` — SEO config
- `.sitecraft/manifest.json` — sync metadata
- `.sitecraft/agent/` — the build kit (this file came from there)

**Never hand-edit anything under `.sitecraft/`.** Those files are overwritten on every sync.

## How to build

```
/build-site          # full build, resumes from BUILD-STATUS.md if present
/sync-status         # show where the build is
/seo-deploy          # refresh robots/sitemap/JSON-LD only
```

If your IDE doesn't have the SiteCraft plugin installed, the same commands are described in
`.sitecraft/agent/plugin/commands/*.md` — open them and follow the procedure manually.

## Tech stack

Locked. See `.sitecraft/agent/context/tech-stack.md`. Don't substitute libraries.

## Status

Live build status is in `BUILD-STATUS.md` at this repo's root. Read it first.
