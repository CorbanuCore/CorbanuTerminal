# Source me on glitch. PF-84 #570-#573 QA: disposable homes, fake tokens, local mock only.
export Q=$HOME/corbanu-rtx/f57x-qa
export S=$Q/run
export BASE=$HOME/corbanu-rtx/f57x-bin/corbanu-base
export CAND=$HOME/corbanu-rtx/f57x-bin/corbanu-cand
export SOCK=$S/tmux.sock
mkdir -p "$S/homes" "$S/logs" "$S/ws" "$S/fakehome"
# run <bin> <corbanu-home> args...   (clean env, disposable HOME, no native keyring)
run() {
  local bin=$1 home=$2; shift 2
  env -i HOME="${FH:-$S/fakehome}" USER="$USER" TERM=xterm-256color LANG=C.UTF-8 PATH=/usr/bin:/bin \
    CORBANU_HOME="$home" CORBANU_TEST_NO_NATIVE_KEYRING=1 ${XENV[@]+"${XENV[@]}"} "$bin" "$@"
}
show() { printf '$ %s\n' "$*" | sed "s#$S#\$S#g; s#$HOME/corbanu-rtx/f57x-bin/##g"; "$@" 2>&1 | sed "s#$S#\$S#g"; printf '[exit %s]\n' "${PIPESTATUS[0]}"; }
mocklines() { wc -l < "$1" | tr -d ' '; }
ctl() { curl -s -X POST -H 'content-type: application/json' -d "$1" "$U/_ctl" > /dev/null; }
tx() { tmux -S "$SOCK" "$@"; }
cap() { tx capture-pane -p -t "$1" -S "${2:--200}"; }
send_line() {
  local try
  for try in 1 2; do
    tx send-keys -t "$1" -l -- "$2"; sleep 0.8
    cap "$1" -60 | tr -d '\n' | grep -qF -- "${2:0:40}" && break
    tx send-keys -t "$1" C-u; sleep 0.5
    [ "$try" = 2 ] && { echo "send_line: composer never showed the text" >&2; return 1; }
  done
  tx send-keys -t "$1" Enter
}
pick() {
  local s=$1 re=$2 max=${3:-60} i
  for ((i = 0; i < max; i++)); do
    cap "$s" -40 | grep -E '^ *› +[0-9]+\. ' | tail -1 | grep -Eq -- "$re" && { tx send-keys -t "$s" Enter; sleep 2; return 0; }
    tx send-keys -t "$s" Down; sleep 0.25
  done
  echo "pick: /$re/ not found" >&2; return 1
}
# start_tui <session> <bin> <home> args...
start_tui() {
  local s=$1 bin=$2 home=$3; shift 3
  local cmd; cmd=$(printf '%q ' env -i HOME="${FH:-$S/fakehome}" USER="$USER" TERM=xterm-256color LANG=C.UTF-8 PATH=/usr/bin:/bin \
    CORBANU_HOME="$home" CORBANU_TEST_NO_NATIVE_KEYRING=1 ${XENV[@]+"${XENV[@]}"} "$bin" -C "$S/ws" "$@")
  tx new-session -d -s "$s" -x 170 -y 52 "$cmd; echo EXITED-\$?; sleep 600"
}
