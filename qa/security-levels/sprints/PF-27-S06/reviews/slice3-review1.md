VERDICT: CHANGES REQUESTED

I reviewed the code only. I did not build or run anything. The routing works for shell and unified exec, but there are two gaps that could expose secrets, and the main file test can't tell protection apart from simple lack of access.

**High**

1. **On Windows, protection only covers files that exist at launch, and a replaced file can lose it.** `launch_contract.rs:433-447`, and the `*.sqlite*` glob at about line 422. Deny entries are only added for existing paths, and `deny_read_resolver.rs:34-55` turns the glob into a fixed list of files at launch time. That leaves three gaps:
   - A long-running ConPTY or unified-exec session started before `auth.json` or a wallet/vault file exists (for example, before login) can read the file once it appears.
   - SQLite files created after launch, such as new `-wal`/`-shm` files or a new `state_N.sqlite`, are not covered.
   - If the elevated backend protects a file with deny entries on the file itself, rewriting it by temp file plus rename (the usual way `auth.json` is refreshed) creates a new file. The new file takes CODEX_HOME's permissions and loses the deny.
   
   **Fix:** On Windows, deny the containing directories with inheritable entries instead of relying on per-file checks: `secrets/`, the wallet directory and the broker runtime directory. Then either move `auth.json` and the `*.sqlite*` files into a denied subdirectory, or deny-read the whole of CODEX_HOME except an explicit allowlist. Add a test that creates and renames `auth.json` while a sandboxed process is running and confirms it still cannot be read.

2. **`apply_patch` on Windows skips the elevated-backend check.** `tools/runtimes/apply_patch.rs:~104` calls `file_tool_permissions(..., attempt.sandbox != SandboxType::None, ...)`. Under the unelevated restricted-token backend, read denies don't work, yet patching gets the protected profile instead of being refused. In-process tools are correctly limited to macOS/Linux at `turn_context.rs:633`, so this is inconsistent. Patch hunks fail with error text, and patch moves and update context matching read the target file, so this is a read oracle on protected files. **Fix:** pass `sandboxed && (!cfg!(windows) || windows_sandbox_uses_elevated_backend(attempt.windows_sandbox_level, network_present))`, or refuse whenever `check_sandbox` would refuse. Add a Windows unit test for this.

**Medium**

3. **The file test has no check that CODEX_HOME is readable in the first place.** `launch_contract_windows_tests.rs:110`: `NOTES` is only printed, never asserted. The tempdir sits under the runner's profile, and the sandbox user may not be able to read it at all. In that case `VAULT/AUTH/CONFIG/SQLITE-DENIED` passes whether or not the deny entries work. **Fix:** `assert!(files.contains("NOTES-READ"))`. Also add the same probe with the unprotected `base` profile and assert that `AUTH-READ` appears.

4. **The memory probe doesn't test slice 1's process DACL under the sandbox.** Lines 140-169: a separate sandbox user usually can't open another user's process even without the DACL. The sandboxed `vm_read=denied` result proves isolation, not the DACL. The positive control (lines 131-135) runs unsandboxed only. **Fix:** also run the sandboxed probe against the unhardened target. Record what it shows: if it is also denied, say the sandbox user is the boundary. Otherwise assert `granted` there, so the hardened case actually exercises the DACL.

5. **Nothing tests the backend selection itself.** `sandboxing.rs:480-484` takes `proxy_enforced = network.is_some()`, which matches `manager.rs:513`. That's correct now, but no test checks that a `WindowsSandboxLevel::RestrictedToken` attempt with no network is refused through `SandboxAttempt`. Only `check_sandbox` is called directly, and with a hard-coded `true`. **Fix:** add a test through `env_for`/`protect_launch` that covers both levels and the case with a proxy.

**Low**

6. **The exec-server path always passes `proxy_enforced=false`** (`sandboxing.rs:582-587`) even though `managed_network` is present. The result fails closed (it is refused unless the configured level is Elevated), but if the remote side picks a different backend the decision can be wrong. **Fix:** derive the flag from `command.managed_network.is_some()`, or document that exec-server launches are always refused on Windows.

7. **Windows environment allowlist gaps** (`protocol/src/secretless_launch.rs`). The `_HOME`, `_DIR` and `PATH` suffixes let through `GH_CONFIG_DIR`, `CARGO_HOME` (which can hold `credentials.toml`) and `GNUPGHOME`-style variables. They do point to readable credential stores that aren't protected. `APPDATA`/`LOCALAPPDATA`/`USERPROFILE` are not allowed, which is safe but breaks tools. Also unprotected and readable by the sandbox user wherever profile reads are granted: `%APPDATA%\gcloud`, `%USERPROFILE%\.git-credentials`, `%USERPROFILE%\.config\gh`, `%APPDATA%\npm\etc\npmrc` / `.npmrc`, `.cargo\credentials.toml` and `%USERPROFILE%\.ssh`. **Fix:** add these to `PROTECTED_HOME_ENTRIES`, deny-read them as directories, and drop `*_CONFIG_DIR` from the suffix allowance.

8. **Credential Manager and DPAPI:** these are bound to the user, so the sandbox user can't decrypt the real user's blobs. That's acceptable, but it needs to be stated in the sprint record, along with a check that the elevated setup does not grant the sandbox group read access to the whole `%USERPROFILE%`.

**Confirmed fine**
- Shell, unified exec and ConPTY sessions go through `SandboxAttempt::protect_launch`.
- External agents and `unix_escalation` pass `false`, so they are refused.
- In-process file tools fail closed on Windows.
- `ZDOTDIR` is only set on Unix.
- The `USERPROFILE` fallback is correct.
- Windows path case and 8.3 short names are resolved by the ACL on the object, so they're not an issue. The exception is junctions created later inside a writable workspace that point at CODEX_HOME. Those are still covered as long as the denies sit on the target, which is another reason for directory-level denies.
