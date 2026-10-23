# Create an "existing single-account home" hM with the PRE-PF-84 build (44b527f11f): Z.AI default key saved
# through TUI onboarding (masked paste from a tmux buffer that is deleted on paste), one TUI and one exec session.
. "$(dirname "$0")/../env.sh"; . $S/scripts/tmuxlib.sh
H=$S/homes/hM; mkdir -p $H
printf 'model = "glm-5.3-flash"\nmodel_provider = "zai"\n\n[projects."%s/work"]\ntrust_level = "trusted"\n' $S > $H/config.toml
s=m0-onboard
$T new-session -d -s $s -x 160 -y 50 "cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H $PRE --no-alt-screen; echo TUI_EXITED rc=\$?; sleep 9999"
wait_for $s 'API key|model: ' 60; cap $s | grep -v '^\s*$' | head -6 | sed 's/^/   | /'
if cap $s | grep -q 'API key'; then
  printf %s "$(helper provider/zai_api_key)" | $T load-buffer -b k -
  $T paste-buffer -p -d -b k -t $s; sleep 1; send_enter $s
  wait_for $s 'model: ' 60
fi
type_and_enter $s "$PROMPT"; wait_for $s '^• pong' 90; echo "-- pre-PF-84 TUI after onboarding:"; cap $s | grep -v '^\s*$' | grep -E 'pong|resume|401' | sed 's/^/   | /'
type_and_enter $s "/exit"; wait_for $s 'TUI_EXITED' 20; cap $s | grep -o -E 'corbanu resume [0-9a-f-]{36}' | tail -1 | awk '{print $3}' > $S/hM-tui-session; $T kill-session -t $s
$T list-buffers 2>/dev/null | grep -c '^k:' | sed 's/^/   tmux key buffers left: /'
out=$(cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H $PRE exec --skip-git-repo-check "$PROMPT" 2>&1)
printf '%s\n' "$out" | grep -E '^session id:' | sed 's/^session id: //' > $S/hM-exec-session
echo "-- pre-PF-84 exec: $(printf '%s\n' "$out" | grep -c -E '^pong') pong line(s); session $(cat $S/hM-exec-session)"
