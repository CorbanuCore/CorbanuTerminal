# #425 positive: an account of another provider is used by a spawned agent on that provider.
# hZ-after: zai session on the DEFAULT Z.AI key (env), kimi-code:kmain holds the real Kimi key, and there is
# NO default Kimi key anywhere, so a kimi-code child that answers can only have used kmain.
. "$(dirname "$0")/live-env.sh"
H=$S/homes/hZ-after
export ZAI_API_KEY="$X"; KEEP=(ZAI_API_KEY)
P1='Call the spawn_agent tool exactly once with model "kimi-k3" (exactly that string) and message "Reply with exactly child-ok". Then wait for the agent and finally reply with exactly the text the child returned.'
P2='Call the spawn_agent tool exactly once with model_provider "kimi-code", model "k3", fork_turns "none" and message "Reply with exactly child-ok". Then wait for the agent and finally reply with exactly the text the child returned.'
show() { printf '%s\n' "$1" | grep -E -i 'child-ok|spawn|error|401|missing|session id' | grep -v -E 'Call the spawn_agent|^user$' | sort -u | head -12 | sed 's/^/   | /'; }
: skip-v1; rc=skipped
echo "== S1 (multi_agent v1 cannot move a zai child to another provider: spawn_agent refuses the model switch; see first run)"
out=$(cx $H exec --skip-git-repo-check --enable multi_agent_v2 --account kimi-code:kmain "$P2" 2>&1); rc=$?
echo "== S2 --account kimi-code:kmain, spawn_agent model_provider kimi-code fork_turns none (multi_agent_v2) (exit $rc)"; show "$out"
out=$(cx $H exec --skip-git-repo-check --enable multi_agent_v2 "$P2" 2>&1); rc=$?
echo "== S3 control: same spawn without --account (no default Kimi key exists) (exit $rc)"; show "$out"
echo "-- recorded accounts (rollouts in this home, newest first):"
for f in $(ls -t $(find $H/sessions -name 'rollout-*.jsonl') | head -5); do
  python3 - "$f" <<'PY'
import json,sys
p=sys.argv[1]; prov=acc=src=None
for line in open(p):
    try: d=json.loads(line)
    except Exception: continue
    t=d.get('type'); pl=d.get('payload') or {}
    if t=='session_meta': src=json.dumps(pl.get('source'))[:60]
    if t=='turn_context': prov=pl.get('model_provider'); acc=pl.get('provider_account')
print(f"   {p.rsplit('/',1)[1][:40]} source={src} model_provider={prov} provider_account={acc}")
PY
done
