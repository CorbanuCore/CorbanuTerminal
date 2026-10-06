# Finding that changed the sandbox mechanism (2026-10-06, manual run 1)

Build: first implementation (legacy `SandboxPolicy::WorkspaceWrite`, cwd only,
no network, tmp excluded, plus `approval_policy = "untrusted"`), GLM 5.2 on zai,
isolated `CODEX_HOME`, keys sent through tmux.

1. Prompt: `Run exactly this one shell command and nothing else: echo hi > ../outside/x.txt`
2. Approval prompt shown; the human pressed `y`.
3. Result on screen:

```
✔ You approved codex to run echo hi > ../outside/x.txt this time
• Ran echo hi > ../outside/x.txt
  └ (no output)
• Done — exit code 0. The file ../outside/x.txt was written (outside the writable workspace root).
```

The rollout showed the turn's sandbox policy was the intended
`workspace-write` (no network, tmp excluded) and no escalation was requested.
Cause: under `untrusted`, `should_bypass_approval` treats the first approval as
covering an unsandboxed retry after a sandbox denial
(`core/src/tools/orchestrator.rs`, `core/src/tools/sandboxing.rs`).

Same probe with a permission profile that has a denied-read entry (manual run
1b): after approval the output was `/bin/bash: ../outside/x.txt: Operation not
permitted` and nothing was written. Aggressive therefore uses the
`corbanu-aggressive` profile; see `06-outside-tmp-network-denied.txt`.
