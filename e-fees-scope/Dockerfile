FROM rust:1.89-slim AS builder

WORKDIR /app

# Install build deps (curl for utoipa-swagger-ui, git for container-utils)
RUN apt-get update && apt-get install -y curl git pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Clone container-utils from Forgejo (path dependency at ../../../container-utils/rust)
RUN git clone --depth 1 https://forge.mms.name/emittiv/container-utils.git /container-utils

# Minimal workspace (only scope + core)
RUN printf '[workspace]\nmembers = ["crates/e-fees-core", "e-fees-scope"]\nresolver = "2"\n' > Cargo.toml

# Skip Cargo.lock — the minimal workspace differs from the full workspace,
# so Cargo regenerates with correct path dependency resolution.
COPY crates/ crates/
COPY e-fees-scope/ e-fees-scope/

RUN cargo build --release -p e-fees-scope

# Runtime
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/e-fees-scope /usr/local/bin/

EXPOSE 3201

CMD ["e-fees-scope"]
