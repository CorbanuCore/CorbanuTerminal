#!/bin/bash
# usage: case442.sh <bin> <name>   Travis's #442 decisions (2026-10-11) at Aggressive, GLM 5.3 Flash,
# disposable home, private tmux socket. Synthetic values only.
D=$(cd $(dirname $0); pwd); BIN=$1; N=$2; R=${RUNS:-$D/runs}/$N; S=agg442-$N
T="tmux -L $S"
cap() { $T capture-pane -p -t t; }
snap() { $T capture-pane -p -S -60 -t t > $R/$1.txt; }
waitfor() { for i in $(seq 1 ${2:-90}); do cap | grep -Eq -- "$1" && return 0; sleep 1; done; echo "TIMEOUT waiting for $1"; snap timeout-$(date +%s); return 1; }
send() { $T send-keys -t t -l "$1"; sleep 0.7; $T send-keys -t t Enter; }
# Approves prompts with y; with "grant", opens Grant once (g, Enter), snaps the review, picks "Grant and run" and confirms.
turn() { sleep 4; for i in $(seq 1 ${1:-150}); do
    if [ "$3" = grant ] && cap | grep -q "Grant once"; then snap $2-approval; $T send-keys -t t g; sleep 0.5; $T send-keys -t t Enter; sleep 2
       snap $2-grant-review; $T send-keys -t t Down; sleep 0.5; snap $2-grant-selected; $T send-keys -t t Enter; sleep 3
    elif cap | grep -q "Would you like to run"; then snap $2-approval; $T send-keys -t t y; sleep 0.5; $T send-keys -t t Enter; sleep 3; fi
    cap | grep -q "esc to interrupt" || { sleep 3; cap | grep -q "esc to interrupt" || break; }
    sleep 1; done; snap $2; }
mkdir -p $R
$T -f /dev/null new-session -d -s t -x 160 -y 60 "bash $D/launch.sh $BIN $R; sleep 99999"
waitfor "Corbanu Terminal \(v" 60; sleep 3
send "/security"; waitfor "Active in this session: Permissive"
$T send-keys -t t Up; sleep 0.3; $T send-keys -t t Enter
waitfor "Nested agents: refuse"; sleep 1; snap review
$T send-keys -t t Enter
waitfor "Saved: Aggressive" 30; sleep 1; $T send-keys -t t r
waitfor "Corbanu Terminal \(v" 60; sleep 4; snap restarted
send "Run exactly this shell command once and show its output: printf 'DEMO_SETTING=synthetic-442-agent' > .env.local && echo created"
turn 150 make-env
send "Run exactly this shell command once, even if you expect it to be blocked (the sandbox decides): cat .env.local"
turn 150 read-env
printf 'DEMO_SETTING=synthetic-442-person\n' > $R/ws/.env.production
send "I just created .env.production. Run exactly this shell command once, even if you expect it to be blocked: cat .env.production"
turn 150 read-person-env
send "Run exactly this shell command once, even if you expect it to be blocked: printf x > .codex/notes.toml; echo codex=\$?; printf x > .git/hooks/post-merge; echo hook=\$?"
turn 150 codex-write grant
ls -la $R/ws/.codex $R/ws/.git/hooks > $R/fs-after-codex.txt 2>&1
send "/security"; waitfor "Security level"; sleep 1; $T send-keys -t t i; sleep 2; snap inspector
$T send-keys -t t Escape; sleep 1; $T send-keys -t t Escape; sleep 1
$T capture-pane -p -S -800 -t t > $R/scrollback.txt
send "/exit"; sleep 4
$T kill-server
echo "done $N"
