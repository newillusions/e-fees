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

# python3 + pyyaml + beautifulsoup4 are fp-template's own toolchain: fill.py
# builds the proposal HTML, and issue-check/column-check/quote-check are the
# three pre-issue gates. All four are pure HTML/DOM python and need no browser.
#
# There is deliberately NO Chromium here. PDF rendering goes to the gotenberg
# service named by GOTENBERG_URL (see e_fees_core::export::gotenberg), which is
# the whole point of this image: no 400 MB browser, no per-container Chromium
# to patch. The consequence is named in FP-TEMPLATE-INTEGRATION.md rather than
# hidden: fp-template's engine/measure.py still launches a local Chromium via
# Playwright during the FILL step, so until that gains a remote backend the
# render route in THIS image answers 422 with the fill failure. The manifest
# route, and every other route, is unaffected.
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates openssh-client \
      python3 python3-yaml python3-bs4 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/e-fees-api /usr/local/bin/

# fp-template, vendored at the commit pinned in e-fees-api/fp-template.ref.
# The build context must already carry it: run scripts/vendor-fp-template.sh
# first (CI does this in .forgejo/workflows/build-containers.yml). A missing
# directory fails the build here, which is the intended loud failure - an image
# without the template would 503 on every proposal render for no visible reason.
COPY .vendor/fp-template /opt/fp-template
ENV FP_TEMPLATE_ROOT=/opt/fp-template

WORKDIR /data

EXPOSE 3200

CMD ["e-fees-api"]
