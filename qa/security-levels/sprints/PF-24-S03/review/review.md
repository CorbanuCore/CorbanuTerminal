**Verdict: request changes.**

The overall design is sound. Flag-off behaviour is unchanged. Resume, fork and pane restore carry the Aggressive permissions, and remote or daemon targets are refused. The problem is the launch check: it only spot-checks a few results, and other config sources can quietly undo several rows while the UI shows Aggressive as active. I didn't edit or run anything; this is from reading the code.

**Your three deviations**
1. **Apply at next start:** agree. Web search, the environment policy and the exec policy are fixed for the session. But the check has to run on every config reload, not just the first (finding 4).
2. **`corbanu-aggressive` profile with a deny-read entry:** agree, and it is necessary. `sandboxing.rs:250` and `:280` (`unsandboxed_execution_allowed` and `sandbox_permissions_preserving_denied_reads`) keep approved and escalated commands inside the sandbox. The profile itself can be widened by other config (finding 1).
3. **`/permissions` guards:** agree in principle. They only cover two entry points, though, rather than one choke point (finding 7).

**Checked and fine**
- With no state file, launch is a no-op. The context is installed with `picker_enabled=false`, and the `/status` line, the view and the `/permissions` guard behave as before.
- The Aggressive overrides put `approval_policy` and `default_permissions` in the session-flags layer. `resume_permission_settings_for_overrides` therefore returns `OverrideFromCurrentConfig` for resume, fork and pane restore, so an older Permissive thread's permissions are not restored.
- Because the overrides are non-empty, the implicit local daemon is never reused. An explicit remote target is refused.
- Built-in agent roles don't set any of the Aggressive keys.

**Findings**

1. **High: other config can widen the Aggressive profile** (`tui/src/security/aggressive.rs:58-100`, `:182-215`)
   - **Problem:** `merge_toml_values` (`config/src/merge.rs:94-118`) deep-merges tables. A user config, a `-p` profile or a trusted project's `.codex/config.toml` (`permissions` isn't on the project denylist) can add entries to `[permissions.corbanu-aggressive]`. Examples: `filesystem."/Users/x/.ssh"="write"`, `workspace_roots=[…]`, network `unix_sockets`. `verify` only probes `/tmp`, `$TMPDIR`, the parent folder and the state file, so it still passes and the UI shows Aggressive with extra writable roots.
   - **Fix:** in `verify`, fail if any layer other than the session-flags layer defines `permissions.corbanu-aggressive`. Also assert that the resolved writable roots are exactly the current folder (and no workspace roots).

2. **High: the vault row checks file contents, not the exec policy that actually loads** (`aggressive.rs:224`, `launch.rs:84-95`)
   - **Problem:** `load_exec_policy_with_warning` (`core/src/exec_policy.rs:618`) silently falls back to an empty policy (or just the managed-requirements policy) when any `.rules` file fails to parse. That includes a project `.codex/rules/*.rules` in a trusted repo. Also, `ignore_user_and_project_exec_policy_rules` skips `CODEX_HOME/rules` entirely. In both cases the `corbanu vault` forbid rule is gone while the row shows as enforced.
   - **Fix:** in `verify`, call the strict `load_exec_policy(&config.config_layer_stack)` and require `Forbidden` for each vault program (e.g. `["corbanu","vault","get"]`, plus one absolute-path form). Treat a load error as a failure.

3. **Medium: child agents with a custom role don't inherit every row** (`core/src/agent/role.rs:150-160`, `:378`; `multi_agents_common.rs:234`)
   - **Problem:** a role's config layer is inserted as the highest-precedence session-flags layer. After that, only approval policy, reviewer and permission profile are reapplied. A role file can still set `web_search`, `shell_environment_policy`, `allow_login_shell` or `features.shell_snapshot`. If the role file sits inside the writable workspace, the agent can edit it before spawning. This contradicts the "Child agents: spawned agents get the same values" row.
   - **Fix:** reapply those four fields in `apply_spawn_agent_runtime_overrides`. Alternatively, fail verification when any configured role's file sets them or lives under a writable root.

