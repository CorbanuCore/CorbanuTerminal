#!/bin/bash
# remote. usage: drv.sh <session> <op> [args]
#   text "<literal>"     send literal text (no Enter)
#   key <tmux-key> [n]   send a key n times
#   wait "<regex>" [s]   wait until two identical captures match (default 120 s); prints MATCH or TIMEOUT
#   cap <name>           save the visible screen to cap/<name>.txt
#   scroll <name> [n]    save every distinct screen while pressing Down n times (default 60) to cap/<name>.txt
#   kill                 kill the tmux server (and the candidate) without /exit
Q=$HOME/corbanu-rtx/pf60s04-ia; s=$1; op=$2; shift 2
T="tmux -L ia-$s"
snap(){ $T capture-pane -p -J -t "$s" | sed 's/[[:space:]]*$//'; }
case $op in
  text) $T send-keys -t "$s" -l "$1"; sleep 0.5 ;;
  key) for i in $(seq 1 ${2:-1}); do $T send-keys -t "$s" "$1"; sleep 0.25; done ;;
  wait) end=$(( $(date +%s) + ${2:-120} )); prev=""
        while [ $(date +%s) -lt $end ]; do f=$(snap)
          if [ "$f" = "$prev" ] && grep -Pq -- "$1" <<<"$f"; then echo MATCH; exit 0; fi
          prev=$f; sleep 1; done; echo TIMEOUT; snap | tail -25; exit 1 ;;
  cap) snap > "$Q/cap/$1.txt"; echo "saved $1" ;;
  scroll) out="$Q/cap/$1.txt"; : > "$out.frames"; prev=""
          for i in $(seq 0 ${2:-60}); do f=$(snap | awk 'NF{p=1} p')
            if [ "$f" != "$prev" ]; then printf -- '--- frame %d\n%s\n' $i "$f" >> "$out.frames"; prev="$f"; fi
            [ $i -lt ${2:-60} ] && $T send-keys -t "$s" Down; sleep 0.15; done
          grep -v '^--- frame' "$out.frames" | awk '!seen[$0]++' > "$out"; echo "saved $1 ($(wc -l < $out) lines)" ;;
  kill) $T kill-server; echo killed ;;
esac
