# A missing [provider_accounts] selection: refused for a fresh session, but a resume runs on the recorded account.
. "$(dirname "$0")/tui-lib.sh"
H=$S/homes/hT-after
DEF=$(ls -t $(find $H/sessions -name 'rollout-*.jsonl') | while read f; do grep -q '"provider_account":"default"' "$f" && basename "$f" && break; done | sed -E 's/^rollout-[0-9T:-]+-([0-9a-f-]{36})\.jsonl$/\1/')
echo "default-recorded session: $DEF"
T kill-server 2>/dev/null
echo '== T7 fresh: corbanu -c provider_accounts.zai="ghost"'; start t7 $H -c 'provider_accounts.zai="ghost"'; sleep 6; snap t7
echo '== T8 resume: corbanu -c provider_accounts.zai="ghost" resume <id> (refused up front; onboarding cannot use the recorded account)'; start t8 $H -c 'provider_accounts.zai="ghost"' resume $DEF; sleep 6; snap t8
echo '== T9 recovery: same with --account default, send pong prompt'; start t9 $H -c 'provider_accounts.zai="ghost"' resume --account default $DEF; sleep 8; snap t9 >/dev/null
send t9 "Reply with exactly the word pong and nothing else."; sleep 12; snap t9 | grep -E -i 'pong|rejected|not configured|exited' | tail -3
send t9 "/exit"; sleep 2; T kill-server 2>/dev/null
