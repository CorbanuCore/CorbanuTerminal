# Source after env.sh. (Copied from the S05 acceptance; start_tui takes FH and XENV.) Private tmux socket; rules from docs/tmuxHarness.md:
# literal text and Enter are sent separately, waits need two identical matching
# captures, sessions end with /exit.
tx() { tmux -S "$SOCK" "$@"; }
cap() { tx capture-pane -p -t "$1" -S "${2:--200}"; }
# wait_for <session> <regex> [timeout_s]: two consecutive identical captures that match.
wait_for() {
  local s=$1 re=$2 to=${3:-120} prev="" cur i
  for ((i = 0; i < to * 2; i++)); do
    cur=$(cap "$s" -60)
    if grep -Eq -- "$re" <<<"$cur" && [ "$cur" = "$prev" ]; then return 0; fi
    prev=$cur; sleep 0.5
  done
  echo "TIMEOUT waiting for /$re/ in $s" >&2; return 1
}
send_line() { # send_line <session> <text>: type, verify the composer shows it (retry once), then Enter
  local try
  for try in 1 2; do
    tx send-keys -t "$1" -l -- "$2"; sleep 0.8
    cap "$1" -60 | tr -d '\n' | grep -qF -- "${2:0:40}" && break
    tx send-keys -t "$1" C-u; sleep 0.5
    [ "$try" = 2 ] && { echo "send_line: composer never showed the text; aborting (no Enter sent)" >&2; return 1; }
  done
  tx send-keys -t "$1" Enter
}
# start_tui <session> <home> args...  (candidate unless BIN is set)
start_tui() {
  local s=$1 home=$2; shift 2
  local cmd; cmd=$(printf '%q ' env -i HOME="${FH:-$S/fakehome}" USER="$USER" TERM=xterm-256color LANG=en_US.UTF-8 \
    PATH="$S/pathbin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin" \
    CORBANU_HOME="$home" CORBANU_TEST_NO_NATIVE_KEYRING=1 RUST_LOG=info ${XENV[@]+"${XENV[@]}"} "${BIN:-$CAND}" -C "$S/ws" "$@")
  echo "+ tui[$s]: ${cmd//$S/\$S}" >> "$S/logs/tui-invocations.txt"
  tx new-session -d -s "$s" -x 160 -y 50 "$cmd; echo EXITED-\$?; sleep 600"
}
# start_keyed_tui <socket> <session> <vault-label> <ENVVAR> <home> args...
# Starts a dedicated tmux server whose environment is clean plus ENVVAR=<real key>
# (from the vault helper, never argv), then runs the TUI in it.
start_keyed_tui() {
  local sock=$1 s=$2 label=$3 var=$4 home=$5; shift 5
  local cmd; cmd=$(printf '%q ' "${BIN:-$CAND}" -C "$S/ws" "$@")
  echo "+ tui[$s] home=${home//$S/\$S} HOME=${FH:-\$S/fakehome} env=$var(real default key) + ${XENV[*]:-} : ${cmd//$S/\$S}" >> "$S/logs/tui-invocations.txt"
  key "$label" env CORBANU_HOME="$home" S="$S" FH="${FH:-$S/fakehome}" ${XENV[@]+"${XENV[@]}"} python3 "$(dirname "${BASH_SOURCE[0]}")/cleanexec.py" "$var" -- \
    /opt/homebrew/bin/tmux -S "$sock" new-session -d -s "$s" -x 170 -y 52 "$cmd; echo EXITED-\$?; sleep 600"
}
# pick <session> <regex> [max]: press Down until the selected popup row ("› N. ...") matches, then Enter.
pick() {
  local s=$1 re=$2 max=${3:-60} i
  for ((i = 0; i < max; i++)); do
    cap "$s" -30 | grep -E '^› +[0-9]+\. ' | tail -1 | grep -Eq -- "$re" && { tx send-keys -t "$s" Enter; sleep 2; return 0; }
    tx send-keys -t "$s" Down; sleep 0.25
  done
  echo "pick: /$re/ not found" >&2; return 1
}
