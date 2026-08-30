# Provider self-invocation reliability fix

## Classification and authority

- Change class: bounded fix.
- Product specification: **Shipping MVP — LIVE**.
- Requirement excerpt: “Rust, Apache-2.0, Linux/macOS/Windows, the `corbanu`
  command” and “OpenAI, Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek,
  OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama, LM Studio,
  Corbanu Plan, and custom providers.”
- Restored behavior: an installed Corbanu process can invoke its own provider
  authentication and vault-helper entry points even when its installation
  directory was not present in the inherited `PATH`.
- Authorization boundary: unchanged. The helper still performs the existing
  vault and provider authorization checks; this fix changes only executable
  discovery.

## Incident

Launching Corbanu Terminal by absolute path from the macOS launcher did not add
the installation directory to `PATH`. Claude Plan model selection consequently
failed before authentication with:

```text
provider auth command `corbanu` failed to start: No such file or directory (os error 2)
```

The same lookup gap affected generated `corbanu vault auth-helper ...` commands.
During final qualification, Claude Code 2.1.92 also confirmed a current Max
login while storing its OAuth credential in the macOS Keychain rather than the
legacy credentials file Corbanu expected.

## Fix

- Resolve built-in Claude Plan provider authentication to the runtime's known
  absolute executable path, including configuration rebuilt by the embedded
  app server for a new thread.
- Add a process-lifetime `corbanu` alias to Corbanu's existing private arg0
  helper directory, which is prepended to child-process `PATH`. This covers
  generated Claude pane helpers and agent shell credential operations and is
  recreated on every launch/build.
- On macOS, read Claude Code's current keychain credential when its legacy
  `.credentials.json` file is absent. Credential values remain confined to the
  provider-auth subprocess and are never written to logs or model-visible
  context.

## Evidence

- Final commit: this record's containing fix commit; merge commit recorded in
  the GitHub `main` history.
- Automated tests:
  - `just test -p codex-cli -E 'test(/claude_oauth/)'`: 35 passed, including
    current macOS Keychain, refresh, concurrent refresh, failure redaction, and
    forced-refresh recovery cases.
  - `just test -p codex-arg0`: 10 passed.
  - focused `codex-core` runtime-executable provider-auth tests: 3 passed.
  - focused `codex-app-server` runtime-executable propagation test: 1 passed.
  - focused `codex-cli` process-arg0 doctor test: 5 passed across the CLI
    binaries.
  - `just bazel-lock-update`: passed; module lock remained current.
  - `cargo build --release --bin corbanu --bin codex-code-mode-host`: passed.
- True-TUI workflow: passed in tmux using the release binary, Claude Fable 5
  Plan, and deliberately restricted `PATH=/usr/bin:/bin`.
  - Initial request returned `PROVIDER_AUTH_OK`.
  - An in-flight request was cancelled with Escape.
  - A subsequent request in the same session returned `RECOVERY_OK`.
  - Trace configured provider auth with the absolute release executable and
    contained no executable-lookup or credential-location failure.
  - Artifacts:
    `/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/tmux-artifacts/provider-self-invocation-20260829/logs-keychain-final/`.
  - After the final review fixes and release rebuild, the same restricted-PATH
    workflow returned `FINAL_PROVIDER_AUTH_OK`; its trace recorded an attached
    authorization header and no executable-lookup or credential failure.
    Artifacts:
    `/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/tmux-artifacts/provider-self-invocation-20260829/logs-final-reviewed/`.
  - The final release binary's `doctor --summary` reported the installation as
    consistent, confirming that the private arg0 alias is not counted as a
    duplicate installation.
- Autoreview: Claude Opus 5 Plan at max effort in the Corbanu tmux harness.
  The first pass identified unsafe standalone app-server self-invocation; the
  second identified a false duplicate-install warning from the private arg0
  alias. Both findings were fixed and regression-tested. The final full-diff
  read-only review returned `CLEAN`.
- Human release-candidate sign-off: pending; required before publishing a
  release, not before merging this bounded repair.
