# TUI --account / resume / ghost on home hP (canary proxy attributes every request to an account).
. "$(dirname "$0")/../env.sh"; . $S/scripts/tmuxlib.sh
H=$S/homes/hP; R=$S/proxy/requests.jsonl
DEF=$(python3 -c "import json;d=json.load(open('$S/proxy/canaries.json'));print(next(k for k,v in d.items() if v=='default'))")
TUI_ENV="ZPX_API_KEY=$DEF"
seen() { python3 - "$R" "$1" <<'PY'
import json, sys, collections
rows = [json.loads(l) for l in open(sys.argv[1])][int(sys.argv[2]):]
c = collections.Counter((r['bearer'], r['status']) for r in rows)
print('   proxy saw', len(rows), 'request(s):', dict(c) if rows else '{}', '| canary/real values in bodies:',
      sorted({v for r in rows for v in r['canary_values_in_body']}) or 'none', any(r['real_key_in_body'] for r in rows))
PY
}
one() { local s=$1 expect=$2; shift 2; local n=$(wc -l < $R)
  tui_start $s $H "$@"
  if cap $s | grep -q 'TUI_EXITED'; then cap $s > $S/caps/tui-$s.txt; $T kill-session -t $s; echo "-- $s: exited at start"; seen $n; return; fi
  type_and_enter $s "$PROMPT"; wait_for $s "$expect" 120; echo "-- $s ($*): wait rc=$?"; tui_stop $s; seen $n; }
one p1-work '^• pong|401' --account work
one p2-fake 'was rejected|401' --account fake
id1=$(grep -o -E 'corbanu resume [0-9a-f-]{36}' $S/caps/tui-p1-work.txt | tail -1 | awk '{print $3}')
id2=$(grep -o -E 'corbanu resume [0-9a-f-]{36}' $S/caps/tui-p2-fake.txt | tail -1 | awk '{print $3}')
echo "p1 session $id1; p2 session $id2"
one p4-resume-fake 'was rejected|401|^• pong' resume $id2
one p4b-resume-work '^• pong|401' resume $id1
one p4c-resume-fake-as-main '^• pong|401' resume --account main $id2
# ghost: step-by-step captures (no prompt is typed until we know which screen is up)
n=$(wc -l < $R); s=p3-ghost; mkdir -p $S/logs/$s
$T new-session -d -s $s -x 160 -y 50 "cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H $TUI_ENV $B -c log_dir='\"$S/logs/$s\"' --no-alt-screen --account ghost; echo TUI_EXITED rc=\$?; sleep 9999"
wait_for $s 'API key|model: |rror|TUI_EXITED|Do you trust' 60; cap $s > $S/caps/tui-$s-1-start.txt; echo "-- $s: first screen captured"
if cap $s | grep -q 'API key'; then
  send_enter $s; wait_for $s 'model: |rror|TUI_EXITED|›' 30; cap $s > $S/caps/tui-$s-2-after-enter.txt; echo "-- $s: pressed Enter on the key screen (detected env value = default canary)"
  type_and_enter $s "$PROMPT"; wait_for $s 'pong|401|rror|rejected' 90; cap $s > $S/caps/tui-$s-3-prompt.txt; echo "-- $s: prompt sent"
fi
type_and_enter $s "/exit"; wait_for $s 'TUI_EXITED' 20; cap $s > $S/caps/tui-$s-4-end.txt; $T kill-session -t $s; seen $n
echo "accounts after p3:"; cx $H account list | sed 's/^/  /'
