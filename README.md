# agos-mcp

MCP servers for **AgentOS** — production-grade integrations built in Rust.

Each server is a standalone binary that implements the [Model Context Protocol](https://spec.modelcontextprotocol.io/) and works with any compatible host: AgentOS, Claude Desktop, Cursor, Cline, and more.

---

## Servers

| Server | Description | Status |
|--------|-------------|--------|
| [`gmail-mcp`](./gmail-mcp) | Gmail integration — 25+ tools for search, read, send, drafts, labels, filters, attachments | Active |
| [`linkedin-mcp`](./linkedin-mcp) | LinkedIn integration — posts (text, article, image, document, multi-image, poll, reshare), OAuth | Active |

---

## Design principles

- **Single static binaries** — no Node, Python, or runtime to install; cold-start < 100 ms
- **Secure by default** — OS keychain token storage, least-privilege OAuth scopes, typed errors
- **Production-ready** — rate limiting, structured audit logging, exponential back-off, signed releases

---

## Getting started

### Install (prebuilt binary — no build required)

One command downloads the right binary for your OS/arch from the latest
[GitHub Release](https://github.com/AjasMohammed/agos-mcp/releases) and installs
it to `~/.local/bin`:

```bash
# Gmail MCP
curl -fsSL https://raw.githubusercontent.com/AjasMohammed/agos-mcp/main/install.sh | sh -s -- gmail

# LinkedIn MCP
curl -fsSL https://raw.githubusercontent.com/AjasMohammed/agos-mcp/main/install.sh | sh -s -- linkedin

# Both
curl -fsSL https://raw.githubusercontent.com/AjasMohammed/agos-mcp/main/install.sh | sh -s -- all
```

Then authenticate and attach — no local build path needed:

```bash
gmail-mcp auth                              # opens browser, stores token in OS keychain
agentos mcp attach gmail -- gmail-mcp serve
```

> Releases are produced automatically by [`.github/workflows/release.yml`](./.github/workflows/release.yml)
> when a `v*` tag is pushed (`git tag v0.1.0 && git push origin v0.1.0`). The installer
> requires that at least one release exists.

### Build from source (for development)

```bash
cd gmail-mcp
cargo build --release
./target/release/gmail-mcp auth
./target/release/gmail-mcp serve
```

---

## License

Dual-licensed under [MIT](./gmail-mcp/LICENSE-MIT) and [Apache 2.0](./gmail-mcp/LICENSE-APACHE).
