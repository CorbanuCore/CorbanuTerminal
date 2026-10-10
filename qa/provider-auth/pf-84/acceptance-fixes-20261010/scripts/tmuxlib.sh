T="tmux -L pf84fix"
cap() { $T capture-pane -p -t "$1"; }
waitfor() { # target pattern timeout
  local t=$1 p=$2 n=${3:-60} prev="" cur
  for i in $(seq 1 $n); do cur=$(cap $t); if printf '%s' "$cur" | grep -qE "$p"; then [ "$cur" = "$prev" ] && return 0; prev=$cur; fi; sleep 1; done; return 1; }
start() { # name bin home withenv args...
  local name=$1; shift
  $T new-window -d -n $name "env -i /bin/bash <scratch>/run-tui.sh $*; sleep 99999"; }
say() { $T send-keys -t "$1" -l "$2"; sleep 0.5; $T send-keys -t "$1" Enter; }
