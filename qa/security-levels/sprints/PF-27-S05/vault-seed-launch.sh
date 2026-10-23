#!/bin/bash
# PF-27-S05 demo helper (pass as --bin to scripts/demo_video.py, with
# PF27_CANDIDATE=<corbanu binary> exported). It stores $PF27_VAULT_SEED as the
# Z.AI provider key in the disposable profile's encrypted vault through the
# app-server API (`account/login/start`, providerApiKey), then runs the
# candidate with the variable removed, so the key exists only in the vault.
set -euo pipefail
# Keyring isolation: the seeding app-server and the candidate must never reach
# the real OS keyring (debug builds honour this variable).
[ -n "${CORBANU_TEST_NO_NATIVE_KEYRING:-}" ] || {
  echo "refusing: CORBANU_TEST_NO_NATIVE_KEYRING is not set" >&2; exit 1; }
: "${PF27_CANDIDATE:?export PF27_CANDIDATE=<corbanu binary>}"
case "${CODEX_HOME:-}" in
  */qa/demos/out/*) ;;
  *) echo "refusing: CODEX_HOME is not a disposable demo profile" >&2; exit 1 ;;
esac
[ "${CORBANU_HOME:-}" = "$CODEX_HOME" ] && [ "${PFTERMINAL_HOME:-}" = "$CODEX_HOME" ] || {
  echo "refusing: CORBANU_HOME and PFTERMINAL_HOME must equal CODEX_HOME" >&2; exit 1; }
: "${PF27_VAULT_SEED:?no seed credential}"
python3 - "$PF27_CANDIDATE" <<'PY'
import json, os, subprocess, sys

proc = subprocess.Popen(
    [sys.argv[1], "app-server"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True,
)

def send(message):
    proc.stdin.write(json.dumps(message) + "\n")
    proc.stdin.flush()

def reply(request_id):
    for line in proc.stdout:
        message = json.loads(line)
        if message.get("id") == request_id:
            return message
    raise SystemExit("app-server closed early")

send({"id": 1, "method": "initialize",
      "params": {"clientInfo": {"name": "pf27s05-seed", "version": "1"}}})
reply(1)
send({"method": "initialized"})
send({"id": 2, "method": "account/login/start",
      "params": {"type": "providerApiKey", "provider": "zai",
                 "apiKey": os.environ.pop("PF27_VAULT_SEED")}})
result = reply(2)
proc.stdin.close()
proc.wait(timeout=30)
if "error" in result:
    raise SystemExit(f"seeding failed: {result['error'].get('message')}")
PY
unset PF27_VAULT_SEED
exec "$PF27_CANDIDATE" "$@"
