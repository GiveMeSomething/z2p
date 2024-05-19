# Build binary from source
FROM rust:1.78.0 AS build
WORKDIR /app

RUN apt update \
  && apt install lld clang -y
COPY . .

ENV SQLX_OFFLINE true
RUN cargo build --release

# Smaller runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install openssl - it's dynamically linked by some of our dependencies
# Install ca-certificates - it's needed to verify TLS certificates when establishing HTTPS connections
RUN apt-get update \
  && apt-get install openssl -y \
  && apt-get install ca-certificates -y \
  && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/z2p z2p
COPY config config
ENV APP_ENV production
ENTRYPOINT [ "./z2p" ]

