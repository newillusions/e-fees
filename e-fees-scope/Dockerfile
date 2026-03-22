FROM rust:1.89-slim AS builder

WORKDIR /app

# Install build deps (curl for utoipa-swagger-ui, git for container-utils)
RUN apt-get update && apt-get install -y curl git pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Clone container-utils from Forgejo (path dependency at ../../../container-utils/rust)
RUN git clone --depth 1 https://forge.mms.name/emittiv/container-utils.git /container-utils

# Minimal workspace (only scope + core)
# Create a minimal workspace and override the container-utils path dependency
# to point to the cloned repo location.
RUN printf '[workspace]\nmembers = ["crates/e-fees-core", "e-fees-scope"]\nresolver = "2"\n\n[workspace.dependencies]\nemittiv-container-utils = { path = "/container-utils/rust" }\n\n[patch.crates-io]\nemittiv-container-utils = { path = "/container-utils/rust" }\n' > Cargo.toml
COPY crates/ crates/
COPY e-fees-scope/ e-fees-scope/

RUN cargo build --release -p e-fees-scope

# Runtime
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/e-fees-scope /usr/local/bin/

EXPOSE 3201

CMD ["e-fees-scope"]
