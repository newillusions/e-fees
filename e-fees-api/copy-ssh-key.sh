#!/bin/sh
# copy-ssh-key.sh SRC_KEY DEST_DIR PUID PGID
#
# Copies an SSH private key (and its known_hosts sibling, if present) out of a
# read-only mount into a PUID:PGID-owned scratch directory, so a dropped-
# privilege process can still read it.
#
# WHY THIS EXISTS: /root/.ssh is a SEPARATE, READ-ONLY bind mount (default host
# path /mnt/user/appdata/e-fees-api/ssh, per templates/e-fees-api.xml's "SSH Keys"
# entry) used by the Nextcloud folder-export pipeline (src/ssh.rs). Its private
# key is, by near-universal OpenSSH convention, mode 600 owned by uid 0 - and
# OpenSSH itself refuses a group/other-readable private key, so loosening the
# mounted file's permissions is not an option either way. Because the mount is
# read-only, docker-entrypoint.sh's chown_skip_app_env() only ever touches
# /config (a different mount entirely) - /root/.ssh's ownership never changes to
# match the new runtime PUID/PGID. Once the container drops privileges via gosu,
# the dropped-privilege `ssh` subprocess can no longer open a uid-0-owned 600
# file it doesn't own. Copying it out to a directory THIS container's runtime
# user owns is the standard workaround (same pattern as martin/pa PR #275's
# copy-ssh-key.sh for pa-core's Nextcloud SFTP key).
#
# Kept as a separate, parameterized script (not inlined in docker-entrypoint.sh)
# so this logic is unit-testable without invoking the real entrypoint or
# running as root - see copy-ssh-key.test.sh.
#
# No-op (exit 0, does not create DEST_DIR) when SRC_KEY doesn't exist - the
# Nextcloud SSH feature is optional; most deployments don't mount a key.
set -eu

SRC_KEY="${1:?usage: copy-ssh-key.sh SRC_KEY DEST_DIR PUID PGID}"
DEST_DIR="${2:?usage: copy-ssh-key.sh SRC_KEY DEST_DIR PUID PGID}"
PUID="${3:?usage: copy-ssh-key.sh SRC_KEY DEST_DIR PUID PGID}"
PGID="${4:?usage: copy-ssh-key.sh SRC_KEY DEST_DIR PUID PGID}"

if [ ! -f "$SRC_KEY" ]; then
    exit 0
fi

mkdir -p "$DEST_DIR"
chmod 700 "$DEST_DIR"

DEST_KEY="$DEST_DIR/$(basename "$SRC_KEY")"
cp "$SRC_KEY" "$DEST_KEY"
chmod 600 "$DEST_KEY"

SRC_DIR="$(dirname "$SRC_KEY")"
if [ -f "$SRC_DIR/known_hosts" ]; then
    cp "$SRC_DIR/known_hosts" "$DEST_DIR/known_hosts"
    chmod 644 "$DEST_DIR/known_hosts"
fi

chown -R "${PUID}:${PGID}" "$DEST_DIR"
