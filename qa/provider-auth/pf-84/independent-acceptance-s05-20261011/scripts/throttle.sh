#!/bin/bash
# throttle.sh <home> <tag> : per-account 429 cooldown with the mock provider.
# Synthetic canary keys only (no real credential): default=env D, accounts a and b.
# The mock answers 429 (Retry-After 120 s) to b's key and "pong" to every other key.
# VIA=mitm (default): built-in `zai` reaches the mock through HTTPS_PROXY + a throwaway CA
# (SSL_CERT_FILE), nothing leaves the host. VIA=custom: a custom `mock` provider instead.
cd "$(dirname "$0")" && . ./env.sh && . ./tmuxlib.sh
H=$1 TAG=$2 B=${BIN:-$CAND} PORT=${PORT:-18452} OUT=$EV/captures/throttle-$TAG.txt
CD=qa-s05-canary-default-7c1e CA=qa-s05-canary-acct-a-91b2 CB=qa-s05-canary-acct-b-5d03
MLOG=$S/logs/mock-$TAG.jsonl; : > "$MLOG"
tx kill-session -t "mock-$TAG" 2>/dev/null
VIA=${VIA:-mitm}
if [ "$VIA" = mitm ]; then
  tx new-session -d -s "mock-$TAG" "cd $PWD && MOCK_LABELS=default=$CD,a=$CA,b=$CB MOCK_429=$CB MOCK_RETRY_AFTER=120 python3 mitmmock.py $PORT $MLOG $S/certs"
  PROV=zai MODEL=glm-5.3-flash VAR=ZAI_API_KEY; export HTTPS_PROXY=http://127.0.0.1:$PORT SSL_CERT_FILE=$S/certs/ca.pem
else
  tx new-session -d -s "mock-$TAG" "MOCK_LABELS=default=$CD,a=$CA,b=$CB MOCK_429=$CB MOCK_RETRY_AFTER=120 python3 $PWD/usageproxy.py mock $PORT $MLOG"
  PROV=mock MODEL=mock-model VAR=MOCK_API_KEY
fi
sleep 1
mkdir -p "$H"
cat > "$H/config.toml" <<EOF
model = "$MODEL"
model_provider = "$PROV"
approval_policy = "never"
sandbox_mode = "read-only"

[features]
named_accounts = ${FLAG:-true}

[model_providers.mock]
name = "QA mock (429 for account b)"
base_url = "http://127.0.0.1:$PORT/v1"
env_key = "MOCK_API_KEY"
wire_api = "chat"
EOF
SECRET=$CA run_secret_stdin "$B" "$H" account add $PROV a >/dev/null 2>&1
SECRET=$CB run_secret_stdin "$B" "$H" account add $PROV b >/dev/null 2>&1
{ echo "# $TAG: $B home=${H//$S/\$S} via=$VIA provider=$PROV"; run "$B" "$H" account list 2>&1 | grep -v WARNING; } > "$OUT"
x() { # x <name> [--account n]
  local name=$1; shift
  local t0; t0=$(python3 -c 'import time;print(time.time())')
  SECRET=$CD env CORBANU_HOME="$H" S="$S" timeout 60 python3 ./cleanexec.py $VAR -- "$B" exec --skip-git-repo-check -C "$S/ws" \
    "$@" "Reply with exactly the word pong and nothing else." > "$S/logs/$TAG-$name.out" 2> "$S/logs/$TAG-$name.err"
  local rc=$?
  local dt; dt=$(python3 -c "import time;print(round(time.time()-$t0,1))")
  local hits; hits=$(python3 -c "import json,collections,sys;print(dict(collections.Counter((r['label'],r['status']) for r in map(json.loads,open('$MLOG')) if r.get('path','').endswith('/chat/completions'))))")
  echo "=== $name: exec $* -> exit=$rc in ${dt}s; reply=$(tr -d '\n' < "$S/logs/$TAG-$name.out" | cut -c1-40)" >> "$OUT"
  echo "    mock requests so far (label,status): $hits" >> "$OUT"
  grep -Eiv 'call_metrics|Reading additional|WARNING: proceeding' "$S/logs/$TAG-$name.err" | tail -3 | cut -c1-260 | sed 's/^/    stderr: /' >> "$OUT"
}
x b1 --account b
x a1 --account a
x d1
x b2 --account b
x a2 --account a
echo "--- provider_request_state ---" >> "$OUT"
sqlite3 -header "$H"/pfterminal_state_*.sqlite \
  "select provider_id, model, key_fingerprint, case when cooldown_until_ms > strftime('%s','now')*1000 then 'ACTIVE ~' || ((cooldown_until_ms - strftime('%s','now')*1000)/1000) || 's left' else 'none' end as cooldown, last_status, consecutive_429_count from provider_request_state order by key_fingerprint;" >> "$OUT" 2>&1
tx kill-session -t "mock-$TAG"
sed "s#$S#\$S#g" "$OUT"
