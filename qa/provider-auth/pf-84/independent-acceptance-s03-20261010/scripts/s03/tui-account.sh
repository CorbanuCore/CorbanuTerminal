# TUI --account / provider_accounts / resume on real Z.AI (home hS: main/work real, fake synthetic,
# default = synthetic invalid canary in env ZAI_API_KEY). Driven through tmux per docs/tmuxHarness.md.
. "$(dirname "$0")/../env.sh"; . $S/scripts/tmuxlib.sh
H=$S/homes/hS; rm -f $H/security_level.toml
TUI_ENV="ZAI_API_KEY=$(cat $S/default-canary.txt)"
one() { local s=$1 expect=$2; shift 2
  tui_start $s $H "$@"
  if cap $s | grep -q 'TUI_EXITED'; then cap $s > $S/caps/tui-$s.txt; $T kill-session -t $s; echo "-- $s: exited at start"; return; fi
  type_and_enter $s "$PROMPT"; wait_for $s "$expect" 120; echo "-- $s: wait rc=$?"; tui_stop $s; }
one t1-main '^• pong|401|rror' --account main
one t2-fake 'credential was rejected|401' --account fake
one t2b-config-fake 'credential was rejected|401' -c 'provider_accounts.zai="fake"'
one t3-ghost 'not configured|rror'  --account ghost
one t3b-flagoff-fake 'pong|rror' --disable named_accounts --account fake
# resume: the TUI prints "corbanu resume <id>" on exit; reuse the ids of t1 and t2
id1=$(grep -o -E 'corbanu resume [0-9a-f-]{36}' $S/caps/tui-t1-main.txt | tail -1 | awk '{print $3}')
id2=$(grep -o -E 'corbanu resume [0-9a-f-]{36}' $S/caps/tui-t2-fake.txt | tail -1 | awk '{print $3}')
echo "t1 session $id1; t2 session $id2"
one t4-resume-fake 'credential was rejected|401|^• pong' resume $id2
one t4b-resume-fake-as-main '^• pong|401' resume --account main $id2
one t4c-resume-main-config-fake '^• pong|401' resume -c 'provider_accounts.zai="fake"' $id1
one t4d-resume-ghost 'not configured|rror' resume --account ghost $id1
