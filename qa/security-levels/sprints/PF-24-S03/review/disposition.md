# PF-24-S03 review disposition

Reviewer: independent Opus 5.5 (`claude-opus-5-5-plan`, high effort), read-only
`corbanu exec` session over `b8e5f59f7a..ff45f9e03a`. Prompt: `prompt.md`; verdict
and findings: `review.md` (request changes). Fixes landed in `72735863b6`.

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | High | Other layers can widen `permissions.corbanu-aggressive` | Fixed. `verify` fails when any layer other than the launch session flags defines the profile, and when writable roots are not exactly the current folder. Test `another_layer_defining_the_profile_fails_verification`. |
| 2 | High | Vault rule checked by file contents, not the loaded exec policy | Partly fixed. `verify` fails when user/project rules are ignored. The Vault row now rests on the deny-read of the vault store and sign-in file (an approved `$(corbanu vault …)` command fails inside the sandbox: `tmux-run/` and the vault video); the prefix rule is described as refusing direct commands only. Loading the effective exec policy in `verify` needs `codex-execpolicy` as a TUI dependency (shared `Cargo.lock` edit): follow-up for the integration owner. |
| 3 | Medium | Custom roles can change child values | Fixed at start: `verify_roles` refuses roles whose file sets approval, sandbox, permissions, web search, environment, login-shell or snapshot/permission-tool keys, or that agent commands can write. Reapplying those fields in core is out of this TUI sprint. Test `role_that_changes_child_values_fails_verification`. |
| 4 | Medium | Config reloads skip verification | Fixed: `verify_reloaded` runs in `load_config_or_exit_with_fallback_cwd` (onboarding, resume/fork reloads) and `rebuild_config_for_cwd`. |
| 5 | Medium | Stored level can be downgraded unnoticed | Partly fixed: a deleted state file next to the rule file reads as invalid (Aggressive). The pending-restart window and MCP servers are disclosed; a MAC on the state file and "restart now" are new mechanisms, left for PF-24-S02. |
| 6 | Medium | "Every session" overclaims | Fixed copy: TUI sessions and their children; `corbanu exec` and IDE sessions named as not covered. |
| 7 | Low | Permission guards are not one choke point | Latent path closed: `update_feature_flags(GuardianApproval)` is refused while Aggressive. A single choke point in thread routing is outside this sprint's write scope. |
| 8 | Low | Row text overstates | Fixed: copy reworded; `request_permissions_tool` and `exec_permission_approvals` forced off and verified. |
| 9 | Low | cwd `/` or `$HOME` | Fixed: `/` is refused; `$HOME` gives a startup warning. |
| 10 | Low | `auth.json` readable; hooks undisclosed | Fixed: `auth.json` deny-read; hooks, MCP servers and apps listed as unchanged. |
| 11 | Low | Test gaps | Added: launch refusal, profile widening, roles, deleted state; config tests use `without_managed_config_for_tests`. Exec-policy load test waits on finding 2's dependency; guard and resume paths are covered by the tmux run (08, 13, 15). |
| 12 | Nit | Module-wide `dead_code`, wildcard arm, fsync, non-UTF-8 home | Fixed: allow scoped to the old inspector state, exhaustive match, parent-dir fsync on Unix, non-UTF-8 Corbanu home refused for Aggressive. |
