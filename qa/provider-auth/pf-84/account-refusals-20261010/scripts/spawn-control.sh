. "$(dirname "$0")/live-env.sh"
H=$S/homes/hZ-after
export ZAI_API_KEY="$X"; KEEP=(ZAI_API_KEY)
P2='Call the spawn_agent tool exactly once with model_provider "kimi-code", model "k3", fork_turns "none" and message "Reply with exactly child-ok". Then wait for the agent and finally reply with exactly the text the child returned, or the exact error it reported.'
cx $H exec --skip-git-repo-check --enable multi_agent_v2 "$P2" 2>&1 | grep -v -E '^\s*$' | grep -v corbanu_call_metrics | tail -8 | sed 's/^/   | /'
