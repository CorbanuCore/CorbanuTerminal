# Run BY THE COORDINATOR AGENT in its shell: out-of-process workers through the checked-in launcher.
# Inherits the coordinator's CORBANU_HOME and CORBANU_LAUNCHER_* from the TUI. Prints one line per worker.
L="bash $CORBANU_LAUNCHER_WORK_DIR/corbanu-launcher.sh"
w() { local tag=$1; shift; local out rc; out=$($L exec --skip-git-repo-check "$@" "Reply with exactly the word pong-$tag and nothing else." 2>&1); rc=$?
  echo "$tag exit=$rc args=[$*] -> $(printf '%s\n' "$out" | grep -E '^pong|ERROR|Error' | grep -v corbanu_call_metrics | head -1 | cut -c1-200)"
  printf '%s\n' "$out" | grep -E '^session id:' | head -1 | sed 's/^session id: //' > /tmp/pf84s03acc-$tag.sid; }
echo "worker env: CORBANU_HOME=${CORBANU_HOME:+set} ZPX_API_KEY=$([ -n "${ZPX_API_KEY:-}" ] && echo set || echo unset)"
w W1 --account work
w W2 --account fake
w W3 --account ghost
w W4 resume "$(cat /tmp/pf84s03acc-W2.sid)"
w W5 resume --account main "$(cat /tmp/pf84s03acc-W2.sid)"
w W6
