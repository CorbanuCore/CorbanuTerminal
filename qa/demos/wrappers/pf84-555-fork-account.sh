#!/bin/bash
# PF-84 #555 demo wrapper (pass as --bin to scripts/demo_video.py, with
# DEMO_CANDIDATE=<corbanu binary> exported). It enrols $PF84_ACCOUNT_SEED as the
# named Z.AI accounts `work` and `personal` ([provider_accounts] picks
# `personal`), then runs two sessions in the same pane:
#   1. `corbanu --account work`, which records the conversation on `work`;
#   2. `corbanu resume --last`, where the demo forks it.
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
clear
echo "Now: corbanu resume --last   (ran on work; [provider_accounts] zai = \"personal\")"
sleep 2
exec "$REAL" resume --last "$@"
