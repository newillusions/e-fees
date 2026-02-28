FROM rust:1.89-slim AS builder

WORKDIR /app

# Create a minimal workspace with only the crates needed for the API build.
# This avoids pulling in src-tauri and its complex dependencies (tauri, tauri-plugin-mcp, etc.).
RUN printf '[workspace]\nmembers = ["crates/e-fees-core", "e-fees-api"]\nresolver = "2"\n' > Cargo.toml

# Copy the shared core crate
COPY crates/ crates/

# Copy the API crate
COPY e-fees-api/ e-fees-api/

# Build release binary
RUN cargo build --release -p e-fees-api

# Runtime stage
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/e-fees-api /usr/local/bin/

EXPOSE 3200

CMD ["e-fees-api"]
