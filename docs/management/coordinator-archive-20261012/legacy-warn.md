# Worker brief: warn users who run the legacy (unelevated) Windows sandbox

You are an Opus 5.5 High worker. Work autonomously; do not ask questions. Travis approved this.

## Context
- Issue https://github.com/CorbanuCore/CorbanuTerminal/issues/158 and PR #163: the legacy (unelevated) Windows sandbox
  backend in `codex-rs/windows-sandbox-rs` lets a sandboxed command delete or move files outside its writable roots
  wherever the parent folder grants the user FILE_DELETE_CHILD (most of the user profile), in both read-only and
  workspace-write modes. #163 closes the protected-path (.git/.codex/.agents) route only; the rest is a documented
  known limitation. The elevated backend is not affected.
- Travis's decision: ship #163 as a partial fix and **warn users who run the legacy backend**.

## Do
1. Find how the Windows sandbox backend is chosen (config keys / features, e.g. elevated vs unelevated/legacy
   setup, `windows_sandbox` settings, onboarding/setup flow in the TUI, `codex sandbox` CLI) and every surface where
   a user ends up on the legacy backend.
2. Add a clear, non-alarming but explicit warning wherever a session will run commands under the legacy backend on
   Windows: TUI (startup/status notice and the sandbox/permissions setup screens, following existing warning
   patterns and styles), `corbanu exec` stderr, and the `codex sandbox` CLI. Say what's at risk (sandboxed commands
   can delete or move files outside the workspace) and how to switch to the elevated backend (the exact setting /
   command). Don't block anything; keep it dismissible or once-per-session per existing conventions. Add a doc
   section (core/README.md or docs/) and link it from the warning text if the repo has a docs URL pattern.
3. Tests: unit tests for the decision (legacy on Windows → warn; elevated → no warn; non-Windows → no warn) and
   snapshot tests for TUI rendering per the repo's insta conventions. `just fmt`, clippy on touched crates.
4. Branch `codex/legacy-sandbox-warning-20261005` off `origin/main` (merge main again if #163 lands first). Open a PR
   referencing #158; get checks green. Don't merge.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't change sandbox behaviour itself.

## Final report (your last message)
Where warnings appear (with snapshot excerpts), how users switch, tests, PR URL and check status.
