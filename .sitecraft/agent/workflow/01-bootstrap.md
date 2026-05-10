# Step 01 — Bootstrap

## Inputs
- `.sitecraft/manifest.json`
- `.sitecraft/agent/context/data-contracts.md`
- `.sitecraft/agent/templates/BUILD-STATUS.template.md`

## Outputs
- `BUILD-STATUS.md` at repo root (if absent)
- `AGENTS.md` at repo root (if absent)

## Procedure
1. Confirm `.sitecraft/manifest.json` exists. If not, stop — sync hasn't run.
2. Confirm `branding.json`, `customer.json`, `seo.json` are listed in `manifest#files`.
3. If `BUILD-STATUS.md` is missing, copy template → repo root and stamp `Last updated`.
4. If `AGENTS.md` is missing, copy `templates/AGENTS.template.md` → repo root.
5. Read all three data files into memory and sanity-check required fields per
   `context/data-contracts.md`. Missing required field → mark `[!]` on the matching
   workflow row in BUILD-STATUS, append blocker, stop.

## Next
→ `02-scaffold.md`
