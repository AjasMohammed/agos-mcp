# syntax=docker/dockerfile:1.7

# ── build ────────────────────────────────────────────────────────────────────
FROM rust:1.85-slim-bookworm AS build
WORKDIR /src

# keyring v3 on Linux links against libsecret
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        libsecret-1-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release && \
    cp target/release/linkedin-mcp /linkedin-mcp

# ── runtime ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        libsecret-1-0 \
    && rm -rf /var/lib/apt/lists/*

# Token files land in /data — mount a volume here for persistence.
# FileStore reads dirs::data_dir() which respects XDG_DATA_HOME.
ENV XDG_DATA_HOME=/data
VOLUME ["/data"]

COPY --from=build /linkedin-mcp /usr/local/bin/linkedin-mcp
RUN useradd --system --no-create-home --shell /sbin/nologin --uid 65532 nonroot
USER nonroot:nonroot

ENTRYPOINT ["/usr/local/bin/linkedin-mcp"]
# Use file store: no secret-service daemon available inside a container.
# Run `auth --token-store file` on the host first, then mount the token dir.
CMD ["serve", "--token-store", "file"]
