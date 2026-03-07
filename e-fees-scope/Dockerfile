FROM rust:1.89-slim AS builder

WORKDIR /app

# Minimal workspace (only scope + core)
RUN printf '[workspace]\nmembers = ["crates/e-fees-core", "e-fees-scope"]\nresolver = "2"\n' > Cargo.toml

COPY Cargo.lock .
COPY crates/ crates/
COPY e-fees-scope/ e-fees-scope/

# curl needed by utoipa-swagger-ui build script
RUN apt-get update && apt-get install -y curl pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

RUN cargo build --release -p e-fees-scope

# Runtime
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/e-fees-scope /usr/local/bin/

EXPOSE 3201

CMD ["e-fees-scope"]
