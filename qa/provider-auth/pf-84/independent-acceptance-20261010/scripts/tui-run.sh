# tui-run.sh <session> <home> <prompt> <expect-regex> [extra args...]
S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; . $S/tmuxlib.sh
s=$1 home=$2 prompt=$3 expect=$4; shift 4; mkdir -p $S/logs/$s
args=""; for a in "$@"; do args="$args '$a'"; done
$T new-session -d -s $s -x 150 -y 46 "cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin:/opt/homebrew/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$home RUST_LOG=trace $S/bin/corbanu-main -c log_dir='\"$S/logs/$s\"' --no-alt-screen $args; echo TUI_EXITED rc=\$?; sleep 9999"
wait_for $s 'Do you trust|model: ' 60 || true
if cap $s | grep -q 'Do you trust'; then send_enter $s; sleep 1; fi
wait_for $s 'model: ' 30 || true; sleep 2
type_and_enter $s "$prompt"
wait_for $s "$expect" 120; echo "wait rc=$?"
cap $s > $S/caps/tui-$s.txt
type_and_enter $s "/exit"; wait_for $s 'TUI_EXITED' 30; cap $s > $S/caps/tui-$s.txt; $T kill-session -t $s
