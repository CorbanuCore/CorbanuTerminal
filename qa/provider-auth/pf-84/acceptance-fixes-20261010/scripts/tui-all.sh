#!/bin/bash
# TUI checks for #416, #417, #419.1 (before/after). Drives tmux; never prints keys.
S=<scratch>
. $S/tmuxlib.sh
$T has-session -t main 2>/dev/null || $T new-session -d -s main -x 200 -y 50
P="Reply with exactly the word pong and nothing else."
for v in before after; do
  start n417$v $v $S/hn 0 -c "provider_accounts.zai='\"main\"'"; sleep 1
  waitfor main:n417$v 'Run /review|API key|›' 40
  if cap main:n417$v | grep -q 'Use your Z.AI API key'; then :; else say main:n417$v "$P"; waitfor main:n417$v '^. *pong|■' 90; fi
  cap main:n417$v > $S/cap-417-$v.txt
  start w416$v $v $S/hz 1 -c "provider_accounts.zai='\"fake\"'"; waitfor main:w416$v 'Run /review|›' 40
  say main:w416$v "$P"; waitfor main:w416$v 'rejected|■' 90; cap main:w416$v > $S/cap-416-$v.txt
  say main:w416$v "/providers"; waitfor main:w416$v 'r recover credentials' 30; cap main:w416$v > $S/cap-416-providers-$v.txt
  start w419$v $v $S/hw 1 -c "provider_accounts.zai='\"x\"'"; waitfor main:w419$v 'Run /review|›' 40; sleep 3; cap main:w419$v > $S/cap-419-warning-$v.txt
done
echo TUIDONE > $S/tui-all.done
