# Tech stack (locked)

> These are the only versions and libraries the agent may use without writing a deviation note in `BUILD-STATUS.md` § "Deviations". Updating this file is a SiteCraft-side change, not an agent decision.

## Inputs
- (none — this is a leaf node in the brain graph)

## Consumed by
- Every skill in `plugin/skills/`
- `workflow/02-scaffold.md`

## Locked choices

| Concern | Choice | Notes |
|---|---|---|
| Framework | **Next.js 15.x** App Router | Use Server Components by default; `"use client"` only when interactivity demands it |
| Language | **TypeScript 5.x**, `strict: true` | No `any`. `unknown` + narrowing for external data |
| Styling | **Tailwind CSS v4** | CSS-first config; brand tokens via CSS variables, surfaced through `tailwind.config.ts` |
| Components | **shadcn/ui** | Copy-paste under `components/ui/`. Do not add Radix primitives directly |
| Icons | **lucide-react** | One library only |
| Fonts | **next/font** (google or local) | Never `<link>` in `<head>` |
| Forms | **react-hook-form** + **zod** | Schema lives next to the form |
| Database (when needed) | **Postgres via Prisma** | Mirrors SiteCraft itself; reuse the patterns |
| Auth (when needed) | **Clerk** | Same as SiteCraft |
| Hosting target | **Vercel** | Edge-friendly defaults |
| Analytics | **Vercel Analytics** + **Speed Insights** | Free tier; opt-in via env |
| Deployment | Push to `main` → Vercel auto-deploy | No GitHub Actions in client repos |

## Versions to pin in `package.json`

```json
{
  "next": "^15.0.0",
  "react": "^19.0.0",
  "react-dom": "^19.0.0",
  "typescript": "^5.6.0",
  "tailwindcss": "^4.0.0",
  "@tailwindcss/postcss": "^4.0.0",
  "lucide-react": "^0.460.0",
  "react-hook-form": "^7.53.0",
  "zod": "^3.23.0"
}
```

## Forbidden without explicit approval

- Pages Router (use App Router)
- Material UI, Chakra, Bootstrap (use shadcn/Tailwind)
- styled-components / emotion
- Redux / MobX (Server Components + URL state + react-hook-form cover 95%)
- Custom webpack config (use Next.js defaults)
