# LinkedIn MCP Server

Production-grade LinkedIn MCP server written in Rust. Works with any [Model Context Protocol](https://spec.modelcontextprotocol.io/) host: AgentOS, Claude Desktop, Cursor, Cline, and any MCP-compatible tool.

---

## Quick start

```bash
# Build from source
cargo build --release -p linkedin-mcp

# 1. Authenticate (opens browser, stores token in OS keychain)
linkedin-mcp auth

# 2. Start the MCP server over stdio
linkedin-mcp serve

# 3. Wire into your MCP host — see "Host configuration" below
```

---

## Auth

```bash
linkedin-mcp auth                              # default account, keychain store
linkedin-mcp auth --account work               # named account
linkedin-mcp auth --token-store file           # file store (Linux/systemd — see note below)
```

On success, prints:

```
Authenticated as Your Name (urn:li:person:...)
```

### Staying authenticated (long-running agents)

The access token is short-lived and refreshed automatically before expiry. For a
**confidential** LinkedIn app the refresh grant requires the client secret, so the
**serve** process must have it — otherwise the session dies when the access token
expires:

```bash
LINKEDIN_CLIENT_SECRET=... linkedin-mcp serve
# or
linkedin-mcp serve --client-secret ...
```

LinkedIn **rotates** the refresh token on each refresh and the refresh token itself
eventually expires; when it does, a human must re-run `linkedin-mcp auth` (LinkedIn
has no device/headless grant for member posting). To avoid failing mid-workflow,
agents should poll the **`linkedin-auth-status`** tool — it reports, with no network
call, whether re-auth is needed now or soon:

```json
{
  "authenticated": true,
  "access_expires_in_seconds": 3421,
  "refresh_expires_in_days": 58,
  "needs_reauth_soon": false,
  "next_action": "none"
}
```

`serve` also logs a structured warning at startup when the refresh token is within
7 days of expiry (or when `LINKEDIN_CLIENT_SECRET` is unset), suitable for alerting.

### Central broker mode (multi-machine / multi-account)

For deployments where the loopback browser flow (which needs a browser on the same
host) doesn't fit, run the [`linkedin-auth-broker`](../linkedin-auth-broker) and
point `serve` at it. The MCP then fetches already-valid access tokens from the
broker instead of using a local keychain/file store, and refresh happens centrally:

```bash
LINKEDIN_BROKER_URL=https://auth.example.com \
LINKEDIN_BROKER_TOKEN=$BROKER_API_TOKEN \
linkedin-mcp serve
# or: linkedin-mcp serve --broker-url … --broker-token …
```

In this mode the client holds no secret or refresh token: when its cached access
token nears expiry it re-fetches from the broker (which refreshes server-side). If
the broker reports re-auth is required, tool calls surface an auth error and an
operator re-runs the broker's `/li/start` flow. See the broker README for setup.

---

## Host configuration

### AgentOS

```bash
agentos mcp attach linkedin -- linkedin-mcp serve
```

> **Linux / systemd note:** AgentOS runs MCP servers as systemd user services, which may not have access to the D-Bus session bus needed by the OS keychain. If only `ping` appears after attaching, re-auth and serve with `--token-store file`:
>
> ```bash
> linkedin-mcp auth --token-store file
> agentos mcp attach linkedin -- linkedin-mcp serve --token-store file
> ```
>
> Tokens are stored in `~/.local/share/linkedin-mcp/default.json` (mode `0600`).

### Claude Desktop (`~/Library/Application Support/Claude/claude_desktop_config.json`)

```json
{
  "mcpServers": {
    "linkedin": {
      "command": "linkedin-mcp",
      "args": ["serve"]
    }
  }
}
```

### Cursor (`.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "linkedin": {
      "command": "linkedin-mcp",
      "args": ["serve"],
      "env": {}
    }
  }
}
```

---

## Multi-account

```bash
linkedin-mcp auth --account personal
linkedin-mcp auth --account work

linkedin-mcp serve --account work
```

Each account has its own keychain entry. Spawn a separate server process per account and point each MCP host entry at the appropriate one.

---

## BYO OAuth client ID

By default `linkedin-mcp` embeds a community client ID. For production deployments, supply your own:

```bash
linkedin-mcp auth --client-id YOUR_CLIENT_ID --client-secret YOUR_CLIENT_SECRET
```

Register your app in the [LinkedIn Developer Portal](https://developer.linkedin.com/) with redirect URI `http://localhost:17423/callback` and scopes `openid profile email w_member_social`.

---

## File store fallback

```bash
linkedin-mcp auth --token-store file
linkedin-mcp serve --token-store file
```

Tokens are stored as plaintext JSON at `~/.local/share/linkedin-mcp/<account>.json` (mode `0600`). Use only when the OS keychain is unavailable (e.g. Linux headless / systemd service context).

---

## Available tools

| Tool | Description |
|------|-------------|
| `linkedin-auth-status` | Report auth health/expiry and whether re-auth is needed (no network call) |
| `linkedin-whoami` | Identity of the authenticated member (name, email, picture) |
| `linkedin-posts-list` | List posts authored by the authenticated member |
| `linkedin-post-get` | Retrieve a post by URN |
| `linkedin-post-text` | Publish a text-only post |
| `linkedin-post-article` | Publish a post with an article/URL link |
| `linkedin-post-image` | Publish a post with a single image |
| `linkedin-post-multi-image` | Publish a post with multiple images |
| `linkedin-post-video` | Publish a post with a video |
| `linkedin-post-document` | Publish a post with a document (PDF, DOCX, etc.) |
| `linkedin-post-poll` | Publish a poll post |
| `linkedin-post-reshare` | Reshare an existing post |
| `linkedin-post-update` | Edit text or visibility of an existing post |
| `linkedin-post-delete` | Delete a post (must be authored by authenticated member) |
| `linkedin-comment-list` | List comments on a post |
| `linkedin-comment-create` | Add a comment to a post |
| `linkedin-comment-delete` | Delete a comment (must be authored by authenticated member) |
| `linkedin-reaction-add` | Add a reaction to a post or comment |
| `linkedin-reaction-remove` | Remove your reaction from a post |

---

## Development

```bash
# Build
cargo build

# Tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-features -- -D warnings

# Run directly (skips agentos)
cargo run --bin linkedin-mcp -- auth
cargo run --bin linkedin-mcp -- serve
```

---

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) OR [MIT](LICENSE-MIT).

Not affiliated with or endorsed by LinkedIn Corporation. "LinkedIn" is a trademark of LinkedIn Corporation.
