**Verdict: approve with fixes.**

I found no blocking security issue. The change adds only one `read` carve-out for the registry folder. Reads were already full-disk apart from the vault denies, so it widens nothing, and the rest of the Aggressive design is unchanged. In the home-folder workspace, entries can no longer be deleted on macOS or Linux. The fixes below are for edge cases, error messages and test quality. I reviewed the code only; I didn't build or run anything.

**What holds up**
- **macOS (Seatbelt):** the registry is excluded from writes both as the folder itself and as everything under it. That works even if the folder doesn't exist yet, so a command can't create it first.
- **Linux (bubblewrap):** an existing registry gets a read-only bind mount, which can't be removed and blocks new entries. The older Landlock-only mode refuses profiles like this one outright, so it fails closed.
- **Verification:** the new check and `base_overrides` use the same `origin_registry_dir()` result. Like the other rows, it checks the permission policy, not the generated sandbox rules.
- **Nested `corbanu exec`:** `prepare_nested_exec` → `base_overrides` adds the same entry and `verify_aggressive_config` checks it. If the account lookup returns nothing inside the sandbox, the entry and the check are both skipped consistently. The outer sandbox still protects the registry either way.
- **Non-UTF-8 registry path:** the entry is skipped. Verification then fails only when the workspace contains the registry, so this fails closed.

**Findings**

1. **`launch.rs:61-65`, Low–Medium.** If creating the registry fails, launch only logs a warning and continues. On Linux with the home folder as the workspace, bubblewrap then puts an empty read-only file at the first missing ancestor, such as `~/.local` or `~/.local/state`. Agent commands can't create anything under it, and a placeholder file appears in the user's real home during the command.
   - **Fix:** in `verify_sandbox`, when `registry.starts_with(cwd)` and `!registry.is_dir()`, add a failure ("registry missing"). Alternatively, return an error from `prepare` instead of `warn!`.

2. **`aggressive.rs:128-132`, Low.** If the account home path contains glob characters (`*?[`…), the `read` key is treated as a glob pattern. Only `deny` entries may use globs, so loading the config fails and Aggressive won't start in any folder, not only the home folder.
   - **Fix:** skip the entry when the path contains glob characters, as already done for non-UTF-8 paths, and let the verification check cover the in-workspace case.

3. **`aggressive.rs:349-357`, Low.** With a non-UTF-8 or glob path, the failure reads "registry … is writable", which doesn't tell the user the real cause.
   - **Fix:** when the entry was skipped, report "registry path cannot be expressed in the profile (non-UTF-8)".

4. **`aggressive_tests.rs:225-273`, Medium (test quality).**
   - (a) The test returns early and passes when `CORBANU_TEST_ACCOUNT_HOME` is unset, so a plain `cargo test` / nextest run reports a green test that checked nothing.
   - (b) It only checks the policy model. Nothing confirms that the rules handed to Seatbelt and bubblewrap actually make the registry read-only.
   - (c) Its working folder is `registry.parent()`, not the account home, which is the documented scenario.
   - (d) No test covers a missing registry, folder creation in `LaunchPlan::prepare`, `prepare_nested_exec` including the entry, or the non-UTF-8 skip.
   - **Fix:** pass the registry path into `base_overrides`/`verify` as a parameter so tests can use a temp directory without the environment variable. Then:
     - assert that `get_writable_roots_with_cwd(cwd)[0].read_only_subpaths` contains the registry;
     - use the account home as the working folder;
     - in `launch_tests::stored_aggressive_…`, assert the registry folder exists;
     - add a `prepare_nested_exec` case.

5. **`launch.rs:61`, Low (test isolation).** In debug builds without `CORBANU_TEST_ACCOUNT_HOME` (plain `cargo test`), `launch_tests` now creates `~/Library/Application Support/Corbanu/aggressive-homes` in the operator's real home. `just test` sets the variable, but this is still a new side effect on the real home. The fix in finding 4 removes it.

6. **QA README / `ROWS` "Sandbox" text, Nit (evidence/docs).**
   - All evidence comes from a macOS debug build that uses `CORBANU_TEST_ACCOUNT_HOME`. The README's Linux `mv .local x` claim and the Linux behaviour for an existing or missing registry have no recorded run. Add a bubblewrap run, or label Linux as unverified.
   - The "write only in the current folder" row text doesn't mention the read-only exceptions inside the folder (Corbanu home, registry). Consider one clause.

The known ancestor-rename limit (e.g. `mv Library …`) is accurately described in the README and is unchanged by this PR.