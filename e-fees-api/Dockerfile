FROM rust:1.90-slim AS builder

WORKDIR /app

# Install build deps (curl for utoipa-swagger-ui, git for container-utils)
RUN apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*

# Clone container-utils from Forgejo (path dependency at ../../../container-utils/rust)
RUN git clone --depth 1 https://forge.mms.name/emittiv/container-utils.git /container-utils

# Create a minimal workspace.
RUN printf '[workspace]\nmembers = ["crates/e-fees-core", "e-fees-api"]\nresolver = "2"\n' > Cargo.toml

# Copy the shared core crate
COPY crates/ crates/

# Copy the API crate and fix container-utils path for Docker context
COPY e-fees-api/ e-fees-api/
RUN sed -i 's|path = "../../../container-utils/rust"|path = "/container-utils/rust"|' e-fees-api/Cargo.toml

# Copy the workspace lockfile so the build resolves the SAME (tested) dependency
# tree as local — notably keeping surrealdb's transitive deps pinned and
# diskann-free. Without this, a fresh resolve pulls diskann 0.52.0, which fails
# to compile under the builder's rustc. Build unlocked (the regenerated minimal
# workspace manifest lists fewer members than the lock, so --locked would reject
# it); cargo still reuses the pinned versions for the deps it needs.
COPY Cargo.lock ./Cargo.lock

# Build release binary
RUN cargo build --release -p e-fees-api

# Runtime stage
FROM debian:trixie-slim

# NO python3, NO fp-template checkout, NO browser in this image (dropped
# 2026-09-06 with the docbuilder client). Proposal rendering in production
# goes to the remote fp-docbuilder service via DOCBUILDER_URL
# (see crates/e-fees-core/src/export/docbuilder.rs): that service owns
# fill.py, the three pre-issue gates, and the renderer end to end, so this
# host needs none of fp-template's own toolchain.
#
# The local fill+gate+render pipeline (e_fees_core::export::fp_render,
# FP_TEMPLATE_ROOT, GOTENBERG_URL) is unchanged in the code and still exercised
# by tests, but is DEV-ONLY as of this image: without python3 or a vendored
# fp-template checkout here, FP_TEMPLATE_ROOT can never be set on this
# container, so `select_proposal_backend` always resolves to the docbuilder
# path when DOCBUILDER_URL is set, and to "not configured" (503) otherwise.
# See docs/development/FP-TEMPLATE-INTEGRATION.md.
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates openssh-client \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/e-fees-api /usr/local/bin/

WORKDIR /data

EXPOSE 3200

CMD ["e-fees-api"]
