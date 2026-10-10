#!/bin/bash
# local. usage: keyscan.sh: count files containing any of the four vault values (value never printed) in the remote scratch
# (homes, logs, captures, the scratch workspace) and in this evidence directory, plus key-shaped strings in both. Each value
# is resolved by the installed wrapper on the consuming command and fed to grep -F -f on stdin; an empty value aborts
# (an empty pattern file would give a false 0).
RTX=${RTX:-<rtx-user>@<rtx-host>}; D=$(cd "$(dirname "$0")/.." && pwd)
V(){ env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper "$1" < /dev/null; }
for lab in openai-api-key provider/zai_api_key provider/kimi_api_key claude-plan-test-token; do
  [ "$(V "$lab" | wc -c)" -gt 20 ] || { echo "$lab: value empty or short; scan aborted"; exit 1; }
  r=$(V "$lab" | ssh -o BatchMode=yes $RTX 'cd ~/corbanu-rtx/pf60s04-ia && grep -rlF -f - homes logs cap work 2>/dev/null' | wc -l | tr -d ' ')
  l=$(grep -rlF -f <(V "$lab") "$D" 2>/dev/null | wc -l | tr -d ' ')
  echo "$lab (value non-empty): remote scratch $r files; evidence dir $l files"
done
KS='sk-[A-Za-z0-9_-]{20,}|Bearer [A-Za-z0-9._-]{20,}'
echo "key-shaped (sk-..., sk-ant-..., Bearer <20+>) in evidence dir: $(grep -rlE "$KS" "$D" 2>/dev/null | wc -l | tr -d ' ') files"
echo "key-shaped in remote scratch (homes, logs, cap, work): $(ssh -o BatchMode=yes $RTX "cd ~/corbanu-rtx/pf60s04-ia && grep -rlE '$KS' homes logs cap work 2>/dev/null" | wc -l | tr -d ' ') files"
