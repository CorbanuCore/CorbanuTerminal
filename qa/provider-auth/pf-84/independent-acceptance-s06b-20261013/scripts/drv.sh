# source drv.sh <socket-name>: interactive driving context for the #555 TUI runs.
D=/Volumes/CorbanuDrive/Corbanu/worktrees/s06b-acceptance-20261013/qa/provider-auth/pf-84/independent-acceptance-s06b-20261013/scripts
source $D/env.sh; source $D/tmuxlib.sh; source $D/z_setup.sh
export SOCK=$S/tmux-$1.sock
# zt <session> <home> args... : TUI with the real Z.AI key as ZAI_API_KEY (default account)
zt() { local s=$1 h=$2; shift 2; export SOCK=$S/tmux-$s.sock; start_keyed_tui $SOCK $s provider/zai_api_key ZAI_API_KEY $S/homes/$h "$@"; }
# nt <session> <home> args... : TUI with no Z.AI key in the environment
nt() { local s=$1 h=$2; shift 2; export SOCK=$S/tmux-$s.sock; local cmd; cmd=$(printf '%q ' "${BIN:-$CAND}" -C "$S/ws" "$@")
  echo "+ tui[$s] home=$h (no ZAI key) : ${cmd//$S/\$S}" >> "$S/logs/tui-invocations.txt"
  SECRET= env CORBANU_HOME=$S/homes/$h S=$S FH=$FH ${XENV[@]+"${XENV[@]}"} python3 $D/cleanexec.py -- /opt/homebrew/bin/tmux -S $SOCK new-session -d -s $s -x 170 -y 52 "$cmd; echo EXITED-\$?; sleep 600"; }
# turn <s> <word>: ask for a single word and wait for the reply bullet
turn() { send_line $1 "Reply with exactly the word $2"; wait_for $1 "^• $2\$" 90 || sleep 5; }
status() { send_line $1 "/status"; wait_for $1 'Limits:|Account usage' 20; sleep 1; }
quit() { tx send-keys -t $1 Escape; sleep 0.5; send_line $1 "/exit"; sleep 3; }
use() { export SOCK=$S/tmux-$1.sock; }
# ft <session> <home> args... : TUI with a FAKE Z.AI key as ZAI_API_KEY (so the default account answers 401)
ft() { local s=$1 h=$2; shift 2; export SOCK=$S/tmux-$s.sock; local cmd; cmd=$(printf '%q ' "${BIN:-$CAND}" -C "$S/ws" "$@")
  echo "+ tui[$s] home=$h ZAI_API_KEY=<fake default key> : ${cmd//$S/\$S}" >> "$S/logs/tui-invocations.txt"
  SECRET=qa-fake-zai-default.S6BFAKE env CORBANU_HOME=$S/homes/$h S=$S FH=$FH ${XENV[@]+"${XENV[@]}"} python3 $D/cleanexec.py ZAI_API_KEY -- /opt/homebrew/bin/tmux -S $SOCK new-session -d -s $s -x 170 -y 52 "$cmd; echo EXITED-\$?; sleep 600"; }
