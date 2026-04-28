# gmail-mcp

**A standalone, production-grade Gmail MCP server written in Rust.**

Works with any [Model Context Protocol](https://spec.modelcontextprotocol.io/) host: AgentOS, Claude Desktop, Cursor, Cline, and any MCP-compatible tool.

---

## Features

- **25+ tools** — search, read, threads, labels, filters, drafts, attachments, send, modify, batch ops
- **Secure auth** — OS keychain storage (macOS Keychain, Windows Credential Manager, Linux libsecret); encrypted-file fallback; PKCE loopback OAuth; device-code flow; service-account support
- **Least-privilege scopes** — three presets: `read`, `write`, `full`
- **Rate limiting** — token-bucket (250 QU/s) with 30-second guard; 429s trigger exponential back-off with jitter
- **Typed errors** — `RateLimited`, `ScopeMissing`, `NotFound` etc. surfaces actionable messages to the LLM
- **Structured audit log** — JSON to stderr; compatible with syslog, file rotation, or any log aggregator
- **Single static binary** — no Node, no Python, no runtime to install; cold-start in < 100 ms
- **Signed releases** — GitHub Releases with cosign signatures + SBOM

---

## Quick start

```bash
# Install (one of)
cargo install gmail-mcp                           # from crates.io (once published)
brew install agentos-foundry/tap/gmail-mcp        # Homebrew tap
docker pull ghcr.io/agentos/gmail-mcp:latest      # Docker

# 1. Authenticate (opens browser, stores token in OS keychain — nothing touches disk as plaintext)
gmail-mcp auth                                    # read-only (default)
gmail-mcp auth --scopes write                     # + send/draft/modify
gmail-mcp auth --scopes full                      # all scopes

# 2. Start the MCP server over stdio
gmail-mcp serve

# 3. Wire into your MCP host — see "Host configuration" below
```

---

## Scope presets

| Preset | Grants | Allowed tools |
|--------|--------|---------------|
| `read` | `gmail.readonly` | search, read, thread, labels list, filters list, drafts list, profile, attachment download |
| `write` | `gmail.modify` + `gmail.send` | all read tools + send, create/update/delete drafts, modify labels, trash, batch ops, label/filter CRUD |
| `full`  | `gmail.labels` + `gmail.modify` + `gmail.send` | everything |

Tools that require an un-granted scope return a typed `ScopeMissing` error the host can surface as *"grant more permissions"*.

---

## Host configuration

### Claude Desktop (`~/Library/Application Support/Claude/claude_desktop_config.json`)

```json
{
  "mcpServers": {
    "gmail": {
      "command": "gmail-mcp",
      "args": ["serve"]
    }
  }
}
```

### Cursor (`.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "gmail": {
      "command": "gmail-mcp",
      "args": ["serve"],
      "env": {}
    }
  }
}
```

### AgentOS

```bash
agentos mcp install gmail
```

> **Linux / systemd note:** AgentOS runs MCP servers as systemd user services, which may not have access to the D-Bus session bus needed by the OS keychain. If tools are missing after attaching, re-auth and serve with `--file-store`:
>
> ```bash
> gmail-mcp auth --file-store
> agentos mcp attach gmail -- gmail-mcp serve --file-store
> ```

---

## Multi-account

```bash
gmail-mcp auth --account work
gmail-mcp auth --account personal

gmail-mcp serve --account work       # serve a single identity
```

Each account has its own keychain entry. To serve multiple accounts simultaneously, spawn separate server processes (one per account) and configure each MCP host entry to point to the appropriate one.

---

## BYO OAuth client ID

By default `gmail-mcp` embeds a community client ID subject to Google's shared rate limits. For production/enterprise deployments, supply your own:

```bash
GMAIL_MCP_CLIENT_ID=your-client-id gmail-mcp auth --scopes write
# or
gmail-mcp auth --client-id your-client-id --scopes write
```

Register your app in [Google Cloud Console](https://console.cloud.google.com/) with redirect URI `http://127.0.0.1` (loopback).

---

## File store fallback (not recommended)

```bash
gmail-mcp auth --file-store
```

Tokens are stored in an Argon2+AES-256-GCM encrypted file. A warning is printed on every startup. **Not recommended**—use only when the OS keychain is unavailable.

---

## Available tools

| Tool | Description |
|------|-------------|
| `gmail_search` | Search with Gmail query syntax |
| `gmail_read` | Read a message by ID |
| `gmail_get_thread` | Get a full thread |
| `gmail_list_labels` | List all labels |
| `gmail_get_label` | Get label by ID |
| `gmail_list_filters` | List all filters |
| `gmail_get_filter` | Get filter by ID |
| `gmail_list_drafts` | List drafts |
| `gmail_get_profile` | Get mailbox profile |
| `gmail_download_attachment` | Download a message attachment |
| `gmail_send` | Compose and send a new message |
| `gmail_create_draft` | Create a draft |
| `gmail_update_draft` | Update an existing draft |
| `gmail_send_draft` | Send a saved draft |
| `gmail_delete_draft` | Delete a draft |
| `gmail_modify_labels` | Add/remove labels on a message |
| `gmail_trash` | Move message to trash |
| `gmail_untrash` | Restore message from trash |
| `gmail_create_label` | Create a label |
| `gmail_update_label` | Update a label |
| `gmail_delete_label` | Delete a label |
| `gmail_get_or_create_label` | Get label by name, create if absent |
| `gmail_create_filter` | Create a filter |
| `gmail_delete_filter` | Delete a filter |
| `gmail_create_filter_from_template` | Create filter from a named template |
| `gmail_batch_modify_labels` | Modify labels on up to 500 messages |
| `gmail_batch_trash` | Trash multiple messages concurrently |
| `gmail_batch_delete` | Permanently delete up to 500 messages |

---

## Audit log

All tool calls emit a structured JSON audit event to stderr:

```json
{
  "ts": "2026-04-18T14:00:00Z",
  "event": "tool_call",
  "account": "default",
  "account_email_hash": "sha256:...",
  "tool": "gmail_search",
  "args_hash": "sha256:...",
  "scopes_used": ["gmail.readonly"],
  "result": "ok",
  "duration_ms": 142,
  "gmail_cost_units": 5,
  "trace_id": "abc123"
}
```

Redirect stderr to a log aggregator, file, or syslog as needed.

---

## Development

```bash
# Build
cargo build

# Tests (no Gmail account needed)
cargo test --workspace

# Lint
cargo clippy --workspace --all-features -- -D warnings

# Security audit
cargo audit
```

---

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) OR [MIT](LICENSE-MIT).

Not affiliated with or endorsed by Google. "Gmail" is a trademark of Google LLC.
