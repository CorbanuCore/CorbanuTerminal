#!/bin/bash
# PF-84 #555 demo wrapper: [provider_accounts] zai = "ghost" names an account
# that does not exist. A session on --account work records the conversation,
# then `corbanu resume --last` resumes it.
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
echo "Now: corbanu resume --last   ([provider_accounts] zai = \"ghost\" does not exist)"
sleep 2
exec "$REAL" resume --last "$@"
