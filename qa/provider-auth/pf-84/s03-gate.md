# PF-84-S03 gate evidence (account selection per session, CLI and worker)

Candidate: branch `feat/pf-84-s03-account-selection`, base
`0fa45b54f0ca65a6ce3485c27281624cb6f8b2cb` (merged with main `3683dca19d`), code commit `047d9e1039`. Behind the
default-off `named_accounts` feature. macOS arm64 debug build; every run used disposable
homes and `CORBANU_TEST_NO_NATIVE_KEYRING=1`.

## What the slice delivers

- Precedence: `--account` (exec, TUI, `corbanu resume`) or the spawn `account` argument >
  the account recorded in the thread (resume) > `[provider_accounts]` > `default`.
  `--account <name>` selects an account of the session's provider; `<provider>:<name>`
  names another provider. With the feature off a named `--account` is an error (it would
  otherwise run on the default account); `--account default` is always accepted.
- Every thread start, resume and spawned agent checks vault metadata (no secret decrypted)
  and refuses a missing named account with recovery text; nothing falls back to another
  account.
- Turn contexts record `provider_account` (name only, only while the feature is on); resume
  restores it as `<provider>:<name>` when the thread resumes on the recorded provider and no
  `--account` is given; `--account` is stored qualified with the provider it was validated
  against (app-server
  `thread/start` and `thread/resume` take an experimental `providerAccount`).
- Spawn `account` (D3): only configured accounts of the child's provider (the refusal lists
  them); the result shows the child's `account`; under Aggressive (the level in force, an
  unreadable policy counts as Aggressive) the switch needs the human's approval
  (`ask_human`), and is refused when approvals are off. Children without `account` inherit
  the parent's, including role children (explorer, Nazgul, Troll, Orc) and app-server
  `thread/spawnAgent` workers. The argument is offered only when the feature is on and is
  stripped from OpenAI's reserved collaboration schema.
- TUI worker preflight now checks the worker's own account and runs `auth.command` with that
  account (`validate_provider_auth_command_for_account`), fixing the S02 known limit.

## Checks

| Check | Result |
| --- | --- |
| `just test -p codex-core -E 'test(/provider_accounts\|multi_agents\|thread_manager\|schema\|turn_context\|rollout_reconstruction\|history\|spec_plan\|role/)'` | 497 passed |
| `just test -p codex-exec -p codex-cli -p codex-app-server-protocol -p codex-model-provider -p codex-protocol` | 2360 passed |
| `just test -p codex-app-server -E 'test(/thread_processor\|thread_resume\|thread_start\|thread_spawn\|schema\|feedback/)'` | 157 passed |
| `just test -p codex-tui -E 'test(/spawn_orchestration\|cli\|app_server_session/)'` | 117 passed |
| `just test -p codex-core --features codex-core/developer-accounting -E 'test(/provider_accounts\|multi_agents\|thread_manager\|schema/)'` | 194 passed (at `74da7b0c5e`) |
| macOS `cargo clippy --tests` on core, app-server, exec, tui, cli, model-provider, app-server-protocol, protocol | no warnings |
| Linux clippy `-D warnings` | RTX box (`ambient@100.99.88.49`) unreachable over Tailscale on 2026-10-10 (ssh timed out); relying on the PR's Ubuntu clippy job |
| tmux + GLM 5.3 Flash, exec workers (`pf84-exec-account`) | coordinator on the real default Z.AI key runs workers through the checked-in launcher: `--account main` (real key) `pong`; `--account fake` 401; resuming that session without `--account` 401 (recorded `fake`); resume `--account main` `pong`; `--account gone` refused as not configured |
| Second provider (`pf84-exec-account-kimi`) | same five results on `kimi-code` |
| In-process spawn (`pf84-spawn-account`) | GLM spawns with `account = "fake"`: result `{"account":"fake"}`, child errored `401 Unauthorized`, no retry; the coordinator then replies `pong` on its default account |
| TUI flag (`pf84-tui-account`) | `corbanu --account fake`: first prompt shows the 401 and the credential-rejected notice |
| Videos | [exec](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-exec-account-047d9e10392d-2026-10-10.mp4), [exec Kimi](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-exec-account-kimi-047d9e10392d-2026-10-10.mp4), [spawn](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-spawn-account-047d9e10392d-2026-10-10.mp4), [TUI](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-tui-account-047d9e10392d-2026-10-10.mp4); leak scan passed; run directories holding real keys were deleted after recording |

## Independent review (Opus 5.5 High, read-only `corbanu exec`)

Pass 1 (`74da7b0c5e`): changes requested. Fixed in `047d9e1039`:
1. Role children lost the parent's account on config reload: the account now carries over
   when the child stays on the parent's provider (test with `agent_type = "explorer"`).
2. D3 read the configured level: it now uses the level in force (`max` with the effective
   policy; unreadable counts as Aggressive).
3. A prefix-less `--account` is stored as `<provider>:<name>` of the provider it was
   validated against, so a worker or resume on another provider cannot reinterpret it.
4. `thread/spawnAgent` workers inherit the parent's live account (`ThreadConfigSnapshot`).
Also fixed: resume reads only the newest turn context and only for the recorded provider (5);
running-thread resume reports a `providerAccount` mismatch (6); `--account` errors name the
flag (9); tests for inheritance, role children and a non-Aggressive switch without a
prompt (10). Accepted with reasons: v1 `resume_agent` resumes a closed child on the parent's
account (7, does not widen access); prefix-less `--account` on resume is qualified with the
config's provider (8, documented; use `<provider>:<name>`).

Pass 2 (scoped follow-up on `047d9e1039`, integrator-authorized extension): **approve**, no
blocking findings. Its note 6 (a prefix-less `providerAccount` on a running-thread resume
reported a false mismatch) is fixed by normalizing the request first. Follow-ups recorded,
not blocking: tests for a mid-session level raise under D3, `thread/spawnAgent` inheritance
and the running-thread mismatch; a recorded provider removed from config fails the resume
with "unknown provider" (fail closed); the error relabel matches the prefix text.

## Known limits

- The TUI's spawn cell and `/status` do not render the account yet; the spawn result and the
  Aggressive approval prompt show it (`/status` is PF-84-S05, account rows PF-84-S04).
- A nested `corbanu exec --account` started by an agent's shell command is approved as that
  command (approvals are always on under Aggressive); it is not a separate account prompt.
- `thread/fork` does not carry `providerAccount`; a fork uses `[provider_accounts]`.
- Workers do not see the coordinator's environment key (shell policy strips it), so the
  demos use a named `main` account for the real-key worker path; the coordinator itself
  runs on the real default key.

## Open

- Independent code-blind functional design and execution: acceptance step, not the implementer.
