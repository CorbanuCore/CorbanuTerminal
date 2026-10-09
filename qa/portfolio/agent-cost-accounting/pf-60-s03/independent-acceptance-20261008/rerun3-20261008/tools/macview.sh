#!/bin/bash
# usage: macview.sh <sess> <launcher...>: start a TUI in tmux (socket pf60rerun3), accept the trust prompt
s=$1; shift; TM="tmux -L pf60rerun3"
$TM kill-session -t $s 2>/dev/null
$TM new-session -d -s $s -x 200 -y 50 "$*"; sleep 7
$TM capture-pane -p -t $s | grep -q 'Do you trust' && { $TM send-keys -t $s Enter; sleep 5; }
$TM capture-pane -p -t $s | grep -v '^\s*$' | tail -4
