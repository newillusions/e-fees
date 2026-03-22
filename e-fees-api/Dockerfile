FROM rust:1.89-slim AS builder

WORKDIR /app

# Install build deps (curl for utoipa-swagger-ui, git for container-utils)
RUN apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*

# Clone container-utils from Forgejo (path dependency at ../../../container-utils/rust)
RUN git clone --depth 1 https://forge.mms.name/emittiv/container-utils.git /container-utils

# Create a minimal workspace and override the container-utils path dependency
# to point to the cloned repo location.
RUN printf '[workspace]\nmembers = ["crates/e-fees-core", "e-fees-api"]\nresolver = "2"\n\n[workspace.dependencies]\nemittiv-container-utils = { path = "/container-utils/rust" }\n\n[patch.crates-io]\nemittiv-container-utils = { path = "/container-utils/rust" }\n' > Cargo.toml

# Copy the shared core crate
COPY crates/ crates/

# Copy the API crate
COPY e-fees-api/ e-fees-api/

# Build release binary
RUN cargo build --release -p e-fees-api

# Runtime stage
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates openssh-client && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/e-fees-api /usr/local/bin/

WORKDIR /data

EXPOSE 3200

CMD ["e-fees-api"]
