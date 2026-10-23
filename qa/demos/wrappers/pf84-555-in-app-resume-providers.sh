#!/bin/bash
# PF-84 #555 demo wrapper: [provider_accounts] zai = "personal". A session on
# --account work records a conversation, then account work is removed and a
# new session starts on personal, where the demo uses /resume.
set -euo pipefail
[ -n "${CORBANU_TEST_NO_NATIVE_KEYRING:-}" ] || {
  echo "refusing: CORBANU_TEST_NO_NATIVE_KEYRING is not set" >&2; exit 1; }
REAL="${DEMO_CANDIDATE:?set DEMO_CANDIDATE to the candidate corbanu binary}"
case "${CODEX_HOME:-}" in
  */qa/demos/out/*) ;;
  *) echo "refusing: CODEX_HOME is not a disposable demo profile" >&2; exit 1 ;;
esac
: "${PF84_ACCOUNT_SEED:?no seed credential}"
for name in work personal; do
  printf %s "$PF84_ACCOUNT_SEED" | "$REAL" account add zai "$name" >/dev/null
done
unset PF84_ACCOUNT_SEED ZAI_API_KEY
"$REAL" --account work "$@"
"$REAL" account remove zai work >/dev/null
clear
echo "Now: corbanu   (account work was removed; this session runs on personal)"
sleep 2
exec "$REAL" "$@"
