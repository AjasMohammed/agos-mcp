# Brain Graph — file relationships

> Each node names what it **needs to read** before acting and what it **produces or updates**.
> Follow the edges. If you arrive at a node and don't have its inputs loaded, walk back.

## Traversal rules

1. **Entry point is always [`README.md`](./README.md).** It links to everything.
2. Every file in this kit has a `## Inputs` and `## Outputs` block at the top — use those
   as edges, not file proximity.
3. Skills reference workflow steps by number (`workflow/03-branding.md`), never by relative
   guess. Don't invent paths.
4. When confused, run [`/sync-status`](./plugin/commands/sync-status.md) — it re-reads
   `BUILD-STATUS.md` and tells you where you are.

## Edge list

```
README.md ──▶ brain-graph.md
README.md ──▶ plugin/.claude-plugin/plugin.json
README.md ──▶ plugin/commands/{build-site,sync-status,seo-deploy}.md
README.md ──▶ context/{tech-stack,architecture,data-contracts,integrations}.md
README.md ──▶ workflow/01..06
README.md ──▶ research/{plugin-checklist,setup-guide}.md

plugin/commands/build-site.md ──▶ workflow/01-bootstrap.md (entry)
workflow/01-bootstrap.md ──▶ context/data-contracts.md (read schemas)
workflow/01-bootstrap.md ──▶ templates/BUILD-STATUS.template.md (init status)
workflow/01-bootstrap.md ──▶ workflow/02-scaffold.md (next)

workflow/02-scaffold.md ──▶ context/tech-stack.md
workflow/02-scaffold.md ──▶ context/architecture.md
workflow/02-scaffold.md ──▶ plugin/skills/site-scaffolder/SKILL.md
workflow/02-scaffold.md ──▶ workflow/03-branding.md

workflow/03-branding.md ──▶ plugin/skills/brand-applier/SKILL.md
workflow/03-branding.md ──▶ .sitecraft/branding.json (in client repo)
workflow/03-branding.md ──▶ workflow/04-content.md

workflow/04-content.md ──▶ plugin/skills/content-generator/SKILL.md
workflow/04-content.md ──▶ .sitecraft/customer.json (in client repo)
workflow/04-content.md ──▶ workflow/05-seo.md

workflow/05-seo.md ──▶ plugin/skills/seo-publisher/SKILL.md
workflow/05-seo.md ──▶ .sitecraft/seo.json (in client repo)
workflow/05-seo.md ──▶ workflow/06-verify.md

workflow/06-verify.md ──▶ plugin/skills/status-tracker/SKILL.md
workflow/06-verify.md ──▶ BUILD-STATUS.md (final update)

ALL skills ──▶ context/tech-stack.md (no exceptions)
ALL skills ──▶ plugin/skills/status-tracker/SKILL.md (must update status)
```

## Status file is the heartbeat

`BUILD-STATUS.md` (created from [`templates/BUILD-STATUS.template.md`](./templates/BUILD-STATUS.template.md))
lives in the **client repo root** and is the single source of truth for build progress.
Every skill updates it before returning. Every command reads it before acting.

If you're an agent reading this and `BUILD-STATUS.md` doesn't exist yet, you're at step 01.
If it exists, find the first unchecked task and resume from there.
