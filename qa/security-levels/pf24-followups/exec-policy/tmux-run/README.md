# tmux run on GLM 5.2 (`-m glm-5.2 -c model_provider="zai"`)

- **Candidate:** `18b8217d83`.
- **"Before" build:** main plus the unrelated review-screen follow-up, without this change.
- **Profile:** disposable, with `security_levels = true`, the workspace trusted, and Aggressive stored before launch.
- **Keys:** sent with `tmux send-keys`; text and Enter were sent separately.

| Step | Capture | Result |
| --- | --- | --- |
| A1 | `A1-status.txt` | `/status` shows `Aggressive active` and profile `corbanu-aggressive`. |
| A2 | `A2-vault-forbidden.txt` | GLM ran `corbanu vault list` and it was rejected by the vault rule. |
| B | `B-approval-1.txt`, `B-agent-write-rules.txt` | GLM tried to create `.codex/rules/broken.rules` and was approved, but the sandbox refused (`mkdir: .codex: Operation not permitted`). An agent can't break the rules mid-session. |
| C0 (before) | `C0-before-project-broken.txt`, `C0-before-vault.txt`, `C0-before-vault-approved.txt` | With a broken `<workspace>/.codex/rules/broken.rules`, the old build starts, shows only a "custom rules not applied" warning, and still reports `Aggressive active`. `corbanu vault list` now only prompts, and after approval the `corbanu` binary runs. |
| C (after) | `C-project-broken-refused.txt` | The same profile is refused at launch. The message names `broken.rules` and the line. |
| D | `D-host-executable-refused.txt` | `host_executable(name = "corbanu", …)` in `home/rules` makes launch refuse. |
| E | `E-clean-restart-active.txt` | With both files moved away, it restarts with `Aggressive active`. |
