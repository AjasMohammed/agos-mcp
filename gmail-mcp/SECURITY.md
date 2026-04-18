# Security Policy

## Threat model

### What we protect

| Asset | Protection |
|-------|-----------|
| OAuth access tokens | Stored in OS keychain (macOS Keychain, Windows Credential Manager, Linux libsecret). Never written to disk as plaintext. |
| OAuth refresh tokens | Same keychain storage. Zeroized from memory after write. |
| Message content | Never persisted. Flows in-process over stdio and discarded. |
| Audit log | Contains only SHA-256 hashes of account identifiers and argument blobs — no raw content or tokens. |

### Scope of threat model

- **In scope:** token exfiltration via filesystem read, token exfiltration via network, prompt-injection that leaks tokens, supply-chain compromise of a dependency.
- **Out of scope:** OS-level attacker with root/admin (can read any keychain); browser-level attacker during OAuth loopback (mitigated by PKCE state parameter); server-side Google API compromise.

### Mitigations

| Threat | Mitigation |
|--------|-----------|
| Plaintext token on disk | Keychain is the default; `--file-store` triggers a loud startup warning and uses Argon2+AES-256-GCM |
| Token interception during OAuth | PKCE (S256 code challenge), random state parameter, loopback-only redirect (`127.0.0.1`) |
| Prompt injection leaking tokens | Tokens never appear in tool outputs; audit log hashes only |
| Malicious MCP host | OAuth tokens are scoped to Gmail only; all scopes are granted interactively by the user |
| Overly broad permissions | Least-privilege by default (`read` preset); write operations require explicit `--scopes write` or `--scopes full` |
| Supply-chain attack | All dependencies pinned in `Cargo.lock`; `cargo audit` runs in CI on every push; `cargo deny` enforces license and duplicate-dep policy |
| Shared community client ID rate limits | Document BYO client ID; enterprise users must use their own GCP project |

---

## Supported versions

Only the **latest stable release** receives security fixes. We do not backport patches to older minor versions.

---

## Reporting a vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Report via **GitHub Security Advisories** (private):
> https://github.com/agentos/gmail-mcp/security/advisories/new

Or email: `security@agentos.org` (PGP key available on request).

We aim to acknowledge reports within **48 hours** and ship a fix within **7 days** for critical issues.

---

## Disclosure policy

We follow coordinated disclosure. After a fix is merged and a release is tagged, we:

1. Publish a GitHub Security Advisory with CVE (if applicable).
2. Update the changelog with a `Security` entry.
3. Notify the reporter with credit (unless they prefer anonymity).

---

## Dependency audit

```bash
cargo audit       # checks against RustSec advisory DB
cargo deny check  # enforces license + ban policy
```

Both commands run in CI (`security.yml` workflow) on every push and weekly schedule.
