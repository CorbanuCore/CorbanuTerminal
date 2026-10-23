#!/bin/bash
# usage: case.sh <bin> <name>
# #507 (owner decision 2026-10-12): turning the kill switch off restarts nothing. A /spawn crew with a native
# Corbanu (GLM 5.3 Flash) Troll and a Claude Code (Opus 5.5 Claude Plan) Orc; tasks are sent to both while the
# switch is on; the switch goes off; nothing starts; then a person types in each pane and the waiting task goes
# with that turn.
D=$(cd $(dirname $0); pwd); BIN=$1; N=$2; R=${RUNS:-$D/runs}/$N; S=ks507-$N
T="tmux -L $S"
. $D/lib.sh
mkdir -p $R
$T -f /dev/null new-session -d -s t -x 160 -y 60 "bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "Corbanu Terminal \(v" 60; sleep 3
aggressive
# Crew: Nazgul (native), Troll Burzum (native GLM), Orc Snaga (Claude Code, Claude Plan).
send "/spawn"; waitfor "Quick start"; down 2; key Enter; waitfor "Create Nazgul pane"; key Enter
waitfor "Select Model and Effort"; key Enter; waitfor "Select Reasoning"; key Up; key Up; key Enter
waitfor "bound it as the Nazgul root" 30
send "/spawn"; waitfor "Quick start"; down 3; key Enter; waitfor "Choose the harness"; key Enter
waitfor "Select Model and Effort"; key Enter; waitfor "Select Reasoning"; key Up; key Up; key Enter
waitfor "Spawned Corbanu Terminal Troll pane" 30
send "/spawn"; waitfor "Quick start"; down 4; key Enter; waitfor "Assign Orc Supervisor"; key Enter
waitfor "Choose the harness"; key Down; key Enter; waitfor "Spawn Claude Orc"; down 6; key Enter
waitfor "as a background worker" 30; sleep 1
# Switch on (capture its review).
send "/security"; waitfor "Security level"; sleep 1; key g; waitfor "Grants and kill switch"; sleep 0.5
key Down; key Enter; waitfor "Turn the kill switch on\?"; sleep 1; snap 01-kill-on-review; key Enter
waitfor "Kill switch on" 30; sleep 1; close
send_task() { # <search> <task>
  send "/spawn status"; waitfor "Spawn Status"; sleep 1; $T send-keys -t t -l "$1"; sleep 1; key Enter
  waitfor "Send Task to" 10; sleep 0.5; send "$2"; sleep 3; }
send_task "send task to burzum" "Create a file named codex-task.txt in the workspace containing the single line pf507-codex-task."
send_task "send task to claude" "Reply with exactly the word pf507-claude-task and nothing else."
snap 02-tasks-wait
# Switch off (capture its review), then wait: nothing may start.
send "/security"; waitfor "Security level"; sleep 1; key g; waitfor "Grants and kill switch"; sleep 0.5
key Down; key Enter; waitfor "Turn the kill switch off\?"; sleep 1; snap 03-kill-off-review; key Down; key Enter
waitfor "Kill switch off" 30; sleep 1; close; sleep 1; snap 04-switch-off
sleep 45
send "/spawn status"; waitfor "Spawn Status"; sleep 1; snap 05-status-45s-after-off; key Escape; sleep 0.5
{ echo "45 s after switch-off:"; ls $R/ws; ls $R/home/panes/*/ 2>/dev/null | grep -c "turn-.*\.jsonl" | sed 's/^/Claude turn files: /'; } > $R/05-files-45s-after-off.txt
# A person types in the Troll pane: the waiting task goes with that turn.
send "/panes"; sleep 2; $T send-keys -t t -l "burzum"; sleep 1; snap 06-select-troll; key Enter; sleep 2
send "Go ahead."; waitfor "Worked for|─ Worked|tok/s" 5; sleep 60; snap 06-troll-typed
{ echo "after typing in the Troll pane:"; ls $R/ws; cat $R/ws/codex-task.txt 2>/dev/null; } > $R/06-files-after-troll.txt
# A person types in the Claude pane: the waiting task goes with that turn.
send "/spawn status"; waitfor "Spawn Status"; sleep 1; $T send-keys -t t -l "claude-"; sleep 1
snap 07-select-claude; key Enter; sleep 2
send "Hello."; sleep 60; snap 07-claude-typed
$T capture-pane -p -S -1000 -t t > $R/scrollback.txt
send "/exit"; sleep 4
$T kill-server
echo "done $N"
