#!/usr/bin/env bash
# Real providers, candidate developer-accounting build, one disposable home (r1).
# Real keys come only from the installed helper inside the consuming command (stdin or the env of the
# process), never argv, never printed. Named fake accounts get 401 while the REAL key sits in the
# provider's env var (no fallback). Output: captures/real.txt (re-acceptance copy of the 2026-10-11 script)
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"
export FH=$S/fakehome H=$S/homes/r1
[ -d $H ] && rm -r $H < /dev/null; mkdir -p $H
printf 'model = "glm-5.3-flash"\nmodel_provider = "zai"\n[features]\nnamed_accounts = true\n[projects."%s/ws"]\ntrust_level = "trusted"\n' $S > $H/config.toml
OUT=$EV/captures/real.txt; XENV=()
q() { grep -v 'WARNING: proceeding' | sed "s#$S#\$S#g"; }
# x <name> <label-for-env-or-> <ENVVARS(comma)|-> args... : exec --json, summarise reply/usage/errors
x() {
  local name=$1 label=$2 vars=$3; shift 3
  local cmd=(exec --json --skip-git-repo-check -C "$S/ws" "$@" "Reply with exactly the word pong and nothing else.")
  if [ "$label" = - ]; then run "$CAND" "$H" "${cmd[@]}" < /dev/null > $S/logs/real-$name.jsonl 2> $S/logs/real-$name.err
  else key "$label" env CORBANU_HOME=$H S=$S FH=$FH python3 $D/cleanexec.py ${vars//,/ } -- "$CAND" "${cmd[@]}" < /dev/null > $S/logs/real-$name.jsonl 2> $S/logs/real-$name.err; fi
  local rc=$?
  echo "=== $name: exec $* $( [ "$label" = - ] || echo "[env ${vars} = REAL key from $label]") -> exit=$rc $(python3 -c "
import json
for l in open('$S/logs/real-$name.jsonl'):
  try: e=json.loads(l)
  except ValueError: continue
  t=e.get('type')
  if t=='item.completed' and e['item'].get('type')=='agent_message': print('reply', repr(e['item']['text'][:30]), end=' ')
  elif t=='turn.completed': print('usage', json.dumps(e['usage'], sort_keys=True), end=' ')
  elif t in ('turn.failed','error'): print(t, (e.get('message') or e.get('error',{}).get('message',''))[:260], end=' ')
") $(grep -E '^Error|ERROR: ' $S/logs/real-$name.err | grep -v 'codex_api::' | sort -u | cut -c1-300 | tr '\n' ' ')" | q
}
{
echo "## candidate $(cat $S/bin/COMMIT-cand)"
echo "## enrolment (values on stdin)"
key provider/zai_api_key run_secret_stdin $CAND $H account add zai real 2>&1 | q
SECRET=qa-fake-zai-4444444444444444.S6BFAKE run_secret_stdin $CAND $H account add zai fake 2>&1 | q
key provider/kimi_api_key run_secret_stdin $CAND $H account add kimi-code alt 2>&1 | q
SECRET=sk-kimi-fake-S6BFAKE run_secret_stdin $CAND $H account add kimi-code kfake 2>&1 | q
key claude-plan-test-token run_secret_stdin $CAND $H account add claude-plan cp --kind claude-token 2>&1 | q
SECRET=qa-fake-claude-plan-token-S6B run_secret_stdin $CAND $H account add claude-plan cpfake --kind claude-token 2>&1 | q
SECRET=sk-fake-default-S6BFAKE run_secret_stdin $CAND $H login --with-api-key 2>&1 | q
key openai-api-key run_secret_stdin $CAND $H login --account real --with-api-key 2>&1 | q
SECRET=sk-fake-named-S6BFAKE run_secret_stdin $CAND $H login --account fake --with-api-key 2>&1 | q
run $CAND $H account list 2>&1 | q
A0=$(shasum -a 256 $H/auth.json | cut -c1-16); echo "auth.json (default = fake API key): $A0"
echo; echo "## OpenAI gpt-5.4"
x oa-real - - -m gpt-5.4 -c 'model_provider="openai"' --account real
x oa-fake openai-api-key OPENAI_API_KEY,CODEX_API_KEY -m gpt-5.4 -c 'model_provider="openai"' --account fake
x oa-default - - -m gpt-5.4 -c 'model_provider="openai"'
x oa-ghost openai-api-key OPENAI_API_KEY -m gpt-5.4 -c 'model_provider="openai"' --account ghost
echo "auth.json after: $(shasum -a 256 $H/auth.json | cut -c1-16)"
echo; echo "## Z.AI glm-5.3-flash"
x zai-real - - --account real
x zai-fake provider/zai_api_key ZAI_API_KEY --account fake
x zai-default provider/zai_api_key ZAI_API_KEY
echo; echo "## Kimi k3 (kimi-code)"
x kimi-alt - - -m kimi-k3 -c 'model_provider="kimi-code"' --account alt
x kimi-kfake provider/kimi_api_key KIMI_API_KEY -m kimi-k3 -c 'model_provider="kimi-code"' --account kfake
echo; echo "## Claude Plan"
x cp - - -m claude-opus-5-5-plan -c 'model_provider="claude-plan"' --account cp
x cpfake - - -m claude-opus-5-5-plan -c 'model_provider="claude-plan"' --account cpfake
echo; echo "## flag off: named OpenAI refused, default unchanged"
x oa-flagoff - - -m gpt-5.4 -c 'model_provider="openai"' --account real --disable named_accounts
} > "$OUT" 2>&1
echo done
