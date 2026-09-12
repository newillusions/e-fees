#!/usr/bin/env bash
# copy-ssh-key.test.sh - fixture-based unit tests for copy-ssh-key.sh.
# Run: bash e-fees-api/copy-ssh-key.test.sh
#
# No vitest/bats harness for shell scripts in this repo; mirrors the
# self-contained bash test runner used by martin/pa's copy-ssh-key.test.sh
# (PR #275, pa-core-rs/copy-ssh-key.test.sh) - identical script, adapted only
# in its HERE-relative path.
#
# Runs unprivileged: chown is called with the TEST PROCESS's own uid:gid
# (always permitted for a non-root process to "chown" to itself), so the
# script's real chown call path is exercised without needing root.
set -euo pipefail

# shellcheck disable=SC1007  # false positive on the CDPATH-reset idiom - see docker-entrypoint.sh
HERE="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
COPY_SSH_KEY="$HERE/copy-ssh-key.sh"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

OWN_UID="$(id -u)"
OWN_GID="$(id -g)"

fail=0

case_missing_source_is_noop() {
	local name="missing-source-is-noop"
	local src="$TMP_ROOT/$name/src/id_ed25519"
	local dest="$TMP_ROOT/$name/dest"

	out=$("$COPY_SSH_KEY" "$src" "$dest" "$OWN_UID" "$OWN_GID" 2>&1)
	rc=$?
	if [ "$rc" -eq 0 ] && [ ! -d "$dest" ]; then
		echo "PASS: $name"
	else
		echo "FAIL: $name -- rc=$rc dest_exists=$([ -d "$dest" ] && echo yes || echo no) output: $out"
		fail=1
	fi
}

case_copies_key_with_correct_perms() {
	local name="copies-key-with-correct-perms"
	local src_dir="$TMP_ROOT/$name/src"
	local src="$src_dir/id_ed25519"
	local dest="$TMP_ROOT/$name/dest"
	mkdir -p "$src_dir"
	echo "fake-private-key-material" >"$src"
	chmod 600 "$src"

	out=$("$COPY_SSH_KEY" "$src" "$dest" "$OWN_UID" "$OWN_GID" 2>&1)
	dest_key="$dest/id_ed25519"
	dest_mode=$(stat -f '%Lp' "$dest_key" 2>/dev/null || stat -c '%a' "$dest_key" 2>/dev/null)
	dest_dir_mode=$(stat -f '%Lp' "$dest" 2>/dev/null || stat -c '%a' "$dest" 2>/dev/null)
	if [ -f "$dest_key" ] && [ "$(cat "$dest_key")" = "fake-private-key-material" ] \
		&& [ "$dest_mode" = "600" ] && [ "$dest_dir_mode" = "700" ]; then
		echo "PASS: $name"
	else
		echo "FAIL: $name -- key_mode=$dest_mode dir_mode=$dest_dir_mode output: $out"
		fail=1
	fi
}

case_copies_known_hosts_when_present() {
	local name="copies-known-hosts-when-present"
	local src_dir="$TMP_ROOT/$name/src"
	local src="$src_dir/id_ed25519"
	local dest="$TMP_ROOT/$name/dest"
	mkdir -p "$src_dir"
	echo "fake-private-key-material" >"$src"
	echo "10.0.20.12 ssh-ed25519 AAAAfakehostkey" >"$src_dir/known_hosts"

	out=$("$COPY_SSH_KEY" "$src" "$dest" "$OWN_UID" "$OWN_GID" 2>&1)
	if [ -f "$dest/known_hosts" ] && grep -q "fakehostkey" "$dest/known_hosts"; then
		echo "PASS: $name"
	else
		echo "FAIL: $name -- output: $out"
		fail=1
	fi
}

case_no_known_hosts_does_not_error() {
	local name="no-known-hosts-does-not-error"
	local src_dir="$TMP_ROOT/$name/src"
	local src="$src_dir/id_ed25519"
	local dest="$TMP_ROOT/$name/dest"
	mkdir -p "$src_dir"
	echo "fake-private-key-material" >"$src"

	out=$("$COPY_SSH_KEY" "$src" "$dest" "$OWN_UID" "$OWN_GID" 2>&1)
	rc=$?
	if [ "$rc" -eq 0 ] && [ -f "$dest/id_ed25519" ] && [ ! -f "$dest/known_hosts" ]; then
		echo "PASS: $name"
	else
		echo "FAIL: $name -- rc=$rc output: $out"
		fail=1
	fi
}

case_missing_source_is_noop
case_copies_key_with_correct_perms
case_copies_known_hosts_when_present
case_no_known_hosts_does_not_error

if [ "$fail" -ne 0 ]; then
	echo "copy-ssh-key.test.sh: FAILURES PRESENT"
	exit 1
fi
echo "copy-ssh-key.test.sh: all cases passed"
