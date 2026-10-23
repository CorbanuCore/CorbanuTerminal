#!/bin/bash
# providers.sh : second/third providers in one home (hc), candidate developer-accounting build.
#  - Claude Plan: named account `cp` holding the real plan token, and `cpfake` holding a fake token.
#    (The default Claude Plan account is never used, so no Claude Code login is read.)
#  - OpenAI gpt-5.4 on the default account (API-key login); named OpenAI accounts are out of scope
#    until S06 and must be refused.
#  - Z.AI fake account, so the view holds named work.
# Then the ledger is reconciled against each thread's rollout.
cd "$(dirname "$0")" && . ./env.sh
H=$S/homes/hc OUT=$EV/captures/providers.txt; mkdir -p "$H"; : > "$OUT"
printf 'model = "glm-5.3-flash"\nmodel_provider = "zai"\n\n[features]\nnamed_accounts = true\n' > "$H/config.toml"
key claude-plan-test-token run_secret_stdin "$CAND" "$H" account add claude-plan cp --kind claude-token 2>&1 | grep -v WARNING >> "$OUT"
SECRET="qa-fake-claude-plan-token-S05" run_secret_stdin "$CAND" "$H" account add claude-plan cpfake --kind claude-token 2>&1 | grep -v WARNING >> "$OUT"
SECRET="qa-fake-zai-3333333333333333.S05FAKE4" run_secret_stdin "$CAND" "$H" account add zai fake 2>&1 | grep -v WARNING >> "$OUT"
key openai-api-key run_secret_stdin "$CAND" "$H" login --with-api-key 2>&1 | grep -v WARNING | tail -1 >> "$OUT"
echo "account add openai x: $(SECRET=qa-fake-openai run_secret_stdin "$CAND" "$H" account add openai x 2>&1 | grep -v WARNING | head -1 | cut -c1-200)" >> "$OUT"
x() { local name=$1; shift
  run "$CAND" "$H" exec --json --skip-git-repo-check -C "$S/ws" "$@" "Reply with exactly the word pong and nothing else." > "$S/logs/pv-$name.jsonl" 2> "$S/logs/pv-$name.err"
  local rc=$?
  echo "=== $name: exec $* -> exit=$rc $(python3 -c "
import json,sys
for l in open('$S/logs/pv-$name.jsonl'):
  try: e=json.loads(l)
  except ValueError: continue
  t=e.get('type')
  if t=='thread.started': print('thread', e['thread_id'], end=' ')
  elif t=='item.completed' and e['item'].get('type')=='agent_message': print('reply', repr(e['item']['text'][:30]), end=' ')
  elif t=='turn.completed': print('usage', json.dumps(e['usage'], sort_keys=True), end=' ')
  elif t in ('turn.failed','error'): print(t, e.get('message') or e.get('error',{}).get('message','')[:200], end=' ')
") $(grep -E '^Error|ERROR' "$S/logs/pv-$name.err" | tail -1 | cut -c1-200)" >> "$OUT"; }
x claude-cp     -m claude-opus-5-5-plan -c model_provider='"claude-plan"' --account cp
x claude-cpfake -m claude-opus-5-5-plan -c model_provider='"claude-plan"' --account cpfake
x openai-default -m gpt-5.4 -c model_provider='"openai"'
x openai-named  -m gpt-5.4 -c model_provider='"openai"' --account work
x zai-fake      -m glm-5.3-flash -c model_provider='"zai"' --account fake
echo "--- ledger vs rollouts ---" >> "$OUT"
python3 ./reconcile.py "$H"/pfterminal_state_*.sqlite - "$H/sessions" >> "$OUT"
echo "--- throttle keys ---" >> "$OUT"
sqlite3 "$H"/pfterminal_state_*.sqlite "select provider_id, model, key_fingerprint from provider_request_state" | sed -E 's/(env:[A-Za-z_]+:|auth:[A-Za-z_]+:)[0-9a-f]{8,}/\1<hash>/' >> "$OUT"
sed -i '' "s#$S#\$S#g" "$OUT"; cat "$OUT"
