#!/bin/bash
# lease.sh <home> <tag>: per-account request lease on built-in zai through the TLS-intercepting
# mock (synthetic canaries only). Account a's reply is held for 12 s. While it is in flight we
# read provider_request_state and start (1) a default-account request, (2) an account-b request,
# (3) a second account-a request, and record when each reaches the mock.
cd "$(dirname "$0")" && . ./env.sh && . ./tmuxlib.sh
H=$1 TAG=$2 B=${BIN:-$CAND} PORT=${PORT:-18465} OUT=$EV/captures/lease-$TAG.txt
CD=qa-s05-canary-default-7c1e CA=qa-s05-canary-acct-a-91b2 CB=qa-s05-canary-acct-b-5d03
MLOG=$S/logs/lease-$TAG.jsonl; : > "$MLOG"
tx new-session -d -s "lease-$TAG" "cd $PWD && MOCK_LABELS=default=$CD,a=$CA,b=$CB MOCK_SLOW=a:12 python3 mitmmock.py $PORT $MLOG $S/certs"
sleep 1; mkdir -p "$H"
printf 'model = "glm-5.3-flash"\nmodel_provider = "zai"\n\n[features]\nnamed_accounts = %s\n' "${FLAG:-true}" > "$H/config.toml"
SECRET=$CA run_secret_stdin "$B" "$H" account add zai a >/dev/null 2>&1
SECRET=$CB run_secret_stdin "$B" "$H" account add zai b >/dev/null 2>&1
export HTTPS_PROXY=http://127.0.0.1:$PORT SSL_CERT_FILE=$S/certs/ca.pem
T0=$(python3 -c 'import time;print(time.time())')
x() { local name=$1; shift
  SECRET=$CD env CORBANU_HOME="$H" S="$S" timeout 90 python3 ./cleanexec.py ZAI_API_KEY -- "$B" exec --skip-git-repo-check -C "$S/ws" "$@" "pong" \
    > "$S/logs/lease-$TAG-$name.out" 2>&1
  echo "$name exit=$? done_at=+$(python3 -c "import time;print(round(time.time()-$T0,1))")s" >> "$S/logs/lease-$TAG-done.txt"; }
: > "$S/logs/lease-$TAG-done.txt"
echo "# $TAG: $B home=${H//$S/\$S}; account a's reply is held 12 s by the mock" > "$OUT"
x a1 --account a & sleep 4
echo "--- provider_request_state at +4s (a1 in flight) ---" >> "$OUT"
sqlite3 -header "$H"/pfterminal_state_*.sqlite "select key_fingerprint, case when lease_owner is null then '-' else 'held' end as lease, case when lease_until_ms > strftime('%s','now')*1000 then 'until +' || ((lease_until_ms - strftime('%s','now')*1000)/1000) || 's' else '-' end as lease_until from provider_request_state order by key_fingerprint" >> "$OUT"
x d1 & x b1 --account b & x a2 --account a &
wait
cat "$S/logs/lease-$TAG-done.txt" >> "$OUT"
echo "--- mock arrivals (seconds after start; label, status) ---" >> "$OUT"
python3 - "$MLOG" "$T0" >> "$OUT" <<'PY'
import json, sys
t0 = float(sys.argv[2])
for r in map(json.loads, open(sys.argv[1])):
    if r.get("path", "").endswith("/chat/completions"):
        print(f"  +{(r.get('arrived') or r['ts']) - t0:5.1f}s {r['label']:<8} {r['status']}")
PY
tx kill-session -t "lease-$TAG"
cat "$OUT"
