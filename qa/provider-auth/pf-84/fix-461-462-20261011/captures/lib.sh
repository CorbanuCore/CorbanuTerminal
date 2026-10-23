S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84fix
T="tmux -L pf84fixqa"
capv() { $T capture-pane -p -t "$1"; }
cap() { $T capture-pane -p -t "$1" -S -${2:-400}; }
settle() { local s=$1 prev="" cur i=0; while [ $i -lt 10 ]; do cur=$(capv "$s"); [ "$cur" = "$prev" ] && return 0; prev=$cur; sleep 0.5; i=$((i+1)); done; }
wait_for() { local s=$1 pat=$2 to=${3:-60} prev="" cur i=0
  while [ $i -lt $((to*2)) ]; do cur=$(capv "$s"); if printf '%s' "$cur" | grep -q -E -- "$pat" && [ "$cur" = "$prev" ]; then return 0; fi; prev=$cur; sleep 0.5; i=$((i+1)); done
  echo "TIMEOUT waiting for /$pat/ in $s"; return 1; }
send_text() { $T send-keys -t "$1" -l -- "$2"; }
key() { local s=$1; shift; for k in "$@"; do $T send-keys -t "$s" "$k"; sleep 0.4; done; settle "$s"; }
select_row() { local s=$1 re=$2 i=0; while [ $i -lt 60 ]; do capv "$s" | grep -E '^\s*[>›]' | grep -q -E -- "$re" && { settle "$s"; return 0; }; $T send-keys -t "$s" Down; sleep 0.25; i=$((i+1)); done; echo "NO ROW /$re/"; return 1; }
# tui_start <session> <bin> <home> [args]
tui_start() { local s=$1 b=$2 home=$3; shift 3; local args="" a; for a in "$@"; do args="$args '$a'"; done; mkdir -p "$home" $S/logs/$s
  $T new-session -d -s "$s" -x 140 -y 45 "cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$home ${TUI_ENV:-} $b -c log_dir='\"$S/logs/$s\"' -c tui.animations=false --no-alt-screen $args; echo TUI_EXITED rc=\$?; sleep 9999"
  sleep 3; settle "$s"; }
