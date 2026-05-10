# Setup guide — wiring this kit into the SiteCraft sync

For SiteCraft maintainers. Explains how the kit gets from this repo into client repos.

## Inputs
- This whole `prompt/` folder
- `lib/serializers/index.ts`, `lib/serializers/*.ts` (existing serializers)
- `inngest/functions/sync-project.ts` (existing sync engine)
- `CLAUDE.md` rule #6 — serializers must be pure & deterministic

## What changes in the SiteCraft repo

### 1. Add a new serializer: `lib/serializers/agent-package.ts`

```ts
import type { ProjectFull } from '@/lib/db'
import type { SerializedFile } from './index'
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

const KIT_ROOT = join(process.cwd(), 'prompt')
const DEST_PREFIX = '.sitecraft/agent'

export function serializeAgentPackage(_project: ProjectFull): SerializedFile[] {
  // Walk prompt/ recursively, mirror to .sitecraft/agent/ in the client repo.
  // Pure: same prompt/ contents → byte-identical output. The project arg is
  // unused today but kept for future per-client customization.
  const files: SerializedFile[] = []
  walk(KIT_ROOT, (abs) => {
    const rel = relative(KIT_ROOT, abs).replaceAll('\\', '/')
    files.push({
      path: `${DEST_PREFIX}/${rel}`,
      contents: readFileSync(abs, 'utf8'),
    })
  })
  // Stable order — required for byte-identical determinism
  files.sort((a, b) => a.path.localeCompare(b.path))
  return files
}

function walk(dir: string, visit: (abs: string) => void) {
  for (const name of readdirSync(dir).sort()) {
    const abs = join(dir, name)
    const s = statSync(abs)
    if (s.isDirectory()) walk(abs, visit)
    else if (s.isFile()) visit(abs)
  }
}
```

### 2. Register in `lib/serializers/index.ts`

```ts
import { serializeAgentPackage } from './agent-package'

export function serializeAll(project: ProjectFull): SerializedFile[] {
  return [
    ...serializeManifest(project),
    ...serializeBranding(project),
    ...serializeCustomer(project),
    ...serializeSeo(project),
    ...serializeBlog(project),
    ...serializeSitemap(project),
    ...serializeAgentPackage(project),   // ← new
  ]
}
```

### 3. Snapshot test

`lib/serializers/__tests__/agent-package.test.ts` — assert the output paths and that
`README.md` contains the brain-graph header. Re-run on every PR.

### 4. Manifest update

Extend `serializeManifest` to include `agentKitVersion` (read from
`prompt/plugin/.claude-plugin/plugin.json#version`). Bumping the version in the plugin
file is the signal to clients that the kit changed.

### 5. No changes needed to `inngest/functions/sync-project.ts`

`serializeAll()` is the only edge — the new serializer flows through automatically and
gets committed via the existing `commitFiles()` helper (CLAUDE.md rule #5 still satisfied).

## What the client experiences

After the next sync:

```
client-repo/
  .sitecraft/
    branding.json
    customer.json
    seo.json
    manifest.json
    agent/                   ← NEW: full mirror of prompt/
      README.md
      brain-graph.md
      plugin/...
      context/...
      workflow/...
      templates/...
      research/...
```

They open the repo, run `/plugin install ./.sitecraft/agent/plugin` (Claude Code) or just
read `AGENTS.md` (other agents), then run `/build-site`.

## Versioning

The kit version (in `plugin.json`) is the contract. Bump:

- **patch** — wording, comments, prompt tweaks
- **minor** — new skill, new command, new workflow step
- **major** — change to JSON contracts, file paths, or skill APIs

Major bumps require a parallel update to `lib/serializers/*.ts` (they share contracts).

## Cleanup

`.sitecraft/agent/` mirrors `prompt/`. If a file is removed from `prompt/`, the next sync
**must** delete it from the client repo. The current `commitFiles` helper in
`lib/github/commit.ts` handles upserts but check whether it deletes orphans — if not,
add a "tombstone" pass that lists `.sitecraft/agent/**` in the client tree, diffs against
the new serializer output, and deletes the difference. (Open as a follow-up issue.)
