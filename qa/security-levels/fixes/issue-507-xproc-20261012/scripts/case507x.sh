#!/bin/bash
# usage: case507x.sh <bin> <name>
# #507 (Travis, 2026-10-12): two Corbanu Terminal processes on ONE disposable home. Process B runs a slow command;
# process A turns the kill switch on in /security; B's command must stop within the stated latency (about 1 s, at
# most 2 s) and B must refuse new turns; A turns it off and B answers again. GLM 5.3 Flash.
D=$(cd $(dirname $0); pwd); BIN=$1; N=$2; R=${RUNS:-$D/runs}/$N
A="tmux -L agg507x-$N-a"; B="tmux -L agg507x-$N-b"
cap() { $1 capture-pane -p -t t; }
snap() { $1 capture-pane -p -S -80 -t t > $R/$2.txt; }
waitfor() { local t=$1; shift; for i in $(seq 1 ${2:-90}); do cap "$t" | grep -Eq -- "$1" && return 0; sleep 1; done; echo "TIMEOUT waiting for $1"; return 1; }
send() { $1 send-keys -t t -l "$2"; sleep 0.7; $1 send-keys -t t Enter; }
close() { for i in 1 2 3 4; do cap "$A" | grep -Eq "Grants and kill switch|Security level|Kill switch o(n|ff) in" || return 0; $A send-keys -t t Escape; sleep 1; done; }
idle() { sleep 4; for i in $(seq 1 ${2:-120}); do cap "$1" | grep -q "esc to interrupt" || return 0; sleep 1; done; }
now() { python3 -c 'import time; print(f"{time.time():.2f}")'; }
mkdir -p $R
# One home, two processes (B's launch reuses A's home: launch.sh only adds folders and rewrites config.toml).
$A -f /dev/null new-session -d -s t -x 160 -y 60 "bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "$A" "Corbanu Terminal \(v" 60; sleep 2
$B -f /dev/null new-session -d -s t -x 160 -y 60 "bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "$B" "Corbanu Terminal \(v" 60; sleep 3
{ echo "A pid: $(pgrep -f "agg507x-$N-a" | head -1)"; echo "processes on the home:"; ps -ax -o pid,command | grep -F -- "$BIN" | grep -v grep | sed "s|$R|<run>|g"; } > $R/processes.txt
# B: a slow command (Permissive in B; it started before A saved Aggressive).
send "$B" "Run exactly this shell command once, unchanged, and wait for it to finish: touch cmd-started.txt && sleep 45 && touch ran-after-kill-switch.txt && echo FINISHED"
# The command itself is running once it has made its first file.
for i in $(seq 1 120); do [ -e $R/ws/cmd-started.txt ] && break; sleep 1; done
[ -e $R/ws/cmd-started.txt ] && echo "B's command is running" > $R/started.txt || { echo "B's command never started" > $R/started.txt; }
sleep 2; snap "$B" b-running
# A: confirm Aggressive (no restart), then the kill switch in its grants view.
send "$A" "/security"; waitfor "$A" "Active in this session: Permissive"
$A send-keys -t t Down; sleep 0.3; $A send-keys -t t Down; sleep 0.3; $A send-keys -t t Enter
waitfor "$A" "Switch to Aggressive"; sleep 1; $A send-keys -t t Enter
waitfor "$A" "Saved: Aggressive" 30; sleep 1; $A send-keys -t t Escape; sleep 1
send "$A" "/security"; waitfor "$A" "Security level"; sleep 1
$A send-keys -t t g; waitfor "$A" "Grants and kill switch"; sleep 0.5
$A send-keys -t t Down; sleep 0.3; $A send-keys -t t Enter
waitfor "$A" "Turn the kill switch on\?"; sleep 1; snap "$A" a-kill-on-review
T0=$(now); $A send-keys -t t Enter
waitfor "$A" "Kill switch on" 30; snap "$A" a-kill-on-done
# B: poll every 0.1 s for the stop.
T1=""; for i in $(seq 1 100); do if cap "$B" | grep -q "Stopped: the security kill switch"; then T1=$(now); break; fi; sleep 0.1; done
snap "$B" b-after-switch
python3 -c "import sys; t0=float('$T0'); t1='$T1'; print(f'confirm->B stop message: {float(t1)-t0:.2f} s (includes A writing the file)' if t1 else 'B: NO STOP MESSAGE within 10 s')" > $R/latency.txt
close
sleep 45; { [ -e $R/ws/ran-after-kill-switch.txt ] && echo "MARKER: written (B's command ran)" || echo "MARKER: absent (B's command was stopped)"; } > $R/marker.txt
send "$B" "What is 2+2?"; idle "$B" 60; sleep 2; snap "$B" b-turn-kill-on
# A: turn it off.
send "$A" "/security"; waitfor "$A" "Security level"; sleep 1
$A send-keys -t t g; waitfor "$A" "Grants and kill switch"; sleep 0.5
$A send-keys -t t Down; sleep 0.3; $A send-keys -t t Enter
waitfor "$A" "Turn the kill switch off\?"; sleep 0.5; $A send-keys -t t Down; sleep 0.3; snap "$A" a-kill-off-review
T0=$(now); $A send-keys -t t Enter; waitfor "$A" "Kill switch off" 30
T1=""; for i in $(seq 1 100); do if cap "$B" | grep -q "turned off in another Corbanu Terminal"; then T1=$(now); break; fi; sleep 0.1; done
python3 -c "t0=float('$T0'); t1='$T1'; print(f'off confirm->B notice: {float(t1)-t0:.2f} s' if t1 else 'B: NO OFF NOTICE within 10 s')" >> $R/latency.txt
snap "$B" b-after-off; close
send "$B" "What is 3+3?"; idle "$B" 120; sleep 2; snap "$B" b-turn-kill-off
$B capture-pane -p -S -600 -t t > $R/b-scrollback.txt; $A capture-pane -p -S -600 -t t > $R/a-scrollback.txt
send "$A" "/exit"; send "$B" "/exit"; sleep 4
$A kill-server; $B kill-server
cat $R/started.txt $R/latency.txt $R/marker.txt
echo "done $N"
