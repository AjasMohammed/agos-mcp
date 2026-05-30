#!/bin/sh
# install.sh — one-command installer for the AgentOS MCP servers.
#
# Downloads a prebuilt binary from the latest GitHub Release (no Rust toolchain,
# no building from source) and drops it into an install dir on your PATH.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/AjasMohammed/agos-mcp/main/install.sh | sh -s -- gmail
#   curl -fsSL https://raw.githubusercontent.com/AjasMohammed/agos-mcp/main/install.sh | sh -s -- linkedin
#   curl -fsSL https://raw.githubusercontent.com/AjasMohammed/agos-mcp/main/install.sh | sh -s -- all
#
# Env overrides:
#   AGOS_VERSION=v0.1.0     pin a release tag (default: latest)
#   AGOS_BIN_DIR=~/.local/bin   install location (default: ~/.local/bin)
set -eu

REPO="AjasMohammed/agos-mcp"
BIN_DIR="${AGOS_BIN_DIR:-$HOME/.local/bin}"
VERSION="${AGOS_VERSION:-latest}"

say()  { printf '\033[1;34m==>\033[0m %s\n' "$1"; }
warn() { printf '\033[1;33mwarn:\033[0m %s\n' "$1" >&2; }
die()  { printf '\033[1;31merror:\033[0m %s\n' "$1" >&2; exit 1; }

# ---- which servers to install --------------------------------------------
WHICH="${1:-all}"
case "$WHICH" in
  gmail)    PROJECTS="gmail-mcp" ;;
  linkedin) PROJECTS="linkedin-mcp" ;;
  all|"")   PROJECTS="gmail-mcp linkedin-mcp" ;;
  *) die "unknown target '$WHICH' (expected: gmail | linkedin | all)" ;;
esac

# ---- detect platform -> target triple -------------------------------------
os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
  Linux)  os_part="unknown-linux-musl" ;;
  Darwin) os_part="apple-darwin" ;;
  MINGW*|MSYS*|CYGWIN*) die "Windows: download the .zip from https://github.com/$REPO/releases manually" ;;
  *) die "unsupported OS: $os" ;;
esac
case "$arch" in
  x86_64|amd64)  arch_part="x86_64" ;;
  arm64|aarch64) arch_part="aarch64" ;;
  *) die "unsupported architecture: $arch" ;;
esac
TARGET="${arch_part}-${os_part}"
say "platform: $TARGET"

# ---- resolve version tag ---------------------------------------------------
if [ "$VERSION" = "latest" ]; then
  say "resolving latest release..."
  # Capture the full response first; piping curl into `grep -m1` makes grep
  # close the pipe early, which trips curl's "(23) Failure writing output".
  resp="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest")"
  VERSION="$(printf '%s\n' "$resp" | grep '"tag_name"' | head -n1 | cut -d'"' -f4)"
  [ -n "$VERSION" ] || die "could not find a published release. Push a v* tag to trigger the release workflow first."
fi
say "version: $VERSION"

mkdir -p "$BIN_DIR"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# ---- download + install each requested binary ------------------------------
for proj in $PROJECTS; do
  asset="${proj}-${VERSION}-${TARGET}.tar.gz"
  url="https://github.com/$REPO/releases/download/$VERSION/$asset"
  say "downloading $asset"
  curl -fsSL "$url" -o "$tmp/$asset" \
    || die "download failed: $url (does this release include $TARGET?)"
  tar -xzf "$tmp/$asset" -C "$tmp"
  install -m 0755 "$tmp/${proj}-${VERSION}-${TARGET}/${proj}" "$BIN_DIR/${proj}"
  say "installed $proj -> $BIN_DIR/$proj"
done

# ---- PATH hint -------------------------------------------------------------
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) warn "$BIN_DIR is not on your PATH. Add this to your shell profile:"
     printf '\n    export PATH="%s:$PATH"\n\n' "$BIN_DIR" ;;
esac

# ---- next steps ------------------------------------------------------------
echo
say "Done. Next steps:"
for proj in $PROJECTS; do
  echo
  echo "  # 1. Authenticate (opens a browser, stores token in OS keychain):"
  echo "  $proj auth"
  echo
  echo "  # 2. Attach to AgentOS (no build path needed):"
  echo "  agentos mcp attach ${proj%-mcp} -- $proj serve"
done
