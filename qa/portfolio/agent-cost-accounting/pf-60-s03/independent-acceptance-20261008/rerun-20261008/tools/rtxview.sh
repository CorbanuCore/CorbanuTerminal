# usage (remote): rtxview.sh <sess> <home> <repo> <thread-id or ""> : start TUI in tmux, accept trust prompt
cd <rtx-dir>; R=$PWD; s=$1; h=$2; repo=$3; id=$4
tmux -L pf60rerun kill-session -t $s 2>/dev/null
if [ -n "$id" ]; then a="resume $id"; else a=""; fi
tmux -L pf60rerun new-session -d -s $s -x 200 -y 50 "BIN=${BIN:-} $R/rtx-tui.sh $R/$h $R/repos/$repo $a"; sleep 6
tmux -L pf60rerun capture-pane -p -t $s | grep -q 'Do you trust' && { tmux -L pf60rerun send-keys -t $s Enter; sleep 5; }
tmux -L pf60rerun capture-pane -p -t $s | grep -v '^\s*$' | tail -4
