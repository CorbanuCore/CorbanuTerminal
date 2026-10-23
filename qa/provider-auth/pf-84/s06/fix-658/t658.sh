#!/bin/bash
# #658 tmux check: a TUI started on named ChatGPT account `work` (mock user tuiwork) in a home whose default
# login is mock user tuidefault. Usage: t658.sh <label> <corbanu-binary>. Fake tokens only; disposable home.
set -u
L=$1 BIN=$2
S=${QA_SCRATCH:?set QA_SCRATCH to a scratch directory}
SCR=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)/qa/provider-auth/pf-84/independent-acceptance-s06b-20261013/scripts
P=34781 U=http://127.0.0.1:34781
H=$S/homes/t658-$L FH=$S/fh-$L LOG=$S/logs/mock-$L.jsonl SOCK=$S/tmux-$L.sock
mkdir -p $S/logs $S/ws $FH; [ -d $H ] && rm -r $H < /dev/null; mkdir -p $H; : > $LOG
python3 $SCR/mock.py --port $P --log $LOG > $S/logs/mock-$L.out 2>&1 &
MOCK=$!; trap 'kill $MOCK 2>/dev/null' EXIT; sleep 1
cat > $H/config.toml <<EOT
model = "gpt-5.4"
model_provider = "openai"
approval_policy = "never"
sandbox_mode = "read-only"
openai_base_url = "$U/v1"
chatgpt_base_url = "$U/backend-api/codex"
suppress_unstable_features_warning = true
[features]
named_accounts = ${NAMED:-true}
[projects."$S/ws"]
trust_level = "trusted"
EOT
run() { env -i HOME=$FH USER=$USER TERM=xterm-256color LANG=en_US.UTF-8 PATH=/opt/homebrew/bin:/usr/bin:/bin \
  CORBANU_HOME=$H CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U "$BIN" "$@"; }
ctl() { curl -s -X POST -H 'content-type: application/json' -d "{\"next_user\":\"$1\",\"ttl\":3600,\"refresh_ttl\":3600}" $U/_ctl >/dev/null; }
ctl tuidefault; run login --device-auth --experimental_issuer $U < /dev/null 2>&1 | grep -iE "success|logged|error" | head -2
[ "${NAMED:-true}" = true ] && ctl tuiwork && run login --account work --device-auth --experimental_issuer $U < /dev/null 2>&1 | grep -iE "success|logged|signed|error" | head -2
[ -n "${KEY_ACCOUNT:-}" ] && printf 'sk-fake-658-apikey' | run login --account $KEY_ACCOUNT --with-api-key 2>&1 | tail -1
[ "${NAMED:-true}" = true ] && run account list 2>&1 | head -6
if [ -n "${RESUME:-}" ]; then
  run exec --skip-git-repo-check -C $S/ws --account work "Reply with exactly: pong" < /dev/null > $S/logs/exec-$L.txt 2>&1
  RID=$(sed -n 's/^session id: //p' $S/logs/exec-$L.txt | head -1); echo "recorded $RID on work"
  ACCOUNT_ARGS="resume $RID"
fi
shasum -a 256 $H/auth.json > $S/logs/auth-before-$L.sha256
MARK=$(wc -l < $LOG)
T="tmux -S $SOCK"
CMD=$(printf '%q ' env -i HOME=$FH USER=$USER TERM=xterm-256color LANG=en_US.UTF-8 PATH=/opt/homebrew/bin:/usr/bin:/bin \
  CORBANU_HOME=$H CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U \
  "$BIN" -C $S/ws -c 'model_provider="openai"' -m gpt-5.4 ${ACCOUNT_ARGS---account work})
$T new-session -d -s t -x 160 -y 50 "$CMD; echo EXITED-\$?; sleep 600"
wait_for() { local i prev="" cur; for ((i=0;i<${2:-60}*2;i++)); do cur=$($T capture-pane -p -t t -S -80); grep -Eq -- "$1" <<<"$cur" && [ "$cur" = "$prev" ] && return 0; prev=$cur; sleep 0.5; done; echo "TIMEOUT /$1/"; return 1; }
send() { $T send-keys -t t -l -- "$1"; sleep 0.8; $T send-keys -t t Enter; }
wait_for 'Corbanu Terminal \(v' 60; sleep 3
send "Reply with exactly: pong"; wait_for 'mock: ' 90; sleep 2
send "/status"; wait_for 'Session:' 30; sleep 2
send "/plugins"; sleep 4; $T send-keys -t t Escape; sleep 1
send "/apps"; sleep 4; $T send-keys -t t Escape; sleep 1
$T capture-pane -p -t t -S -200 | grep -v '^\s*$' | sed "s#$S#\$S#g" > $S/logs/screen-$L.txt
send "/exit"; wait_for 'EXITED-' 30; sleep 1
$T kill-server 2>/dev/null
echo "== $L: backend/model requests during the TUI run, by path and identity"
tail -n +$((MARK+1)) $LOG | python3 -c '
import sys,json,collections
c=collections.Counter()
for l in sys.stdin:
    e=json.loads(l); w=e.get("who") or {}
    who=w.get("user") or w.get("canary") or w.get("scheme","-")
    c[(e.get("ev"), e.get("path","-"), who)]+=1
for (ev,p,who),n in sorted(c.items(), key=lambda x:(x[0][1],x[0][2])): print("%4d %-14s %-60s %s"%(n,ev,p,who))
'
echo "auth.json unchanged: $(shasum -a 256 -c $S/logs/auth-before-$L.sha256 2>&1 | tail -1)"
