#!/bin/sh
# e-fees-api container entrypoint. Runs as root (see Dockerfile - no USER line) so it can
# do the PUID/PGID privilege drop below, then execs the real process as an unprivileged
# user.
set -e

# --- PUID/PGID privilege drop (Unraid best practice, same shape as linuxserver.io images -
# docs.unraid.net "Managing and Customizing Containers": "PUID=99 and PGID=100: These set
# the user and group IDs for file permissions"; same pattern as martin/pa PR #275 and
# martin/anchor PR #16). Defaults 99:100 are Unraid's own nobody:users. Remap the image's
# baked-in "efeesapi" user/group to the requested ids (`-o` allows sharing an id with a
# pre-existing system account, e.g. Debian's own "users" group at gid 100), chown the dirs
# the app actually writes to, then drop from root via gosu.
PUID="${PUID:-99}"
PGID="${PGID:-100}"

if [ "$PUID" = "0" ]; then
    echo "entrypoint: PUID=0 - running as root (explicitly requested)" >&2
fi

current_uid="$(id -u efeesapi)"
current_gid="$(id -g efeesapi)"

if [ "$current_gid" != "$PGID" ]; then
    groupmod -o -g "$PGID" efeesapi
fi
if [ "$current_uid" != "$PUID" ]; then
    usermod -o -u "$PUID" efeesapi
fi

# Chown the writable mount to the runtime id. /config (the "Appdata" template mount) is the
# sole rw Unraid mount for this image; /config/app.env is a SEPARATE, READ-ONLY bind mount
# nested inside it (the "Application Config" template mount, hot-reloadable) - skip it by
# name so chown -R never treats it as part of the writable tree. Skip the whole tree when
# ownership already matches (cheap top-level stat), so an unchanged-PUID restart doesn't
# re-walk it. No `find` dependency: debian:trixie-slim's "slim" variant does not guarantee
# findutils is installed, so this loops over shell globs instead.
chown_skip_app_env() {
    dir="$1"
    [ -d "$dir" ] || return 0
    owner="$(stat -c '%u:%g' "$dir")"
    [ "$owner" = "${PUID}:${PGID}" ] && return 0
    chown "${PUID}:${PGID}" "$dir"
    for entry in "$dir"/* "$dir"/.[!.]* "$dir"/..?*; do
        [ -e "$entry" ] || continue
        [ "$(basename "$entry")" = "app.env" ] && continue
        chown -R "${PUID}:${PGID}" "$entry"
    done
}

chown_skip_app_env /config

# /root/.ssh (the optional "SSH Keys" template mount, default host path
# /mnt/user/appdata/e-fees-api/ssh) is READ-ONLY and typically host-root-owned - it can
# never be chowned to the runtime PUID/PGID, and once privileges drop below, the Nextcloud
# folder-export pipeline's `ssh` subprocess (src/ssh.rs, invoked from routes/fee_export.rs
# and routes/rescan.rs) could no longer open a uid-0-owned mode-600 private key. Copy it out
# to a PUID/PGID-owned scratch dir instead - same pattern as martin/pa PR #275's
# copy-ssh-key.sh. No-op if no key is mounted (NC_SSH_HOST and the SSH Keys mount are both
# optional - most deployments don't use the Nextcloud pipeline). EFEES_SSH_DIR must match
# src/config.rs's NC_SSH_KEY fallback default.
EFEES_SSH_DIR=/run/efees-ssh
copy-ssh-key.sh /root/.ssh/id_ed25519 "$EFEES_SSH_DIR" "${PUID}" "${PGID}"

exec gosu "${PUID}:${PGID}" "$@"
