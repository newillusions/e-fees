FROM rust:1.85-slim AS builder

WORKDIR /app

# Copy workspace-level files
COPY Cargo.toml Cargo.lock ./

# Copy the shared core crate
COPY crates/ crates/

# Copy the API crate
COPY e-fees-api/ e-fees-api/

# Create a dummy src-tauri to satisfy workspace member (won't be built)
RUN mkdir -p src-tauri/src && echo 'fn main() {}' > src-tauri/src/main.rs
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml

# Build release binary
RUN cargo build --release -p e-fees-api

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/e-fees-api /usr/local/bin/

EXPOSE 3200

CMD ["e-fees-api"]
