# SiteCraft Agent Build Kit — `prompt/`

> **Purpose.** A self-contained, brain-graph-style instruction package that turns a synced
> client repo into a first-build website. Any agentic IDE (Claude Code, Cursor, Antigravity,
> Windsurf) reads this folder and runs the workflow.

This folder is **the template**. SiteCraft's sync engine copies it into each client repo
under `.sitecraft/agent/` on every sync, so every client repo carries its own up-to-date
copy alongside `branding.json`, `customer.json`, and `seo.json`.

---

## How to use (the user-facing flow)

1. Dashboard pushes the client repo (existing flow — `inngest/functions/sync-project.ts`).
2. Repo now contains both **data** (`.sitecraft/branding.json`, etc.) and **instructions**
   (`.sitecraft/agent/`, copied from this folder).
3. Open the repo in your IDE, run `/build-site` (or `/sitecraft-build`).
4. The agent reads the data + workflow, scaffolds the site, writes `BUILD-STATUS.md`, and
   keeps it updated as it works.

---

## Brain graph (read this first)

Every file in this kit names its **inputs**, **outputs**, and **siblings** at the top so an
agent dropped into any one file can find the rest. The graph:

```
                                  README.md  ← you are here
                                      │
                  ┌───────────────────┼────────────────────┐
                  ▼                   ▼                    ▼
           brain-graph.md        plugin/                 context/
           (visual map)              │                      │
                                     │                      │
                  ┌──────────────────┼──────────────┐       │
                  ▼                  ▼              ▼       │
         .claude-plugin/        commands/        skills/    │
              plugin.json           │               │       │
                                    ▼               ▼       │
                              build-site.md   site-scaffolder/SKILL.md ─┐
                              sync-status.md  brand-applier/SKILL.md    │
                              seo-deploy.md   seo-publisher/SKILL.md    │
                                              status-tracker/SKILL.md   │
                                              content-generator/SKILL.md│
                                                       │                │
                                                       ▼                │
                                              workflow/                 │
                                              01-bootstrap.md  ◄────────┤
                                              02-scaffold.md            │
                                              03-branding.md            │ all skills
                                              04-content.md             │ reference
                                              05-seo.md                 │ context/*
                                              06-verify.md              │ for tech
                                                       │                │ choices
                                                       ▼                │
                                              templates/                │
                                              BUILD-STATUS.template.md ◄┘
                                              AGENTS.template.md
```

See [`brain-graph.md`](./brain-graph.md) for the full edge list and traversal rules.

---

## File index

| Path | Role | Read when |
|---|---|---|
| [`brain-graph.md`](./brain-graph.md) | Edge list + how to traverse | First, always |
| [`plugin/.claude-plugin/plugin.json`](./plugin/.claude-plugin/plugin.json) | Claude plugin manifest | Installing the plugin |
| [`plugin/commands/build-site.md`](./plugin/commands/build-site.md) | `/build-site` slash command | User invokes build |
| [`plugin/commands/sync-status.md`](./plugin/commands/sync-status.md) | `/sync-status` slash command | User asks "where are we?" |
| [`plugin/commands/seo-deploy.md`](./plugin/commands/seo-deploy.md) | `/seo-deploy` slash command | Push robots.txt + sitemap |
| [`plugin/skills/*/SKILL.md`](./plugin/skills/) | Five skills (one per concern) | Skill auto-loads on trigger |
| [`plugin/agents/site-architect.md`](./plugin/agents/site-architect.md) | Planning subagent | Architecture decisions |
| [`context/tech-stack.md`](./context/tech-stack.md) | Next.js 15, TS, Tailwind, shadcn | Any code-writing step |
| [`context/architecture.md`](./context/architecture.md) | Folder layout + conventions | Scaffolding |
| [`context/data-contracts.md`](./context/data-contracts.md) | JSON schemas the agent consumes | Reading dashboard data |
| [`context/dashboard-fields.md`](./context/dashboard-fields.md) | Every dashboard field — what / why / source / use | Onboarding clients & filling gaps |
| [`context/integrations.md`](./context/integrations.md) | GitHub, Supabase, Vercel, Inngest | Wiring services |
| [`workflow/01..06`](./workflow/) | Step-by-step build sequence | Following the build |
| [`templates/BUILD-STATUS.template.md`](./templates/BUILD-STATUS.template.md) | Status file the agent maintains | Every step |
| [`templates/AGENTS.template.md`](./templates/AGENTS.template.md) | AGENTS.md for the client repo | Once, at scaffold time |
| [`research/plugin-checklist.md`](./research/plugin-checklist.md) | Which plugins/MCPs to install | Setup |
| [`research/setup-guide.md`](./research/setup-guide.md) | Wiring this kit into SiteCraft sync | Maintainer reference |

---

## Edges back to the SiteCraft codebase

This kit is **the source**. The sync engine should be extended to copy it into client repos:

- `lib/serializers/agent-package.ts` — new serializer (see [`research/setup-guide.md`](./research/setup-guide.md))
- `lib/serializers/index.ts` — register the new serializer in `serializeAll()`
- Client repo destination: `.sitecraft/agent/` (mirrors this folder structure)

The existing rules from `CLAUDE.md` still apply — serializers must be pure and deterministic.
