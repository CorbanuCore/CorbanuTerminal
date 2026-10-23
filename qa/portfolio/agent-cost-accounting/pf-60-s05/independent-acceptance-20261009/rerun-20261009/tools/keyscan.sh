#!/bin/bash
# usage: keyscan.sh <path>... : count files containing any of the three vault values (value never printed) or a
# key-shaped string. Values are fed to grep through a process substitution on the consuming command.
V(){ env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper "$1" < /dev/null; }
for lab in provider/kimi_api_key claude-plan-test-token openai-api-key; do
  n=$(grep -rlF -f <(V "$lab") "$@" 2>/dev/null | wc -l | tr -d ' '); echo "$lab: $n files"
done
echo "key-shaped (sk-…, sk-ant-…, Bearer <20+>): $(grep -rlE 'sk-[A-Za-z0-9_-]{20,}|Bearer [A-Za-z0-9._-]{20,}' "$@" 2>/dev/null | wc -l | tr -d ' ') files"
