**Verdict: approve with fixes.** Most of the earlier findings are fixed, but one "now" line can still claim more protection than the config enforces (new finding 1). Fix it before handoff.

## Earlier findings

| # | Finding | Status | Notes |
|---|---|---|---|
| 1 | Network and Sandbox rows ignore commands that run outside the sandbox | Resolved | `escapes = restricted && !has_denied_read_restrictions()` matches core's `unsandboxed_execution_allowed`. When `escapes` is true, the Sandbox, Network and Vault rows all give the weaker wording. |
| 2 | Summary drops writable paths and can misname full access | Resolved | The row now lists paths from `get_writable_roots_with_cwd`, and handles full-disk write, no sandbox and external sandbox separately. Read access isn't shown, but the row makes no claim about reads, so nothing is overclaimed. |
| 3 | Secret-variable probe can say "removed" when secrets are passed | Resolved | The real environment now goes through `create_env_from_vars` too. `is_secret_name` uses the same seven name fragments, so every surviving name maps to one. |
| 4 | Child-agent row can wrongly say children inherit this session's values | Resolved | A role counts as changing values unless it sets only safe keys; unreadable files count too. `ROLE_KEYS` now includes `sandbox_workspace_write` and `profile`. |
| 5 | 24-row review is cut off and can't be scrolled | Mostly resolved | The review scrolls, and the footer says so. There is still no marker for "more below", and no test renders the full view at 80×24 (the test uses a fixed `body_height = 12`). |
| 6 | Shell-profile claim is too broad | Resolved | The wording is narrowed to "login profiles and shell snapshots", and `use_profile` is now checked. |
| 7 | Vault check tests only the directory | Resolved | The `starts_with(secrets)` check on readable roots covers files under `secrets/`. |
| 8 | Some rows keep launch values after `/permissions`, resume and fork | Partly resolved | This is documented only in the module comment (`current.rs:10-13`). Nothing on screen tells the user that web search, the environment, shell and role rows reflect launch settings. |
| 9 | Narrow widths; work done on every open | Resolved | Indents are dropped when the width is under 20. Current values are computed lazily, only when the picker is enabled. |
| 10 | Test gaps and style | Partly resolved | New tests cover escapes, extra write paths, named variables and roles. Remaining: `current_tests.rs:78` compares a slice of the Vault string instead of the whole string, and there are no tests for full-disk write or permission-request tools. |

## New findings

1. **Medium: the Network and Vault rows overclaim when permission-request tools are on.** `current.rs:116-122`, `current.rs:126-137`
   - **Problem:**
     - When denied reads exist, `escapes` is false, so Network says "off for agent commands" and Vault can say "unreadable".
     - The `RequestPermissionsTool` and `ExecPermissionApprovals` features still let the model ask for extra permissions, and `sandbox_permissions_preserving_denied_reads` blocks only escalation, not these requests.
     - Once approved (or self-approved under auto-review), those extra permissions turn on network inside the sandbox (`sandboxing/src/policy_transforms.rs:492-519`). They can also grant read access to a path more specific than the `secrets` deny entry.
     - The Sandbox row does say "permission-request tools are on", but the Network and Vault rows don't.
   - **Fix:**
     - Compute `widenable = request_tools && approval_policy != Never`.
     - When it's true, Network says "off unless an approved permission request turns it on", and Vault says "unreadable unless an approved permission request grants access".
     - Add a test using a profile with denied reads and the request tools enabled.

2. **Low: Up can appear to do nothing after the terminal is resized.** `security_level_picker.rs:117-119`
   - **Problem:** `scroll_for` limits only the value it returns; the stored `self.scroll` keeps its old value. If the pane grows taller, Up takes several presses before the view moves.
   - **Fix:** in the Up branch, first set `self.scroll` to `self.scroll.min(self.max_scroll.get())`, then subtract 1.

3. **Low: the on-screen indication that rows reflect launch settings is still missing.** (This is earlier finding 8.)
   - **Problem:** under the "loaded config" definition, these rows are accurate. But a resumed thread with different overrides can show web search "disabled" or secrets "removed" when the running thread differs.
   - **Fix:** add a short note under the review saying those rows reflect launch settings, or carry the values through `SessionConfigured`.