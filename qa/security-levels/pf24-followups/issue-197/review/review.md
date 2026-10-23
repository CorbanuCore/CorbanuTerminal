**Verdict: approve with fixes.** The core change is correct. `strict_rules` affects the one place a session loads `.rules` files (`session/mod.rs:614`), and every new thread or child goes through it. When it is off, the old warning-and-fallback path runs unchanged, so Permissive behaves exactly as before. Finding 1 is a pre-existing bug, not something this branch introduced. But it gives the same result as #197 (the TUI shows Aggressive while new threads have no vault rule), and `strict_rules` can't detect it. Track it with this fix or right after.

I couldn't read issue #197 because `gh` had no network access, so this review works from the summary you gave. I didn't build or run any tests.

### How each path gets the setting
- **thread/start, resume, fork:** the app server rebuilds config from its `-c` overrides, which never change after launch. Aggressive's overrides sit above the user and project layers, so neither can turn the setting off.
- **Child agents:** a child either reuses the parent's already-loaded policy (`agent/control.rs:1048`, `codex_delegate.rs:130`) or copies the parent's config. A role file's settings are applied above the `-c` overrides (`role.rs:342,379`), so blocking `strict_rules` in role files is required and correctly done.
- **Guardian/reviewer sessions** get an empty policy. That was already true, and Aggressive forces the reviewer to be you, so they don't run.
- **Things that can still turn it off** (all already true for the other Aggressive settings):
  - `config` overrides sent with a thread/start request are applied after the `-c` overrides (`config_manager.rs:254-262`). The TUI never sends `strict_rules`, though.
  - On Windows, a `managed_config.toml` in the Corbanu home folder ranks above the `-c` overrides.
  - `/permissions` and `requirements.toml` can't reach this setting.

### Findings

1. **High (pre-existing, same issue class): choosing Permissive in `/security` deletes the vault rule file while the session is still Aggressive.**
   - `tui/src/security/level.rs:132,152` and `bottom_pane/security_level_picker.rs:269`.
   - `level::save(Permissive)` calls `sync_rules`, which deletes the rule file right away. The picker says the session "stays Aggressive until" restart.
   - Any `/new`, resume, fork or child that loads its own policy after that has no vault rule.
   - A second Permissive instance does the same thing at launch (`launch.rs:33`).
   - `strict_rules` doesn't help because a missing file is not a parse error.
   - **Fix:** let `save` write only the state file and leave rule syncing to launch, which `LaunchPlan::prepare` already does.
   - For the second-instance case: have an Aggressive process supply the vault rule from somewhere outside the rules folder. Or treat a missing managed rule file as fatal when the policy loads under Aggressive (for example, a "required rules file" setting checked next to `strict_rules`).

2. **Medium: when a `/new` fails, the user first sees a warning that says the opposite of what happened.**
   - `app-server/src/request_processors/thread_processor.rs:1565-1577`.
   - Before the session starts, thread/start sends "Error parsing rules; custom rules not applied." Under strict mode no thread runs at all, so this suggests the session continued without rules, which is exactly the #197 confusion.
   - **Fix:** skip this warning when `config.strict_rules` is on, because the thread/start error already carries the details. Or use a strict-specific summary such as "Rules file does not parse; new threads cannot start until it is fixed."

3. **Low: the failure message is hard to read and gives no next step.**
   - `core/src/session/mod.rs:616` and `tui/src/app/session_lifecycle.rs:706-768`.
   - The TUI shows a chain of prefixes: "Failed to start a fresh session… error creating thread: Fatal error: failed to load rules: failed to parse rules file …". This is followed by the raw multi-line parser error.
   - The previous thread has already been shut down and unsubscribed, so the screen stays on a dead thread. This was already true for any `/new` failure, but this change makes it a likely one.
   - Recovery does work: fix the file, then run `/new` again.
   - **Fix:**
     - In strict mode, build the message with `format_exec_policy_error_with_source(&err)` so it shows one line with a line number.
     - Add a hint: "fix or remove the file, then run /new or /resume."
     - Ideally, also say in the TUI that the previous thread has ended.

4. **Low: a child can reuse a parent policy that was loaded in lenient mode.**
   - `core/src/exec_policy.rs:180-202`.
   - `child_uses_parent_exec_policy` doesn't compare `strict_rules`. If the parent loaded leniently (it fell back) and the child's config is strict, the child reuses the fallback policy.
   - Aggressive can't hit this: the parent is always strict, and role files can't set the key. It is still a gap in the setting's contract.
   - **Fix:** add `&& (parent_config.strict_rules || !child_config.strict_rules)`. This doesn't change Permissive.

5. **Low: tests miss the Aggressive-specific paths.**
   - `tui/src/security/aggressive_tests.rs:295-331` checks config and `verify`, but nothing about "later threads". Rename it, for example `strict_rules_is_forced_and_verified`.
   - No test confirms that a role file setting `strict_rules` is refused. Add one next to the existing `loose` role case (around line 241).
   - `app-server/tests/suite/v2/thread_start.rs:183` enables the setting through `config.toml`, not the `-c` overrides Aggressive uses. Add a case that starts the test server with `-c strict_rules=true` and a user `config.toml` containing `strict_rules = false`. That proves the actual delivery path.

6. **Nit: an unrelated line changed in the generated schema.**
   - `core/config.schema.json:1726`. The `ModelProviderInfo` description change is regeneration catching up with a doc comment already on `origin/main` (`model-provider-info/src/lib.rs:866`).
   - It's harmless. Mention it in the commit message, or split it out so the diff only covers #197.