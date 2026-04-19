# agos-mcp

MCP servers for **AgentOS** — production-grade integrations built in Rust.

Each server is a standalone binary that implements the [Model Context Protocol](https://spec.modelcontextprotocol.io/) and works with any compatible host: AgentOS, Claude Desktop, Cursor, Cline, and more.

---

## Servers

| Server | Description | Status |
|--------|-------------|--------|
| [`gmail-mcp`](./gmail-mcp) | Gmail integration — 25+ tools for search, read, send, drafts, labels, filters, attachments | Active |

---

## Design principles

- **Single static binaries** — no Node, Python, or runtime to install; cold-start < 100 ms
- **Secure by default** — OS keychain token storage, least-privilege OAuth scopes, typed errors
- **Production-ready** — rate limiting, structured audit logging, exponential back-off, signed releases

---

## Getting started

See the README inside each server's directory for install and configuration instructions.

```bash
# Gmail MCP
cd gmail-mcp
cargo build --release
./target/release/gmail-mcp auth
./target/release/gmail-mcp serve
```

---

## License

Dual-licensed under [MIT](./gmail-mcp/LICENSE-MIT) and [Apache 2.0](./gmail-mcp/LICENSE-APACHE).
