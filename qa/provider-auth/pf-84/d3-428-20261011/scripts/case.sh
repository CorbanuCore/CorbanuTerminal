#!/bin/bash
# usage: case.sh <bin> <name> <restart|unverified>   (GLM 5.3 Flash, disposable home, private tmux socket)
D=$(cd $(dirname $0); pwd); BIN=$1; N=$2; MODE=$3; R=$D/$N; S=pf84d3-$N
T="tmux -L $S"
cap() { $T capture-pane -p -t t; }
waitfor() { for i in $(seq 1 ${2:-90}); do cap | grep -q -- "$1" && return 0; sleep 1; done; echo "TIMEOUT waiting for $1"; return 1; }
send() { $T send-keys -t t -l "$1"; sleep 0.5; $T send-keys -t t Enter; }
EXTRA=""; [ $MODE = unverified ] && EXTRA="protected_mode_preflight = true"
$T -f /dev/null new-session -d -s t -x 150 -y 50 "MODE=$MODE EXTRA_FEATURES='$EXTRA' bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "Corbanu Terminal (v" 60
sleep 3
if [ $MODE = restart ]; then
  send "/security"; waitfor "Active in this session: Permissive"
  $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter
  waitfor "Core's level stays Permissive"; $T send-keys -t t Enter
  waitfor "Saved: Aggressive"; $T send-keys -t t r
  waitfor "Restarting Corbanu Terminal"; sleep 12
fi
send "/security"; waitfor "Active in this session: Aggressive"; sleep 1; $T send-keys -t t Escape; sleep 1
send "Use the spawn_agent tool exactly once with account set to work and the message 'Reply with the single word pong'. Then tell me the spawn result verbatim. Do not retry on another account."
for i in $(seq 1 150); do
  if cap | grep -q "Allow the spawned agent to run on account"; then echo "D3 QUESTION SHOWN"; sleep 1; $T send-keys -t t Down; sleep 0.3; $T send-keys -t t Enter; break; fi
  if cap | grep -q '"account":"work"'; then echo "D3 QUESTION NOT SHOWN: child spawned on work"; break; fi
  sleep 1
done
sleep 25
$T capture-pane -p -S -300 -t t > $D/$N.txt
$T kill-server
echo "== rollouts (core turn_context security_level / provider_account)"
for f in $(find $R/home/sessions -name "*.jsonl" | sort); do echo "$(basename $f): $(grep -o '"security_level":"[a-z]*"' $f | sort -u | tr '\n' ' ') $(grep -o '"provider_account":"[a-z]*"' $f | sort -u | tr '\n' ' ')"; done
