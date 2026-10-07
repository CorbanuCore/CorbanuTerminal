#!/bin/bash
# Demo wrapper: stores an unknown level in the disposable profile before launch.
# Keyring isolation: never let a demo candidate reach the real OS keyring.
[ -n "${CORBANU_TEST_NO_NATIVE_KEYRING:-}" ] || {
  echo "refusing: CORBANU_TEST_NO_NATIVE_KEYRING is not set" >&2; exit 1; }
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
case "$REAL" in */release/*)
  echo "refusing: a release candidate ignores CORBANU_TEST_NO_NATIVE_KEYRING" >&2; exit 1;; esac
printf 'version = 1\nlevel = "max"\n' > "$CODEX_HOME/security_level.toml"
exec "$REAL" "$@"
