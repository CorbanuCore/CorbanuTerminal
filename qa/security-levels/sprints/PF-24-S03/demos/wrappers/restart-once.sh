#!/bin/bash
# Demo wrapper: runs the candidate, then starts it again with the same
# arguments and profile so a video can show a real restart.
# Keyring isolation: never let a demo candidate reach the real OS keyring.
[ -n "${CORBANU_TEST_NO_NATIVE_KEYRING:-}" ] || {
  echo "refusing: CORBANU_TEST_NO_NATIVE_KEYRING is not set" >&2; exit 1; }
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
case "$REAL" in */release/*)
  echo "refusing: a release candidate ignores CORBANU_TEST_NO_NATIVE_KEYRING" >&2; exit 1;; esac
"$REAL" "$@"
printf '\n[demo] restarting Corbanu Terminal with the same profile...\n'
sleep 1
exec "$REAL" "$@"
