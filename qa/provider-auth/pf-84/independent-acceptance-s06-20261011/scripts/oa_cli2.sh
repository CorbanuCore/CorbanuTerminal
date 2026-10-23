#!/usr/bin/env bash
# Slice B, second pass with realistic token lifetimes (sign-in tokens already expired, refreshed tokens live 1 h):
# exact refresh counts per process, same-account concurrency across processes, re-login (other user / same user),
# forced_login_method=chatgpt on an API-key account, default precedence vs the pre-S06 baseline.
# Output: captures/oa-cli2.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"
P=34622; U=http://127.0.0.1:$P; LOG=$S/logs/mock-b2.jsonl; : > "$LOG"
python3 "$D/mock.py" --port $P --log "$LOG" --bearer keyacct=sk-canary-keyacct-S06CANARY \
  --bearer defkey=sk-canary-defaultkey-S06CANARY --bearer envkey=sk-envdecoy-S06CANARY --bearer codexenv=sk-codexenv-S06CANARY &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
export FH=$S/fh-b3 H=$S/homes/b3
[ -d $H ] && rm -r $H $FH < /dev/null; mkdir -p $H $FH
cat > $H/config.toml <<EOF
model = "gpt-5.4"
model_provider = "openai"
openai_base_url = "$U/v1"
chatgpt_base_url = "$U/backend-api/codex"
[features]
named_accounts = true
[projects."$S/ws"]
trust_level = "trusted"
EOF
OUT=$EV/captures/oa-cli2.txt
ctl() { curl -s -X POST -H 'content-type: application/json' -d "$1" $U/_ctl > /dev/null; }
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
login() { ctl "{\"next_user\":\"$2\"}"; show run $CAND $H login --account $1 --device-auth --experimental_issuer $U < /dev/null | grep -v -E 'Welcome|command-line|Follow these|Open this|codex/device|one-time code|MOCK-0001|Continue only|^\s*$'; }
ex() { show run ${BIN:-$CAND} $H exec --skip-git-repo-check -C $S/ws "$@" "Reply with exactly: pong" < /dev/null | grep -v -E 'rmcp::transport|^(workdir|approval|sandbox|reasoning|session id|user$|Reply with exactly|warning: Under-dev|tokens used|Reading additional|model:|provider:|corbanu$|129|-{8})|^\s*$'; }
since() { tail -n +$(($1+1)) $LOG | python3 -c "import sys,json
for l in sys.stdin:
  e=json.loads(l); e.pop('t')
  if e['ev'] in ('post-unhandled',) or (e['ev']=='get' and e.get('who',{}).get('scheme')=='none'): continue
  print('   ', json.dumps(e))"; }
cnt() { local b=$(mocklines $LOG); ex "$@"; echo "-- mock events (MCP/plugin noise without credentials omitted):"; since $b; }
authsha() { echo "auth.json: $( [ -f $H/auth.json ] && shasum -a 256 $H/auth.json | cut -c1-16 || echo absent)"; }
{
echo "## home b3; default login = API key (fake canary) via login --with-api-key"
printf 'sk-canary-defaultkey-S06CANARY' | show run $CAND $H login --with-api-key
authsha
ctl '{"ttl":-60,"refresh_ttl":3600}'
login r1 r1; login r2 r2
printf 'sk-canary-keyacct-S06CANARY' | show run $CAND $H login --account key --with-api-key
echo; echo "## one process: exactly one refresh, saved back"
cnt --account r1
echo "## a new process: no refresh (reads the refreshed token from the vault)"
cnt --account r1
echo; echo "## same account (r2, expired) in two processes at once"
b=$(mocklines $LOG)
( ex --account r2 > $S/logs/b3-r2a.txt ) & w1=$!; ( ex --account r2 > $S/logs/b3-r2b.txt ) & w2=$!; wait $w1 $w2
echo "process A:"; grep -E '^mock:|ERROR|\[exit' $S/logs/b3-r2a.txt | sort -u
echo "process B:"; grep -E '^mock:|ERROR|\[exit' $S/logs/b3-r2b.txt | sort -u
since $b
echo "## r2 again afterwards"
cnt --account r2
echo; echo "## re-login r1 as ANOTHER ChatGPT user (r1b): old r1 tokens revoked?"
b=$(mocklines $LOG); login r1 r1b; since $b
show run $CAND $H login --account r1 status
cnt --account r1
echo "## re-login r1 as the SAME user (r1b~2 = same person as r1b)"
b=$(mocklines $LOG); login r1 "r1b~2"; since $b
authsha
echo; echo "## forced_login_method=chatgpt with the API-key account"
show run $CAND $H account list openai
cnt --account key -c 'forced_login_method="chatgpt"'
echo "after:"; show run $CAND $H account list openai; authsha
show run $CAND $H login status
echo "## the same with the TUI-less app-server path is not exercised; default under forced chatgpt:"
cnt -c 'forced_login_method="chatgpt"'
authsha
show run $CAND $H login status
echo; echo "## default precedence with env decoys: candidate vs pre-S06 baseline (copy of the home)"
printf 'sk-canary-defaultkey-S06CANARY' | run $CAND $H login --with-api-key > /dev/null 2>&1; authsha
cp -R $H $S/homes/b3-base
XENV+=(OPENAI_API_KEY=sk-envdecoy-S06CANARY)
cnt
H=$S/homes/b3-base BIN=$BASE cnt
XENV+=(CODEX_API_KEY=sk-codexenv-S06CANARY)
cnt
H=$S/homes/b3-base BIN=$BASE cnt
echo "named account with the same env decoys:"
cnt --account r1
} > "$OUT" 2>&1
echo done
