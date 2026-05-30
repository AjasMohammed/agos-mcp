# syntax=docker/dockerfile:1.7
FROM rust:1.85 AS build
WORKDIR /src
COPY . .
RUN cargo build --release && cp target/release/linkedin-auth-broker /out-broker

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /out-broker /usr/local/bin/linkedin-auth-broker
USER nonroot:nonroot
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/linkedin-auth-broker"]
