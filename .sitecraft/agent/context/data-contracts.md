# Data contracts — `.sitecraft/*.json`

The dashboard is the source of truth. The agent only reads these files; it never writes them.
The full Zod schemas live in the SiteCraft repo at `lib/serializers/*.ts` — this file
mirrors them at a high level so the agent can validate quickly.

## Inputs
- (leaf — agents come here to look up shape)

## Consumed by
- Every skill that reads a `.sitecraft/*.json` file

## Files

### `branding.json`
```jsonc
{
  "colors": {
    "primary": "#0F172A",
    "accent":  "#22D3EE",
    "neutral": { "50": "...", "900": "..." }
  },
  "typography": {
    "heading": { "family": "Inter", "source": "google" },
    "body":    { "family": "Inter", "source": "google" }
  },
  "logo":    { "light": "url-or-base64", "dark": "..." },
  "favicon": "...",
  "tagline": "...",
  "brandVoice": "warm | professional | bold"
}
```

### `customer.json`
```jsonc
{
  "business": {
    "name": "...",
    "address": { "street":"...", "city":"...", "region":"...", "postal":"...", "country":"..." },
    "phone": "...",
    "email": "...",
    "hours": [{ "day":"Mon", "open":"09:00", "close":"17:00" }]
  },
  "pages": [
    { "slug":"/", "type":"home", "sections":["hero","features","cta"] },
    { "slug":"/about", "type":"about" }
  ],
  "services":     [{ "name":"...", "blurb":"...", "icon":"..." }],
  "testimonials": [{ "quote":"...", "author":"...", "role":"..." }],
  "products":     [{ "name":"...", "price":"...", "image":"..." }]
}
```

### `seo.json`
```jsonc
{
  "siteUrl": "https://example.com",
  "defaults": {
    "titleTemplate": "%s — Acme Co.",
    "description":   "...",
    "openGraph":     { "image":"..." },
    "twitter":       { "card":"summary_large_image" }
  },
  "pages": [
    { "slug":"/", "title":"Home", "description":"..." }
  ],
  "robots":  { "rules":[{ "userAgent":"*", "allow":"/", "disallow":["/api/"] }] },
  "routes":  [{ "url":"/", "changeFrequency":"weekly", "priority":1.0 }],
  "schemas": { "organization": true, "localBusiness": true, "website": true }
}
```

### `manifest.json`
```jsonc
{
  "syncedAt": "2026-04-27T12:00:00Z",
  "syncCommitSha": "...",
  "files": ["branding.json","customer.json","seo.json"],
  "agentKitVersion": "0.1.0"
}
```

## Validation rule

Before reading any of these, an agent should confirm `manifest.json` exists and lists the
file. If a file isn't in `manifest.json#files`, treat it as not yet synced — don't act on it.
