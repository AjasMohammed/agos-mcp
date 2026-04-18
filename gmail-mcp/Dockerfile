# syntax=docker/dockerfile:1.7

FROM --platform=$BUILDPLATFORM rust:1.85 AS build
WORKDIR /src
COPY . .
ARG TARGETARCH
RUN case "$TARGETARCH" in \
      amd64) target=x86_64-unknown-linux-musl ;; \
      arm64) target=aarch64-unknown-linux-musl ;; \
    esac && \
    rustup target add $target && \
    apt-get update && apt-get install -y musl-tools && \
    cargo build --release --target $target && \
    cp target/$target/release/gmail-mcp /out/gmail-mcp

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=build /out/gmail-mcp /usr/local/bin/gmail-mcp
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/gmail-mcp"]
CMD ["serve"]
