#!/bin/bash
# PF-84 #545 demo wrapper (pass as --bin to scripts/demo_video.py, with
# DEMO_CANDIDATE=<corbanu binary> exported). It enrols $PF84_ACCOUNT_SEED as
# the named Z.AI account `work`, then runs two sessions in the same pane:
#   1. `corbanu --account work`, which records the conversation on `work`;
#   2. `corbanu resume --last` without --account.
# No ZAI_API_KEY is ever set, so the home has no default Z.AI key.
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
"$REAL" --account work "$@"
clear
echo "Now: corbanu resume --last   (no --account; this home has no default Z.AI key)"
sleep 2
exec "$REAL" resume --last "$@"
