# Plugin & MCP checklist

What to install in your IDE so `/build-site` and friends work end-to-end. Split by IDE
since each has a different mechanism.

## Inputs
- (leaf — read this when setting up a new client repo locally)

## Consumed by
- `setup-guide.md`

---

## Claude Code

The kit **is itself** a Claude Code plugin (see `plugin/.claude-plugin/plugin.json`).
Two ways to use it:

### Option A — install from the synced repo (recommended for clients)

The plugin lives at `.sitecraft/agent/plugin/` in every client repo. From inside the repo:

```
/plugin install ./.sitecraft/agent/plugin
```

That registers the commands (`/build-site`, `/sync-status`, `/seo-deploy`), the skills,
and the `site-architect` subagent for that workspace only.

### Option B — install from a marketplace (recommended for the SiteCraft team)

Publish `prompt/plugin/` as a marketplace entry on a SiteCraft-controlled GitHub repo and
let team members `/plugin marketplace add anew/sitecraft-plugin` once.

### Useful built-ins (no install)

- **shadcn-ui MCP** *(optional)* — speeds up `components/ui/` generation. If unavailable,
  skills fall back to copying primitives via the shadcn CLI.
- **Supabase MCP** — already in this SiteCraft repo's config; useful if a client site needs
  auth/db. Skills only invoke it when `customer.json` requires it.
- **Context7 MCP** — for live Next.js / Tailwind / shadcn docs lookup during scaffolding.
  Helps the agent avoid stale-API mistakes.

---

## Cursor

Cursor doesn't speak the Claude plugin format directly. Instead:

1. Add `.cursor/rules/sitecraft.mdc` pointing at `.sitecraft/agent/README.md` — this makes
   the kit always-loaded context.
2. Use Cursor's Composer with the prompt: `Run the build-site command at .sitecraft/agent/plugin/commands/build-site.md`.
3. MCPs to enable in Cursor settings: Context7, Supabase (if needed).

A future enhancement: ship a small Cursor-specific shim under `prompt/plugin-cursor/` that
mirrors the same commands as `.mdc` rule files.

---

## Antigravity / Windsurf / Cline

All three read `AGENTS.md` and project rules folders. The `templates/AGENTS.template.md`
this kit ships is enough to get them oriented — they'll find `.sitecraft/agent/README.md`
and follow the workflow files step by step.

---

## What the SiteCraft team installs once

This repo (the dashboard) needs no IDE plugins to ship the kit. It needs:

- **GitHub App** (`sitecraft-bot`) with `contents: write`, `pull-requests: write` — already configured.
- **Inngest** — already configured.
- **Vercel** for hosting the dashboard — already configured.

The new bit is the serializer described in `setup-guide.md`.
