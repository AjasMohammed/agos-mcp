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
