#!/bin/bash
# exec-workers.sh <home> <tag> [names...]: corbanu exec workers in one home (candidate unless BIN set).
# Each run gets the provider's REAL default key in its env, so any fallback to the
# default credential would succeed visibly instead of failing.
cd "$(dirname "$0")" && . ./env.sh
H=$1 TAG=$2 B=${BIN:-$CAND} OUT=$EV/captures/exec-$TAG.txt; shift 2
WANT=" ${*:-zai-default zai-work zai-fake zai-ghost kimi-default kimi-alt} "
: > "$OUT"
x() { # x <name> <label> <ENVVAR> args...
  local name=$1 label=$2 var=$3; shift 3
  [[ "$WANT" == *" $name "* ]] || return 0
  echo "=== $name: corbanu exec $* ===" >> "$OUT"
  key "$label" env CORBANU_HOME="$H" S="$S" python3 ./cleanexec.py "$var" -- "$B" exec --json --skip-git-repo-check \
    -C "$S/ws" "$@" "Reply with exactly the word pong and nothing else." > "$S/logs/$TAG-$name.jsonl" 2> "$S/logs/$TAG-$name.err"
  echo "exit=$?" >> "$OUT"
  python3 - "$S/logs/$TAG-$name.jsonl" "$S/logs/$TAG-$name.err" >> "$OUT" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    try: e = json.loads(line)
    except ValueError: continue
    t = e.get("type")
    if t == "thread.started": print("thread", e.get("thread_id"))
    elif t == "item.completed" and (e.get("item") or {}).get("type") == "agent_message": print("reply", repr(e["item"].get("text")))
    elif t == "turn.completed": print("usage", json.dumps(e.get("usage"), sort_keys=True))
    elif t in ("turn.failed", "error"): print(t, json.dumps(e)[:400])
err = open(sys.argv[2]).read().strip().splitlines()
print("stderr(last 4):", *[l[:300] for l in err[-4:]], sep="\n  ")
PY
}
x zai-default provider/zai_api_key ZAI_API_KEY -m glm-5.3-flash -c model_provider='"zai"'
x zai-work    provider/zai_api_key ZAI_API_KEY -m glm-5.3-flash -c model_provider='"zai"' --account work
x zai-fake    provider/zai_api_key ZAI_API_KEY -m glm-5.3-flash -c model_provider='"zai"' --account fake
x zai-ghost   provider/zai_api_key ZAI_API_KEY -m glm-5.3-flash -c model_provider='"zai"' --account ghost
x zp-default provider/zai_api_key ZAI_API_KEY -m glm-5.3-flash -c model_provider='"zproxy"'
x zp-work    provider/zai_api_key ZAI_API_KEY -m glm-5.3-flash -c model_provider='"zproxy"' --account work
x zp-fake    provider/zai_api_key ZAI_API_KEY -m glm-5.3-flash -c model_provider='"zproxy"' --account fake
x zp-ghost   provider/zai_api_key ZAI_API_KEY -m glm-5.3-flash -c model_provider='"zproxy"' --account ghost
x kimi-default provider/kimi_api_key KIMI_API_KEY -m kimi-k3 -c model_provider='"kimi-code"'
x kimi-alt     provider/kimi_api_key KIMI_API_KEY -m kimi-k3 -c model_provider='"kimi-code"' --account alt
sed "s#$S#\$S#g" "$OUT"
