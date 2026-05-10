---
name: status-tracker
description: Maintain BUILD-STATUS.md as the single source of truth for build progress. Every other skill calls this one before returning. Use when a checklist item completes, fails, is skipped, or when "Last updated" needs to refresh.
---

## Inputs
- `BUILD-STATUS.md` (current state)
- `.sitecraft/agent/templates/BUILD-STATUS.template.md` (only if status doesn't exist yet)

## Outputs
- Updated `BUILD-STATUS.md` — one section touched per call

## Procedure
1. If status file is missing, copy the template to repo root.
2. Locate the section matching the calling skill (Scaffold / Branding / Content / SEO / Verify).
3. Mutate exactly one of:
   - `[ ]` → `[x]`  (success)
   - `[ ]` → `[!]`  (blocker — also append a one-line reason under § "Blockers")
   - `[ ]` → `[~]`  (skipped on purpose — note why)
4. Append one line to § "Activity log": `- YYYY-MM-DD HH:MM <skill> — <action>`.
5. Update the top-of-file `Last updated:` timestamp.

## Hard rules
- Never rewrite the whole file — only the targeted section.
- Never delete blocker entries; mark them resolved by adding `(resolved <timestamp>)`.
- Status file is human-readable Markdown. Don't add fenced JSON, frontmatter, or HTML.
