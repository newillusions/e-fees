#!/usr/bin/env bash
# Vendor fp-template into the Docker build context at a pinned commit.
#
#   scripts/vendor-fp-template.sh [destination]
#
# Destination defaults to <repo>/.vendor/fp-template, which e-fees-api's
# Dockerfile COPYs to /opt/fp-template. Run this BEFORE `docker build`; the
# same script runs in CI, so a locally built image matches a CI built one.
#
# WHY A SCRIPT AND NOT A SUBMODULE OR AN IN-DOCKERFILE CLONE
#
#   - Submodule: fp-template is a SUBDIRECTORY of emittiv/gtm, and git cannot
#     submodule a subdirectory of another repository.
#   - Clone inside the Dockerfile: emittiv/gtm is private, so the clone needs a
#     credential, and a credential passed as a build ARG is recoverable from
#     the image history. It never enters an image this way.
#   - This script: one pinned commit in e-fees-api/fp-template.ref, one clone,
#     verified layout, used identically by CI and by a developer. The token
#     stays in the build environment and out of every layer.
#
# The upstream is emittiv/gtm's `fp-template/` and NOT the packaged
# emittiv/fp-template repository: the package reorganises the tree (no
# fill.py, engine/render.sh instead of render.sh) and the driver in
# e_fees_core::export::fp_render expects the gtm layout. gtm is also the
# editor-of-record for the template canon.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REF_FILE="${REPO_ROOT}/e-fees-api/fp-template.ref"
DEST="${1:-${REPO_ROOT}/.vendor/fp-template}"
UPSTREAM_HOST="forge.mms.name"
UPSTREAM_PATH="emittiv/gtm.git"
SUBDIR="fp-template"

[ -f "$REF_FILE" ] || { echo "ERROR: pinned ref file not found: $REF_FILE" >&2; exit 1; }
REF="$(grep -v '^[[:space:]]*#' "$REF_FILE" | tr -d '[:space:]' | head -c 64)"
case "$REF" in
  [0-9a-f]*) [ ${#REF} -eq 40 ] || { echo "ERROR: $REF_FILE must hold a full 40-character commit sha, got '${REF}'" >&2; exit 1; } ;;
  *) echo "ERROR: $REF_FILE must hold a full 40-character commit sha, got '${REF}'" >&2; exit 1 ;;
esac

# FORGEJO_TOKEN is how CI authenticates to the private gtm repo. Without it we
# fall back to whatever git credentials the caller already has (a developer's
# SSH key or credential helper), which is the normal local case.
if [ -n "${FORGEJO_TOKEN:-}" ]; then
  CLONE_URL="https://${FORGEJO_TOKEN}@${UPSTREAM_HOST}/${UPSTREAM_PATH}"
  SAFE_URL="https://***@${UPSTREAM_HOST}/${UPSTREAM_PATH}"
else
  CLONE_URL="ssh://git@ssh.${UPSTREAM_HOST}/${UPSTREAM_PATH}"
  SAFE_URL="$CLONE_URL"
fi

WORK="$(mktemp -d)"
# Never leave a clone (which may carry a tokenised remote URL) behind.
trap 'rm -rf "$WORK"' EXIT

echo "Vendoring ${SUBDIR} from ${SAFE_URL} at ${REF}"
git init --quiet "$WORK"
git -C "$WORK" remote add origin "$CLONE_URL"
git -C "$WORK" config core.sparseCheckout true
git -C "$WORK" sparse-checkout set --no-cone "/${SUBDIR}/*"
if ! git -C "$WORK" fetch --quiet --depth 1 origin "$REF"; then
  echo "ERROR: could not fetch ${REF} from ${SAFE_URL}." >&2
  echo "       In CI this needs a FORGEJO_TOKEN with read access to emittiv/gtm." >&2
  exit 1
fi
git -C "$WORK" checkout --quiet FETCH_HEAD

SRC="${WORK}/${SUBDIR}"
[ -d "$SRC" ] || { echo "ERROR: ${SUBDIR}/ is not present at ${REF}" >&2; exit 1; }

# Refuse to ship a tree the render driver cannot drive. FpRenderConfig::validate
# checks exactly these three, so a wrong layout fails here rather than as a 503
# from a deployed container.
for required in fill.py render.sh index.html tokens.css template.css engine; do
  [ -e "${SRC}/${required}" ] || { echo "ERROR: vendored tree is missing ${required}" >&2; exit 1; }
done

rm -rf "$DEST"
mkdir -p "$(dirname "$DEST")"
cp -R "$SRC" "$DEST"
rm -rf "$DEST/__pycache__" "$DEST"/engine/__pycache__ "$DEST/proposals" "$DEST/corpus" "$DEST/reference" "$DEST/eye-test"
printf '%s\n' "$REF" > "${DEST}/.vendored-from-gtm"

echo "Vendored to ${DEST} ($(find "$DEST" -type f | wc -l | tr -d ' ') files)"
