#!/bin/bash
# PF-84 #555 demo wrapper (pass as --bin to scripts/demo_video.py, with
# DEMO_CANDIDATE=<corbanu binary> exported). The home is configured for OpenAI
# with no credentials. It enrols $PF84_ACCOUNT_SEED as Z.AI account `work`, runs
# a session on Z.AI account work, then `corbanu resume` (the picker).
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
"$REAL" --account zai:work -m glm-5.3-flash -c 'model_provider="zai"' "$@"
clear
echo "Now: corbanu resume   (this home is configured for OpenAI, with no OpenAI credentials)"
sleep 2
exec "$REAL" resume "$@"
