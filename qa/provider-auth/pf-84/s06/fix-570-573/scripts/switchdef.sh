#!/usr/bin/env bash
# #570: a TUI session on named ChatGPT account `chat` (forced_login_method="chatgpt", default login = API key)
# tries "Use default account for this session": refused with the reason; auth.json unchanged. usage: switchdef.sh <BASE|CAND>
set -u
source ~/corbanu-rtx/f57x-qa/env.sh
B=$1; bin=${!B}; s=sd$B
P=34743; export U=http://127.0.0.1:$P; LOG=$S/logs/mock-sd.jsonl; : > "$LOG"
python3 $Q/mock.py --port $P --log "$LOG" --bearer defkey=sk-canary-defaultkey-570 &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
export FH=$S/fh-sd; mkdir -p $FH
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U)
H=$S/homes/sd$B; [ -d $H ] && rm -r $H < /dev/null; mkdir -p $H
printf 'model = "gpt-5.4"\nmodel_provider = "openai"\nopenai_base_url = "%s/v1"\nchatgpt_base_url = "%s/backend-api/codex"\napproval_policy = "never"\nsuppress_unstable_features_warning = true\n[features]\nnamed_accounts = true\n[projects."%s/ws"]\ntrust_level = "trusted"\n' $U $U $S > $H/config.toml
printf 'sk-canary-defaultkey-570' | run $BASE $H login --with-api-key > /dev/null 2>&1
ctl '{"next_user":"named","ttl":3600}'; run $BASE $H login --account chat --device-auth --experimental_issuer $U < /dev/null > /dev/null 2>&1
sed -i '1i forced_login_method = "chatgpt"' $H/config.toml
run $BASE $H account list
OUT=$S/logs/tui-switchdef-$B.txt
{
echo "######## switch to default on $B: TUI on named \`chat\`, forced_login_method=chatgpt, default login = API key"
before=$(sha256sum $H/auth.json | cut -c1-12); echo "default login before: auth.json $before"
start_tui $s $bin $H --account chat; sleep 14
cap $s -60 | grep -v '^\s*$' | grep -E 'Logging out|ChatGPT login|restricted' | head -3
b=$(mocklines $LOG); send_line $s "Reply with exactly: pong"; sleep 12
echo "-- turn on chat:"; tail -n +$((b+1)) $LOG | grep -E '"responses' | sed 's/^/   /'
send_line $s "/providers"; sleep 4; pick $s 'OpenAI · default'; sleep 2; pick $s 'Use default account' 10; sleep 8; tx send-keys -t $s Escape; sleep 1
echo "-- after Use default account for this session:"; cap $s -40 | grep -v '^\s*$' | grep -E '^(■|•|⚠ (Th|Lo|AP|Ch))' -A2 | grep -v MCP | tail -8 | cut -c1-200
if [ -f $H/auth.json ]; then a=$(sha256sum $H/auth.json | cut -c1-12); echo "default login after: auth.json $a $([ $a = $before ] && echo '(unchanged)' || echo '(CHANGED)')"; else echo "default login after: auth.json ABSENT (logged out)"; fi
echo "-- mock requests carrying the default key: $(grep -c defkey $LOG)"
tx send-keys -t $s Escape; sleep 0.5; send_line $s "/exit"; sleep 3; tx kill-session -t $s 2>/dev/null
} > "$OUT" 2>&1
cat "$OUT"
