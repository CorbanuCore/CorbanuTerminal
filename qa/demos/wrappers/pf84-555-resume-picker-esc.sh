#!/bin/bash
# PF-84 #555 demo wrapper: a home with no default Z.AI key. A session on
# --account work records a conversation; then `corbanu resume` opens the picker,
# where the demo presses Esc.
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
echo "Now: corbanu resume   (no default Z.AI key in this home)"
sleep 2
status=0
"$REAL" resume "$@" || status=$?
echo "corbanu resume exited with status $status"
sleep 5
