#!/bin/bash
# PF-84 #609 demo wrapper (pass as --bin to scripts/demo_video.py, with
# DEMO_CANDIDATE=<corbanu binary> exported). It enrols $PF84_ACCOUNT_SEED as
# the named Z.AI account `work` and starts the session on it, with no default
# Z.AI key.
set -euo pipefail
[ -n "${CORBANU_TEST_NO_NATIVE_KEYRING:-}" ] || {
  echo "refusing: CORBANU_TEST_NO_NATIVE_KEYRING is not set" >&2; exit 1; }
REAL="${DEMO_CANDIDATE:?set DEMO_CANDIDATE to the candidate corbanu binary}"
case "${CODEX_HOME:-}" in
  */qa/demos/out/*) ;;
  *) echo "refusing: CODEX_HOME is not a disposable demo profile" >&2; exit 1 ;;
esac
: "${PF84_ACCOUNT_SEED:?no seed credential}"
printf %s "$PF84_ACCOUNT_SEED" | "$REAL" account add zai work >/dev/null
unset PF84_ACCOUNT_SEED ZAI_API_KEY
exec "$REAL" --account work "$@"
