#!/bin/sh
# e-fees-scope container entrypoint. Runs as root (see Dockerfile - no USER line) so it can
# do the PUID/PGID privilege drop below, then execs the real process as an unprivileged
# user.
set -e

# --- PUID/PGID privilege drop (Unraid best practice, same shape as linuxserver.io images -
# docs.unraid.net "Managing and Customizing Containers"; same pattern as martin/pa PR #275,
# martin/anchor PR #16, and this repo's e-fees-api image). Defaults 99:100 are Unraid's own
# nobody:users. Remap the image's baked-in "efeesscope" user/group to the requested ids
# (`-o` allows sharing an id with a pre-existing system account, e.g. Debian's own "users"
# group at gid 100), chown the dirs the app actually writes to, then drop from root via
# gosu.
PUID="${PUID:-99}"
PGID="${PGID:-100}"

if [ "$PUID" = "0" ]; then
    echo "entrypoint: PUID=0 - running as root (explicitly requested)" >&2
fi

current_uid="$(id -u efeesscope)"
current_gid="$(id -g efeesscope)"

if [ "$current_gid" != "$PGID" ]; then
    groupmod -o -g "$PGID" efeesscope
fi
if [ "$current_uid" != "$PUID" ]; then
    usermod -o -u "$PUID" efeesscope
fi

# Chown the writable mount to the runtime id. /config (the "Appdata" template mount) is the
# sole rw Unraid mount for this image; /config/app.env is a SEPARATE, READ-ONLY bind mount
# nested inside it (the "Application Config" template mount, hot-reloadable) - skip it by
# name so chown -R never treats it as part of the writable tree. /data/rfps (the "RFP Data"
# template mount, CORPUS_PATH) and /app/source (the "Source" template mount, unused at
# runtime) are both read-only and are never chowned. Skip the whole /config tree when
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

exec gosu "${PUID}:${PGID}" "$@"
