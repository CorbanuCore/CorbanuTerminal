# 0.1.28 quarantine command ledger

This ledger makes the summarized results in `RETEST-RESULTS.md` reproducible without committing
raw model output or credentials. Variables used below:

```bash
BASE=/home/pfrpc/.pfterminal-debug/packages/standalone/releases/0.1.27-x86_64-unknown-linux-gnu/bin/pfterminal
CANDIDATE=/home/pfrpc/repos/worktrees/pfterminal-128-quarantine/codex-rs/target/release/pfterminal
CANDIDATE_DEBUG=/home/pfrpc/repos/worktrees/pfterminal-128-quarantine/codex-rs/target/release/pfterminal-debug
CANDIDATE_ACP=/home/pfrpc/repos/worktrees/pfterminal-128-quarantine/codex-rs/target/release/pfterminal-acp
```

Each model command was wrapped in `timeout 120` (the regression candidate was allowed 330 seconds),
combined stdout/stderr was captured in memory, elapsed time was measured with `date +%s%N`, and the
capture was hashed with `sha256sum`. No secret value was printed or persisted.

## Regression

```bash
timeout 120 "$BASE" exec --ephemeral --skip-git-repo-check -C /home/pfrpc \
  -c 'model_provider="vercel-anthropic"' -c 'model="claude-fable-5"' \
  'Reply with exactly: OK-BASE-VERCEL-ANTHROPIC' </dev/null

timeout 330 "$CANDIDATE" exec --ephemeral --skip-git-repo-check -C /home/pfrpc \
  -c 'model_provider="vercel-anthropic"' -c 'model="claude-fable-5"' \
  'Reply with exactly: OK-CANDIDATE-VERCEL-ANTHROPIC' </dev/null
```

## Provider turns

```bash
timeout 120 "$CANDIDATE" exec --ephemeral --skip-git-repo-check -C /home/pfrpc \
  -c 'model_provider="anthropic"' -c 'model="claude-fable-5"' \
  'Reply with exactly: OK-ANTHROPIC' </dev/null

timeout 120 "$CANDIDATE_DEBUG" exec --ephemeral --skip-git-repo-check -C /home/pfrpc \
  -c 'model_provider="claude-plan"' -c 'model="claude-fable-5-plan"' \
  'Reply with exactly: OK-CLAUDE-PLAN' </dev/null

timeout 120 "$CANDIDATE" exec --ephemeral --skip-git-repo-check -C /home/pfrpc \
  -c 'model_provider="zai"' -c 'model="glm-5.2"' \
  'Reply with exactly: OK-ZAI' </dev/null

# First Kimi turn is deliberately persistent. Its printed session id is passed to resume.
timeout 120 "$CANDIDATE" exec --skip-git-repo-check -C /home/pfrpc \
  -c 'model_provider="kimi-code"' -c 'model="k3"' \
  'Reply with exactly: OK-KIMI-FIRST' </dev/null
timeout 120 "$CANDIDATE" exec resume --skip-git-repo-check \
  -c 'model_provider="kimi-code"' -c 'model="k3"' "$SESSION_ID" \
  'Reply with exactly: OK-KIMI-RESUME' </dev/null

timeout 120 "$CANDIDATE" exec --ephemeral --skip-git-repo-check -C /home/pfrpc \
  -c 'model_provider="openai"' -c 'model="gpt-5.6-terra"' \
  'Reply with exactly: OK-OPENAI' </dev/null

timeout 120 "$CANDIDATE" exec --ephemeral --skip-git-repo-check -C /home/pfrpc \
  -c 'model_provider="ambient"' -c 'model="z-ai/glm-5.2"' \
  'Reply with exactly: OK-AMBIENT' </dev/null

timeout 120 "$CANDIDATE" exec --ephemeral --skip-git-repo-check -C /home/pfrpc \
  -c 'model_provider="openrouter"' -c 'model="z-ai/glm-5.2"' \
  'Reply with exactly: OK-OPENROUTER' </dev/null
```

## Vanilla home

`pfterminal` pins its home from `PFTERMINAL_HOME`, not `CODEX_HOME`. The test created a `mktemp -d`
home, supplied the existing vault provider values only as process environment variables, and ran:

```bash
PFTERMINAL_HOME="$VANILLA" \
ANTHROPIC_API_KEY="$ANTHROPIC_KEY" AMBIENT_API_KEY="$AMBIENT_KEY" \
ZAI_API_KEY="$ZAI_KEY" KIMI_API_KEY="$KIMI_KEY" OPENROUTER_API_KEY="$OPENROUTER_KEY" \
timeout 120 "$CANDIDATE" exec --ephemeral --ignore-user-config --skip-git-repo-check \
  -C /home/pfrpc 'Reply with exactly: VANILLA-OK' </dev/null
```

## Non-provider surfaces

```bash
"$CANDIDATE" tasknode status --json
PFTERMINAL_HOME="$FRESH_HOME" "$CANDIDATE" tasknode status --json

"$CANDIDATE_ACP" --version
CODEX_ACP_PATH=/bin/true "$CANDIDATE_ACP"

"$CANDIDATE_DEBUG" telegram --health

python3 scripts/install/test_pfterminal_release_contract.py
python3 scripts/install/test_install_sh.py

just test -p codex-model-provider-info
just test -p codex-tasknode-session
just test -p codex-tasknode-session legacy_pending_only_record_migrates
just test --release -p codex-tasknode-session
just test -p codex-telegram
just test -p codex-core load_config_k3_with_explicit_incompatible_provider_repairs_pair
just test -p codex-cli --test pfterminal_acp
```

The candidate worktree is detached at the quarantine commit. These commands do not authorize
installing it, merging it, tagging it, publishing it, or pointing Latest at it.
