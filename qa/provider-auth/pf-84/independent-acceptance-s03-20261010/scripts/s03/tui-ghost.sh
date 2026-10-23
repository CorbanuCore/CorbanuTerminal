# TUI `--account ghost` on the built-in zai provider (home hS; default = synthetic env canary, which Z.AI rejects).
# Step-by-step captures: what does the TUI do with a missing account, and where does a confirmed key go?
. "$(dirname "$0")/../env.sh"; . $S/scripts/tmuxlib.sh
probe() { local v; v=$(cx $H vault auth-helper provider/zai_api_key 2>/dev/null); echo "  default label provider/zai_api_key: $( [ -n "$v" ] && printf %s "$v" | shasum -a 256 | cut -c1-12 || echo '<absent>') (default canary sha12 $(shasum -a 256 < $S/default-canary.txt | cut -c1-12)); vault files: $(ls $H/secrets | tr '\n' ' ')"; }
H=$S/homes/hS; s=${1:-g1-ghost}; ACC=${2:-ghost}; mkdir -p $S/logs/$s
echo "before:"; probe
$T new-session -d -s $s -x 160 -y 50 "cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H ZAI_API_KEY=$(cat $S/default-canary.txt) $B -c log_dir='\"$S/logs/$s\"' --no-alt-screen --account $ACC; echo TUI_EXITED rc=\$?; sleep 9999"
wait_for $s 'API key|model: |rror|TUI_EXITED' 60; cap $s > $S/caps/tui-$s-1-start.txt; echo "-- start screen:"; grep -v '^\s*$' $S/caps/tui-$s-1-start.txt | head -12 | sed 's/^/   | /'
if cap $s | grep -q 'API key'; then
  send_enter $s; sleep 3; wait_for $s 'model: |rror|TUI_EXITED|›' 30; cap $s > $S/caps/tui-$s-2-after-enter.txt
  echo "-- after Enter on the key screen:"; grep -v '^\s*$' $S/caps/tui-$s-2-after-enter.txt | tail -12 | sed 's/^/   | /'
  if cap $s | grep -q 'model: '; then
    type_and_enter $s "$PROMPT"; wait_for $s 'pong|401|rror|rejected' 90; cap $s > $S/caps/tui-$s-3-prompt.txt
    echo "-- after the prompt:"; grep -v '^\s*$' $S/caps/tui-$s-3-prompt.txt | tail -8 | sed 's/^/   | /'
    type_and_enter $s "/exit"; wait_for $s 'TUI_EXITED' 20
  fi
fi
cap $s > $S/caps/tui-$s-4-end.txt; $T kill-session -t $s
echo "after:"; probe
echo "accounts after:"; cx $H account list | sed 's/^/  /'
