I added the legacy-sandbox warning and opened **PR #165**: https://github.com/CorbanuCore/CorbanuTerminal/pull/165. CI is fully green after merging #163 (36 pass, 9 skipped), and the PR is mergeable. I haven't merged it. One gap: nobody has run the TUI by hand on a Windows machine yet, and the repo's policy requires that before release.

**When the warning shows:** only on Windows, when the backend is set to legacy (`RestrictedToken`), and the session is read-only or workspace-write. There's no warning for the elevated backend, a disabled Windows sandbox, Full Access, or other operating systems. It never blocks anything and appears once per session. The decision lives in `codex-sandboxing` (`legacy_windows_sandbox_warning`), and sandbox behaviour is unchanged.

**Where it appears:**
- **TUI startup:** a warning line in the chat history.
  ```
  ⚠ Corbanu Terminal is using the non-admin Windows sandbox. In this mode,
    sandboxed commands can delete or move files outside your workspace in folders
    you own, such as most of your user profile. Switch to the default sandbox
    (one-time Administrator setup) by running /setup-default-sandbox, or set
    `sandbox = "elevated"` under `[windows]` in config.toml. Learn more:
    https://github.com/CorbanuCore/CorbanuTerminal/blob/main/docs/sandbox.md#windows-non-admin-sandbox
  ```
- **TUI, after picking the non-admin sandbox during setup:** the same warning is added.
- **TUI setup and fallback screens:** the non-admin option now explains the risk.
  ```
  2. Use non-admin sandbox (higher risk if prompt injected)       Commands can delete or move files outside your
                                                                  workspace
  ```
  The fallback screen also adds a yellow line: "In the non-admin sandbox, commands can delete or move files outside your workspace in folders you own, such as most of your user profile."
- **`/permissions`:** the footer now names the delete/move risk and still points to `/setup-default-sandbox`.
- **`corbanu exec`:** a `warning:` line on stderr. In `--json` mode it also goes to stderr, so the JSON output is unchanged.
- **`corbanu sandbox` on Windows:** a `warning:` line on stderr before the command runs. Here it also fires when the sandbox setting is disabled, because this command runs every non-elevated setting on the legacy backend.
- **App-server clients (I added this beyond the brief):** sent as a config warning, next to the existing Linux bubblewrap warning.
- **Docs:** a new "Windows non-admin sandbox" section in `docs/sandbox.md`, which the warning links to.

**How users switch:** run `/setup-default-sandbox` in the TUI and approve the Administrator prompt; the choice is saved. Or set this in `config.toml`:
```toml
[windows]
sandbox = "elevated"
```
For a single run, pass `-c windows.sandbox=elevated`. If setup hasn't been done yet, the first sandboxed command asks for Administrator approval once.

**Tests:**
- **Decision unit tests:** 5 new tests, all passing:
  - legacy on Windows warns for read-only and workspace-write;
  - elevated, disabled and Full Access don't warn;
  - non-Windows hosts don't warn.
- **New TUI snapshots:** `legacy_windows_sandbox_startup_warning`, `windows_sandbox_enable_prompt_non_admin_risk` and `windows_sandbox_fallback_prompt_non_admin_risk`, all passing. The Windows-only `/permissions` test now also checks for the risk text.
- **Formatting and clippy:** `just fmt` and clippy on the touched crates are clean.
- **Full local run (tui, exec, cli, sandboxing on macOS):** 5,924 passed and 30 failed. None of the failures are in code this PR touches:
  - socket paths too long under this drive's temp path ("path must be shorter than SUN_LEN");
  - tmux onboarding tests timing out on the wallet daemon;
  - two seatbelt tests;
  - `default_command_popup_items_snapshot`, which already doesn't match current main.
- **Windows-only code:** the startup warning, the post-setup warning and the `corbanu sandbox` warning can't compile on macOS. The Native Windows MSVC build passed in CI.
- **CI flake:** one Windows credential canary failed on a network certificate check; it passed on rerun.

**Branch:** `codex/legacy-sandbox-warning-20261005`, cut from `origin/main`. #163 merged while my PR was open, so I merged main back in (no force-push).

The docs don't yet mention that #163 now protects `.git`, `.codex` and `.agents`. Policy only lets docs describe shipped behaviour, and #163 isn't in a release yet.