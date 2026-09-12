FROM rust:slim AS builder

WORKDIR /app

# Install build deps (curl for utoipa-swagger-ui, git for container-utils)
RUN apt-get update && apt-get install -y curl git pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Clone container-utils from Forgejo (path dependency at ../../../container-utils/rust)
RUN git clone --depth 1 https://forge.mms.name/emittiv/container-utils.git /container-utils

# Minimal workspace (only scope + core)
# Create a minimal workspace.
RUN printf '[workspace]\nmembers = ["crates/e-fees-core", "e-fees-scope"]\nresolver = "2"\n' > Cargo.toml
COPY crates/ crates/

# Copy the scope crate and fix container-utils path for Docker context
COPY e-fees-scope/ e-fees-scope/
RUN sed -i 's|path = "../../../container-utils/rust"|path = "/container-utils/rust"|' e-fees-scope/Cargo.toml

# Pin the same dependency tree as local to keep surrealdb transitive deps stable.
# Build unlocked (minimal workspace has fewer members than the lock, so --locked
# would reject it); cargo still reuses the pinned versions for the deps it needs.
COPY Cargo.lock ./Cargo.lock

RUN cargo build --release -p e-fees-scope

# Runtime
FROM debian:trixie-slim

# PUID/PGID (Unraid best practice, docs.unraid.net "Managing and Customizing
# Containers"; same shape as linuxserver.io images, martin/pa PR #275,
# martin/anchor PR #16, and this repo's e-fees-api image): the container
# starts as root and docker-entrypoint.sh drops to PUID:PGID (default 99:100,
# nobody:users) via `gosu` before exec'ing the e-fees-scope binary. `gosu` is
# the only extra package needed; usermod/groupmod/chown/stat are already part
# of Debian's base (passwd + coreutils, Priority: required).
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates gosu \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r efeesscope && useradd -r -g efeesscope -s /usr/sbin/nologin efeesscope

COPY --from=builder /app/target/release/e-fees-scope /usr/local/bin/
COPY e-fees-scope/docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

EXPOSE 3201

# No USER line here (deliberate): the container must start as root so
# docker-entrypoint.sh can remap the efeesscope user/group to PUID/PGID and
# chown /config, then drop privileges itself via gosu before exec'ing the app
# - see docker-entrypoint.sh. Docker runs HEALTHCHECK (if one is ever added)
# as this image's USER, i.e. root - there is no HEALTHCHECK instruction
# today; the Unraid template's WebUI probe is external.
ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["e-fees-scope"]
