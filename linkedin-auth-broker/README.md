# linkedin-auth-broker

Centralized OAuth broker for the [LinkedIn MCP server](../linkedin-mcp). It holds
the LinkedIn **client secret** and **refresh tokens** in one place, refreshes
access tokens centrally (on demand and on a schedule), and serves short-lived
access tokens to many MCP instances over an authenticated internal API.

> **Do you need this?** Only for multi-machine / multi-account deployments. For a
> single machine, run `linkedin-mcp serve` on its own ("local mode") — no broker.
> The broker is **deploy-once infrastructure** (like a database): you run one
> instance for your whole fleet and leave it up; you do **not** start it alongside
> every MCP. See [deployment modes](../linkedin-mcp#deployment-modes).

This is **Phase 2** of the LinkedIn auth plan (see [`../plans/linkedin-auth.md`](../plans/linkedin-auth.md)).
It exists to solve what the single-instance flow can't:

- **Multi-machine** — the MCP's loopback browser flow needs a browser on the same
  host. The broker uses a single **public redirect URI**, so consent can happen
  anywhere and any number of MCP hosts can consume tokens.
- **Centralized refresh & rotation** — LinkedIn rotates refresh tokens; doing this
  in one audited place avoids races and scattered secrets.
- **Multi-account** — one stored identity per account key, not one process each.

> First-time consent still requires a human at a browser once (LinkedIn has no
> device/headless grant). Everything after that is unattended.

## Endpoints

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| `GET`  | `/healthz` | none | Liveness check |
| `POST` | `/li/start?account=<a>` | bearer | Returns the LinkedIn `authorize_url` to open in a browser |
| `GET`  | `/li/callback` | none (state) | OAuth redirect target; exchanges the code and stores the token |
| `GET`  | `/li/token?account=<a>` | bearer | Returns a currently-valid access token, refreshing first if needed |

Internal endpoints require `Authorization: Bearer $BROKER_API_TOKEN`. The callback
is public (LinkedIn calls it) and is protected against CSRF by a single-use,
expiring `state`.

`/li/token` returns:

```json
{
  "account": "default",
  "access_token": "…",
  "expires_in_seconds": 3421,
  "author_urn": "urn:li:person:…",
  "scopes": ["openid", "profile", "email", "w_member_social"],
  "needs_reauth_soon": false
}
```

It responds `409 reauth_required` when the refresh token is dead — the operator
must re-run `/li/start` for that account.

## Configuration (env)

| Var | Required | Default | Meaning |
|-----|----------|---------|---------|
| `LINKEDIN_CLIENT_ID` | ✅ | — | LinkedIn app client id |
| `LINKEDIN_CLIENT_SECRET` | ✅ | — | Client secret (lives only here) |
| `BROKER_PUBLIC_URL` | ✅ | — | Public base URL; redirect URI is `<this>/li/callback` |
| `BROKER_API_TOKEN` | ✅ | — | Bearer token for internal endpoints |
| `BROKER_BIND_ADDR` | | `0.0.0.0:8080` | Listen address |
| `BROKER_STORE` | | `file` | `file` (persistent) or `memory` (ephemeral/dev) |
| `BROKER_STORE_DIR` | | OS data dir | Directory for the file store |
| `BROKER_REFRESH_SCAN_SECS` | | `300` | Background refresh scan interval |

Register `${BROKER_PUBLIC_URL}/li/callback` as an authorized redirect URL in your
LinkedIn app, with scopes `openid profile email w_member_social`.

## Run

```bash
LINKEDIN_CLIENT_ID=… LINKEDIN_CLIENT_SECRET=… \
BROKER_PUBLIC_URL=https://auth.example.com \
BROKER_API_TOKEN=$(openssl rand -hex 32) \
cargo run --release

# enroll an account (returns authorize_url to open in a browser)
curl -s -X POST -H "Authorization: Bearer $BROKER_API_TOKEN" \
  "http://localhost:8080/li/start?account=default"

# later: fetch a valid access token
curl -s -H "Authorization: Bearer $BROKER_API_TOKEN" \
  "http://localhost:8080/li/token?account=default"
```

Or via Docker:

```bash
docker build -t linkedin-auth-broker .
docker run -p 8080:8080 --env-file broker.env linkedin-auth-broker
```

## Security model

- The client secret and refresh tokens never leave the broker; MCP hosts hold
  only short-lived access tokens fetched from `/li/token`.
- Internal endpoints are gated by `BROKER_API_TOKEN`. Terminate TLS in front of
  the broker (or run behind a mesh); for stronger guarantees use mTLS.
- The `file` store writes tokens as `0600` plaintext JSON. Run on an encrypted
  volume, or implement an encrypted / secret-manager-backed `BrokerStore`
  (the trait is the single seam — `src/store.rs`).

## MCP integration (next step)

The MCP consumes the broker via a `RemoteStore` implementing `linkedin-mcp`'s
`TokenStore` trait, fetching tokens from `/li/token` instead of refreshing
locally. That wiring (making `TokenStore` async + reload-on-expiry) is the
remaining task and is intentionally not yet bundled — see the plan.

## Scaling

The current refresh path uses a coarse global lock and a file/in-memory store —
correct for a single node. For HA/multi-node, implement a Postgres-backed
`BrokerStore` (per-account row locks for refresh) behind the same trait; no other
code changes are required.