4. **Medium: config reloads skip verification** (`lib.rs:1630`, `lib.rs:1834`, `app/config_persistence.rs:111`)
   - **Problem:** three paths rebuild the config without calling `verify`: the reload after onboarding trust or login, the startup resume/fork reload, and the in-session `rebuild_config_for_cwd`. A newly trusted project layer can then bring in findings 1 or 2 unchecked. These reloads also drop the startup warnings added in `finish` (unreadable state, ignored flags).
   - **Fix:** route every Aggressive config load through one helper that runs `verify` and exits or refuses on failure. Keep the warnings in `LevelContext` and show them when the app starts.

5. **Medium: the stored level can be downgraded without anyone noticing** (`level.rs:61`, `:92-96`)
   - **Problem:** a missing state file reads as Permissive with no warning. And after you save Aggressive, the current session keeps running as Permissive (possibly full access) until restart. Its agent, or any MCP server (these run outside the sandbox), can delete the file or write a valid `level = "permissive"`.
   - **Fix:**
     - Treat "rules file present but state file missing" as Invalid.
     - After saving Aggressive, offer to restart now.
     - Detect writes that didn't come from the picker, for example a MAC keyed by a vault or keychain secret.
     - At minimum, add MCP servers to the disclosure as a known limitation.

6. **Medium: "every session" claims too much** (`security_level_picker.rs:~210`)
   - **Problem:** only the TUI's `run_main` applies the level. `corbanu exec`, `app-server` (IDE or daemon) and `mcp-server` ignore a stored Aggressive level.
   - **Fix:** either apply or refuse in those entry points, or change the wording to "every Corbanu Terminal TUI session".

7. **Low: the permission guards aren't a single choke point** (`config_persistence.rs:450`, `:1043`)
   - **Problem:** `update_feature_flags(GuardianApproval)` writes `approval_policy` and `sandbox_mode` to `config.toml`. When the write comes back as overridden, `sync_auto_review_runtime_state_from_effective_config` switches the live thread to `:workspace`, which drops the deny-read entry. The status line would still say Aggressive. No UI reaches this today because `guardian_approval` is Stable, so it's latent.
   - **Fix:** block permission-bearing `OverrideTurnContext` and `ThreadSettingsUpdate` sends in one place (`thread_routing`/`thread_settings`), and guard `update_feature_flags`.

8. **Low: some row text overstates what is enforced** (`aggressive.rs:36`, `:45`)
   - **Problem:**
     - The prefix rule doesn't match `./corbanu vault`, `corbanu -c k=v vault`, `env corbanu vault` or shell indirection. Approval and the vault deny-read entry are the controls that actually hold.
     - "Approved commands stay inside it" is false when `request_permissions_tool` or `exec_permission_approvals` is enabled.
   - **Fix:** reword the rows, and force both features off in the overrides (and check this in `verify`).

9. **Low: a cwd of `/` (or `$HOME`) passes verification** (`aggressive.rs:205`, `:260`)
   - **Problem:** probe paths under the current folder are skipped, so with cwd `/` nearly the whole disk is writable while the row says "current folder".
   - **Fix:** refuse, or warn, when the cwd is the filesystem root or `$HOME`.

10. **Low: undisclosed exposure**
    - **Problem:** `CODEX_HOME/auth.json` and other provider credential files stay readable. Hooks run outside the sandbox with the full environment. Neither is listed in `UNCHANGED`.
    - **Fix:** list them in `UNCHANGED`, or deny-read `auth.json`.

11. **Low: test gaps**
    - **Missing tests:**
      - `finish()` refusing to start when verification fails.
      - The loaded exec policy actually forbidding the vault commands.
      - A project or user layer widening the profile.
      - The `/permissions` and confirmation guards.
      - Resume and fork choosing `OverrideFromCurrentConfig` under Aggressive.
      - A child agent with a role inheriting every row.
    - **Isolation:** `aggressive_tests.rs:118` and `:176` build `ConfigBuilder` without `LoaderOverrides::without_managed_config_for_tests()`, so host managed config or MDM settings can leak into the tests.

12. **Nits**
    - `security/mod.rs:4`: the module-wide `#![allow(dead_code)]` now hides dead code in the new live modules. Scope it to the old inspector items.
    - `level.rs:144`: the wildcard `_` arm goes against the `codex-rs/AGENTS.md` rule to prefer exhaustive matches.
    - `write_atomically` doesn't fsync the parent directory after `persist`.
    - The CODEX_HOME profile key is built with `display()`, which is lossy for non-UTF-8 paths. Reject such paths when Aggressive is chosen.