**Verdict: approve with fixes.**

The change fixes the reported launch-time hole. The probe reads rules the same way a session does: the same layers, disabled layers skipped, the ignore-rules flag, the requirements overlay, and highest decision wins. So a user "allow" rule can't hide the forbid. Two gaps remain. A session started later can still lose the rule, and the probe only checks bare program names. I only read the code. I didn't build it or run the tests here.

1. **Medium: rules are checked only at launch and on TUI config rebuilds, not when a thread starts** (`tui/src/security/launch.rs:129-133`)
   - **Problem:** every app-server `thread/start` reloads config and rules from disk (`app-server/src/request_processors/thread_processor.rs:1565`). The same is true for `/new`, resume, and a subagent whose config folders differ from its parent's. If a `.rules` file breaks after launch, that session still falls back to the requirements-only policy and only shows a `ConfigWarning`, while the TUI keeps showing Aggressive as active.
   - **How it can happen:** the human's own `!` commands, an editor, another process, or an approved unsandboxed command can break the file. Agent commands can't write it, because `codex_home` and project `.codex` are read-only in the profile.
   - **Fix:** best is to make core fail closed. Add a config key such as `exec_policy_parse_errors = "fatal"` and set it in `aggressive::base_overrides`, so every app-server thread and child gets it through `-c`. `ExecPolicyManager::load` would then return `Err` instead of falling back. A second option is to put the vault rule in requirements, since the fallback keeps `requirements().exec_policy`. The minimum is to run `verify_exec_policy` before every thread start/resume/fork the TUI sends, and record the remaining race as known.

2. **Medium: the probe only checks bare names** (`tui/src/security/aggressive.rs:369-376`)
   - **Problem:** a session matches with `resolve_host_executables: true`. Any loaded rules file, user or trusted project, can declare `host_executable(name = "corbanu", paths = ["/elsewhere"])`. Then `/usr/local/bin/corbanu vault auth-helper X` matches no rule, falls back to heuristics, and becomes promptable under `untrusted` instead of forbidden. The bare-name probe still passes.
   - **Fix:** fail if `policy.host_executables()` has a key in `VAULT_PROGRAMS`. Also probe `std::env::current_exe()` as an absolute path using `check_with_options(..., &MatchOptions { resolve_host_executables: true })`. Relative paths like `./corbanu` and wrappers like `env corbanu` already get past the rule today. That's not a regression from this change, but note it as a known limit of prefix rules.

3. **Low: test gaps** (`tui/src/security/aggressive_tests.rs:249`)
   - Add these cases:
     - A broken `.codex/rules/*.rules` in a **trusted** project layer → refused.
     - The same file in an **untrusted/disabled** project → not refused, matching session behaviour.
     - Requirements alone forbidding the vault command (with the user rule file absent) → `verify_exec_policy` passes.
     - A `host_executable` narrowing → refused, once finding 2 is fixed.
     - `verify_reloaded` / `rebuild_config_for_cwd` into a cwd with a broken project rules file.

4. **Low: misleading message for read errors** (`aggressive.rs:362-366`)
   - **Problem:** `ReadDir` and `ReadFile` errors make a session fail fatally ("failed to load rules"). They don't drop the rule. Refusing to launch is still correct, but the text "so the vault rule would be dropped" is wrong for them.
   - **Fix:** match on `ExecPolicyError::ParsePolicy` for that wording, and use a generic "rules cannot be read" for other errors.

5. **Low: two failures for one cause** (`launch.rs:130-133`, `aggressive.rs:310-318`)
   - **Problem:** a missing rule file, or `ignore_user_and_project_exec_policy_rules`, now reports both the old line and "loaded exec policy does not forbid …".
   - **Fix:** either skip the probe's "does not forbid" line when one of those two failures is already listed, or keep both and accept the extra line on purpose.

6. **Low (design): `legacy_core` re-export.** This is acceptable. `legacy_core` exists for startup/config paths, and the check already runs on a core `Config`. Keep it to these two items and add a TODO to move to an app-server RPC (for example, rules-load status on `config/read`) when Aggressive moves to RPCs. The direct `codex-execpolicy` dependency is fine and lighter than also re-exporting `Decision`.

7. **Nit: array element types** (`aggressive.rs:372-373`)
   - **Problem:** `program` is a `&&&str` there, mixed with `&str` literals in one array. That depends on coercion and is easy to misread.
   - **Fix:** use `VAULT_PROGRAMS.into_iter().filter(|program| { let command = [*program, "vault", …] … })`, or build a `Vec<String>` explicitly.

8. **Nit: test style** (`aggressive_tests.rs:260`, `launch_tests.rs` new test)
   - **Problem:** the `assert!(a && b && c)` checks go against the codex-rs preference for `assert_eq!` on whole values.
   - **Fix:** build the expected string from the temp path and the formatted parse error, or at least split into separate assertions so a failure is clear.

9. **Nit: async conversion.** This looks correct. Nothing that isn't `Send` is held across `.await`: `level::context()` returns `&'static`, and `finish` publishes its context only after verification. Every caller now awaits, at `lib.rs:1235`, `lib.rs:2084` and `config_persistence.rs:126`.