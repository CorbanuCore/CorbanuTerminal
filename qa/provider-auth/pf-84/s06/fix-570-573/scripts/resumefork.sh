#!/usr/bin/env bash
# #570 review: `corbanu resume --last` / `corbanu fork --last` of a thread recorded on named account `chat`
# (no --account) while forced_login_method="chatgpt" rules out the default API key. usage: resumefork.sh <BASE|CAND>
set -u
source ~/corbanu-rtx/f57x-qa/env.sh
B=$1; bin=${!B}
P=34744; export U=http://127.0.0.1:$P; LOG=$S/logs/mock-rf.jsonl; : > "$LOG"
python3 $Q/mock.py --port $P --log "$LOG" --bearer defkey=sk-canary-defaultkey-570 &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
export FH=$S/fh-rf; mkdir -p $FH
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
H=$S/homes/rf$B; [ -d $H ] && rm -r $H < /dev/null; mkdir -p $H
printf 'model = "gpt-5.4"\nmodel_provider = "openai"\nopenai_base_url = "%s/v1"\nchatgpt_base_url = "%s/backend-api/codex"\napproval_policy = "never"\nsuppress_unstable_features_warning = true\n[features]\nnamed_accounts = true\n[projects."%s/ws"]\ntrust_level = "trusted"\n' $U $U $S > $H/config.toml
printf 'sk-canary-defaultkey-570' | run $BASE $H login --with-api-key > /dev/null 2>&1
ctl '{"next_user":"chat","ttl":3600}'; run $BASE $H login --account chat --device-auth --experimental_issuer $U < /dev/null > /dev/null 2>&1
OUT=$S/logs/tui-resumefork-$B.txt
turn() { local s=$1 b=$(mocklines $LOG); send_line $s "Reply with exactly: pong"; sleep 12
  echo "-- mock lines for this turn:"; tail -n +$((b+1)) $LOG | grep -E '"responses' | sed 's/^/   /'; }
hash() { [ -f $H/auth.json ] && echo "auth.json $(sha256sum $H/auth.json | cut -c1-12)" || echo "auth.json ABSENT (logged out)"; }
{
echo "######## resume/fork of a named thread on $B ($(basename $bin)); forced_login_method=chatgpt added after the first session"
echo "-- 1. first session on account chat (no restriction yet)"
start_tui rf1 $bin $H --account chat; sleep 12; turn rf1; send_line rf1 /exit; sleep 3; tx kill-session -t rf1 2>/dev/null
sed -i '1i forced_login_method = "chatgpt"' $H/config.toml
echo "default login before resume: $(hash)"
echo "-- 2. corbanu resume --last (no --account)"
start_tui rf2 $bin $H resume --last; sleep 14
cap rf2 -60 | grep -v '^\s*$' | grep -E 'Sign in|ChatGPT login|Logging out|restricted|resum|account' | head -5 | cut -c1-160
turn rf2; echo "default login after resume: $(hash)"
send_line rf2 /status; sleep 3; cap rf2 -70 | grep -E 'Model provider|Account:' | sed 's/^ *//' | cut -c1-120
tx send-keys -t rf2 Escape; sleep 0.5; send_line rf2 /exit; sleep 3; tx kill-session -t rf2 2>/dev/null
echo "-- 3. corbanu fork --last --account chat"
start_tui rf3 $bin $H fork --last --account chat; sleep 14
cap rf3 -60 | grep -v '^\s*$' | grep -E 'Sign in|ChatGPT login|Logging out|fork|account' | head -5 | cut -c1-160
turn rf3; echo "default login after fork --account chat: $(hash)"
send_line rf3 /exit; sleep 3; tx kill-session -t rf3 2>/dev/null
echo "-- 4. corbanu fork --last (no --account): documented limit, default sign-in screen expected"
start_tui rf4 $bin $H fork --last; sleep 14
cap rf4 -40 | grep -v '^\s*$' | tail -6 | cut -c1-160
echo "default login after fork (no --account): $(hash)"
tx send-keys -t rf4 C-c; sleep 1; tx send-keys -t rf4 C-c; sleep 2; tx kill-session -t rf4 2>/dev/null
} > "$OUT" 2>&1
cat "$OUT"
