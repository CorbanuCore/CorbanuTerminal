#!/bin/bash
# Reopen the coordinator's conversation in a tall window and capture the whole /cost first screen.
. "$(dirname "${BASH_SOURCE[0]}")/env.sh"
C=$S/captures
COLS=170 ROWS=80 start cost resume "${1:?session id}"
waitfor cost 'Corbanu Terminal' 60; sleep 3
send cost "/cost"; waitfor cost 'account fake' 90; sleep 2
T capture-pane -p -J -t cost > "$C/tui-cost-full.txt"
: > "$C/tui-cost-scrolled.txt"
for i in $(seq 1 16); do T send-keys -t cost Down; sleep 0.5; echo "--- after Down x$i" >> "$C/tui-cost-scrolled.txt"; T capture-pane -p -J -t cost | awk '/Cost — this conversation/,0' >> "$C/tui-cost-scrolled.txt"; done
T send-keys -t cost Escape; sleep 1; send cost "/exit"; sleep 3; T kill-server 2>/dev/null
