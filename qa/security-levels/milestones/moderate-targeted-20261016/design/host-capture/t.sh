#!/bin/bash
# t.sh CAPDIR cmd args : tmux helper for local packet-screen capture
C="$1"; shift; T=(tmux -f /Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016/h/tmux.conf -S "$C/tmux.sock")
case "$1" in
  start) shift; "${T[@]}" new-session -d -s ${TS:-p} -x 150 -y 45 "/Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016/cap/product.sh $C $*";;
  text) shift; "${T[@]}" send-keys -t ${TS:-p} -l "$*";;
  key) shift; for k in "$@"; do "${T[@]}" send-keys -t ${TS:-p} "$k"; sleep 0.4; done;;
  screen) mkdir -p "$C/screens"; "${T[@]}" capture-pane -p -t ${TS:-p} | tee "$C/screens/$2.txt";;
  wait) for i in $(seq 1 ${3:-60}); do "${T[@]}" capture-pane -p -t ${TS:-p} | grep -q -E "$2" && exit 0; sleep 1; done; echo "TIMEOUT waiting for $2"; exit 1;;
  stop) "${T[@]}" kill-server;;
esac
