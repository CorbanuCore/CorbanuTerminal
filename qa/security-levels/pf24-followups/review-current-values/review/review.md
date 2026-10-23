**Verdict: request changes.** The new review screen can describe protection that isn't actually enforced. On a security display that has to be fixed before handoff.

## Findings

1. **High: the Network and Sandbox rows ignore commands that run outside the sandbox.** `tui/src/security/aggressive.rs:77-112`
   - **Problem:**
     - When the filesystem policy has no denied reads, a command can skip the sandbox in three ways: the model asks for escalation (`require_escalated`) and it's approved, the user has a permanent `Allow` rule, or a blocked command is approved and retried without the sandbox. This is in `core/src/tools/sandboxing.rs:242-284`.
     - Config C with `network_access = false` would show "Network now: off for agent commands", even though approved escalations get full network and full disk.
     - It's worse when the reviewer is `auto-review`, because the model approves those escalations itself.
     - The Aggressive line says "approved commands stay inside it too"; the "now" line never says the opposite.
   - **Fix:**
     - Compute `escapes = !file_system.has_denied_read_restrictions() && profile is not Disabled/External`.
     - When `escapes` is true, add "; approved or allow-listed commands can run outside the sandbox" to the Sandbox row.
     - Change the Network row to "off inside the sandbox; commands run outside it have network".
     - Only print "off" without that qualifier when escape is impossible (denied reads present, or approval `never` with no allow rules).

2. **High: the sandbox summary leaves out writable paths and can misname full access.** `aggressive.rs:77-81`
   - **Problem:**
     - `summarize_permission_profile` builds its list only from `workspace_roots`. It drops the profile's own `writable_roots`, so a named profile with `"/some/dir" = "write"` displays as `profile x: workspace-write [workdir]`.
     - A profile that is unrestricted on disk but has network off displays as `external-sandbox`.
     - Both read as more protection than is in force.
   - **Fix:** don't reuse the summary helper here. List the paths from `file_system.get_writable_roots_with_cwd(cwd)`. Show "full disk write" when `has_full_disk_write_access()` is true, and "full disk read" when `has_full_disk_read_access()` is true.

3. **Medium: the secret-variable probe can say "removed" when secrets are passed.** `aggressive.rs:163-181`
   - **Problem:** the probe tests fixed names only (`API_KEY`, `AWS_SECRET`, …). Any user pattern that names specific variables defeats it:
     - `include_only = ["PATH","HOME","OPENAI_API_KEY"]` shows "secret-like environment variables are removed", but `OPENAI_API_KEY` reaches commands.
     - `exclude = ["AWS_*"]` with the default `ignore_default_excludes = true` hides SECRET from the list, but `STRIPE_SECRET` still passes.
   - **Fix:** also run the real environment through `create_env(&policy, None)` and add every surviving name where `is_secret_name` is true (names only, never values). Only say "removed" when both the probes and the real environment come back empty.

4. **Medium: "spawned agents get this session's values" can be false.** `aggressive.rs:140-150`, `ROLE_KEYS` at `:183`
   - **Problem:** `ROLE_KEYS` leaves out `sandbox_workspace_write` (`writable_roots`, `network_access`), `profile`, and network-proxy keys (`features.network_proxy`, `network`). Under a legacy workspace-write session, a role that sets `sandbox_workspace_write.network_access = true` would widen its children while the row says they inherit this session's values.
   - **Fix:** for the display, count a role as "changes values" if its file sets any key outside a short safe list (`model`, `model_reasoning_effort`, `developer_instructions`, `description`, …). Also add `sandbox_workspace_write` to `ROLE_KEYS` so `verify` covers it.

5. **Medium: on a 24-row terminal the review is cut off and can't be scrolled.**
   - **Where:** `bottom_pane/security_level_picker.rs:223-240`, `security_view.rs:184-200`.
   - **Problem:**
     - With Config C at 80 columns, the snapshot body is about 30 lines plus the footer. Real paths and role lists make it longer.
     - `render` clips the body silently, so the Vault and Child agents rows, "Unchanged" and the restart note can all be hidden while Enter still saves.
     - The point of B-06 is that the user sees what they would lose; here they can't.
   - **Fix:**
     - Add a scroll offset (Up/Down or PgUp/PgDn on the review screen) and a "↓ more" marker.
     - Alternatively, keep confirm disabled until the user has scrolled to the end.
     - Add an 80×24 render test that checks the footer and the "more" marker.

6. **Low: "shell profiles are not loaded" goes further than the checks behind it.** `aggressive.rs:133`
   - **Problem:** the check only covers the shell snapshot and login shells. zsh always reads `~/.zshenv`, and bash reads `$BASH_ENV` if that variable is inherited. The Aggressive row makes the same claim.
   - **Fix:** say "login profiles and shell snapshots are not used" in both rows, or also check whether `BASH_ENV`/`ENV` survive `create_env`.

7. **Low: the vault check tests only the directory itself.** `aggressive.rs:114-116`
   - **Problem:** `can_read_path_with_cwd` takes the most specific matching entry, and it's asked only about `secrets` itself. A user profile with `secrets = deny` plus `secrets/<file> = read` would show "unreadable".
   - **Fix:** also report readable when any readable root returned by `get_readable_roots_with_cwd` lies under `codex_home/secrets`.

8. **Low: which settings stay current after `/permissions`, resume and fork.**
   - **Kept current:** `SessionConfigured` (`chatwidget/session_flow.rs:38-75`) and the `/permissions` setters (`chatwidget/settings.rs`) update approvals, the permission profile, the reviewer, cwd and workspace roots.
   - **Not refreshed:** `web_search_mode`, `shell_environment_policy`, features (`shell_snapshot`, request tools), `allow_login_shell` and `agent_roles`.
   - **Effect:** a thread resumed or forked with different overrides shows the widget's launch values for those rows.
   - **Fix:** say on screen that those rows reflect launch config, or carry those values through the session snapshot.

9. **Low: very narrow widths and doing work that isn't needed.**
   - **Narrow widths:** when the width is 4 or less, the 4-column indent still leaves at least one character per line (`max(1)`), so lines come out wider than the area and `Paragraph` cuts them. Drop the indents when `width < 20`.
   - **Work on every open:** `current_values` runs on every `/security` open, including role-file reads on the UI thread, even when the picker is disabled (`slash_dispatch.rs:368`, `security_view.rs:53-55`). Compute it only when `picker_enabled` is set.

10. **Low: test gaps and style.** `security/aggressive_tests.rs:187-249`
    - **Missing coverage:** nothing tests escalation (finding 1), named profiles with extra write paths (2), name-specific `include_only`/`exclude` (3), the "except custom roles" branch (4), or the 24-row height (5).
    - **Style:**
      - Line 224 asserts a tuple of bools. Compare the whole string instead, as `codex-rs/AGENTS.md` asks for whole-object comparisons.
      - `/tmp, $TMPDIR` is Unix-only, so that assertion needs a platform guard.
      - The hard-coded `current()` fixture in `security_level_picker_tests.rs` will drift from real output. Consider generating it from `current_values` on a built config.