# In-process spawn matrix on real Z.AI GLM 5.3 Flash (home hS from spawn-setup.sh; default account = invalid canary).
# Evidence: exec --json events plus the coordinator's and children's rollouts (rollout_view.py).
# Run as: bash spawn-exec.sh [case...]   (SP9 also needs X="$(helper provider/zai_api_key)")
. "$(dirname "$0")/../env.sh"
H=${H:-$S/homes/hS}; DEFC=$(cat $S/default-canary.txt)
TAIL='Wait for it to finish (wait tool), then reply with one line: the sub-agent result verbatim. If any tool returns an error, reply with that error text verbatim instead.'
sp() { local label=$1 args=$2 prompt=$3 out tid; shift 3
  out=$(cx $H exec --skip-git-repo-check --json "$@" "$prompt $TAIL" 2>$S/sp.err); rc=$?
  tid=$(printf '%s\n' "$out" | python3 -c "import json,sys
for l in sys.stdin:
  try: d=json.loads(l)
  except: continue
  if d.get('type')=='thread.started': print(d['thread_id']); break")
  echo "== $label (exit $rc)"; echo "   exec args: $*"; echo "   spawn request: $args"
  printf '%s\n' "$out" | python3 -c "import json,sys
for l in sys.stdin:
  try: d=json.loads(l)
  except: continue
  i=d.get('item') or {}
  if d.get('type')=='item.completed' and i.get('type')=='agent_message': print('   final reply:', i['text'][:300].replace(chr(10),' '))
  if d.get('type') in ('error','turn.failed'): print('   event:', json.dumps(d)[:300])"
  grep -E '^(ERROR|Error)' $S/sp.err | head -3 | sed 's/^/   stderr: /'
  [ -n "$tid" ] && python3 $S/scripts/rollout_view.py $H $tid | sed 's/^/   /'; }
want() { [ $# -eq 0 ] && return 0; for c in $CASES; do [ "$c" = "$1" ] && return 0; done; return 1; }
E=(ZAI_API_KEY="$DEFC"); [ -n "${REALDEF:-}" ] && E=(ZAI_API_KEY="$X")   # REALDEF=1: default = real key
CASES="$*"; [ -z "$CASES" ] && CASES="SP1 SP2 SP3 SP4 SP5 SP6 SP7 SP8"
want SP1 && sp "SP1 coordinator --account main; spawn without account (inherit; default would 401)" '{message}' \
  'Use the spawn_agent tool exactly once with only the message "Reply with exactly the word child-ok" (no other arguments).' --account main
want SP2 && sp "SP2 coordinator --account main; spawn account=work" '{message, account:"work"}' \
  'Use the spawn_agent tool exactly once with the message "Reply with exactly the word child-ok" and the argument account set to "work".' --account main
want SP3 && sp "SP3 coordinator --account main; spawn account=fake (child must 401; no retry elsewhere)" '{message, account:"fake"}' \
  'Use the spawn_agent tool exactly once with the message "Reply with exactly the word child-ok" and the argument account set to "fake".' --account main
want SP4 && sp "SP4 spawn account=ghost (not configured)" '{message, account:"ghost"}' \
  'Use the spawn_agent tool exactly once with the message "Reply with exactly the word child-ok" and the argument account set to "ghost".' --account main
want SP5 && sp "SP5 spawn account=kimionly (configured for kimi-code only)" '{message, account:"kimionly"}' \
  'Use the spawn_agent tool exactly once with the message "Reply with exactly the word child-ok" and the argument account set to "kimionly".' --account main
want SP6 && sp "SP6 role child agent_type=explorer, no account (inherit)" '{message, agent_type:"explorer"}' \
  'Use the spawn_agent tool exactly once with the message "Reply with exactly the word child-ok" and agent_type set to "explorer" (no account argument).' --account main
want SP7 && sp "SP7 role child agent_type=explorer + account=fake" '{message, agent_type:"explorer", account:"fake"}' \
  'Use the spawn_agent tool exactly once with the message "Reply with exactly the word child-ok", agent_type set to "explorer" and account set to "fake".' --account main
want SP8 && sp "SP8 coordinator selected by config provider_accounts.zai=work; spawn without account" '{message}' \
  'Use the spawn_agent tool exactly once with only the message "Reply with exactly the word child-ok" (no other arguments).' -c 'provider_accounts.zai="work"'
want SP9 && E=(ZAI_API_KEY="$X") && sp "SP9 flag OFF, real default key; ask for account=fake" '{message, account:"fake"}' \
  'Use the spawn_agent tool exactly once with the message "Reply with exactly the word child-ok" and the argument account set to "fake".' --disable named_accounts
true
# --- D3 under Aggressive (exec cannot ask the human): run with AGGR=1 after writing security_level.toml
want SP10 && sp "SP10 Aggressive in force (exec, no approver): spawn account=work" '{message, account:"work"}' \
  'Use the spawn_agent tool exactly once with the message "Reply with exactly the word child-ok" and the argument account set to "work".' --account main --enable security_levels ${AGGR_ARGS}
want SP11 && sp "SP11 Aggressive in force (exec): spawn without account (inherit; not a switch)" '{message}' \
  'Use the spawn_agent tool exactly once with only the message "Reply with exactly the word child-ok" (no other arguments).' --account main --enable security_levels ${AGGR_ARGS}
want SP12 && sp "SP12 Aggressive in force (exec): spawn account=main (same as the parent)" '{message, account:"main"}' \
  'Use the spawn_agent tool exactly once with the message "Reply with exactly the word child-ok" and the argument account set to "main".' --account main --enable security_levels ${AGGR_ARGS}
want SP13 && sp "SP13 Permissive saved level, security_levels on (exec): spawn account=work" '{message, account:"work"}' \
  'Use the spawn_agent tool exactly once with the message "Reply with exactly the word child-ok" and the argument account set to "work".' --account main --enable security_levels ${AGGR_ARGS}
true
