S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; . $S/tmuxlib.sh; s=t5; G=$S/s02/hG; mkdir -p $S/logs/$s
$T new-session -d -s $s -x 160 -y 50 "cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin:/opt/homebrew/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$G RUST_LOG=info $S/bin/corbanu-main -c log_dir='\"$S/logs/$s\"' -s danger-full-access -a never --no-alt-screen; echo TUI_EXITED rc=\$?; sleep 9999"
wait_for $s 'model: ' 60
type_and_enter $s "Run the shell command: sh $S/s01/worker-doctor.sh -- then show me its complete output verbatim in a code block and say which home the worker used."
wait_for $s 'sqlite home.*\n?.*' 180; sleep 20; wait_for $s 'esc to interrupt|›' 120
cap $s > $S/caps/tui-$s-a.txt
type_and_enter $s "Now run the shell command: sh $S/s01/worker-exec.sh -- and show me its complete output verbatim in a code block."
sleep 30; wait_for $s '401|pong' 180
sleep 15; $T capture-pane -p -t $s -S -400 > $S/caps/tui-$s.txt
type_and_enter $s "/exit"; wait_for $s 'TUI_EXITED' 30; $T capture-pane -p -t $s -S -400 > $S/caps/tui-$s.txt; $T kill-session -t $s
