. "$(dirname "$0")/tui-lib.sh"
H=$S/homes/hT-after
T kill-server 2>/dev/null
echo "== T5 positive: corbanu --account fake (configured) reaches the chat"; start t5 $H --account fake; sleep 8; snap t5 | grep -E 'Corbanu Terminal|via zai|exited' | head -4
send t5 "/exit"; sleep 2
echo "== T6 positive: corbanu (default account) reaches the chat, ping answered"; start t6 $H; sleep 8; snap t6 >/dev/null
send t6 "Reply with exactly the word pong and nothing else."; sleep 12; snap t6 | grep -E 'pong|via zai|rejected' | head -4
send t6 "/exit"; sleep 2
T kill-server 2>/dev/null
