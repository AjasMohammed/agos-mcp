---
description: Show the current build status by reading BUILD-STATUS.md and listing remaining steps. Use when resuming work or asking "where are we?".
allowed-tools: Read, Glob
---

# /sync-status

Read `BUILD-STATUS.md` at the repo root and report:

1. **Phase** — which workflow step is in progress (01-bootstrap … 06-verify)
2. **Done** — checked items
3. **Next** — first unchecked item (this is what `/build-site` will resume from)
4. **Blockers** — any `[!]` items, with the recorded reason
5. **Last updated** — timestamp from the file

If `BUILD-STATUS.md` does not exist, say so and recommend running `/build-site --mode=fresh`.

Do not modify any files — this command is read-only.
