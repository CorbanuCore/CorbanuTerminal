**Verdict: approve with fixes.** I found no correctness or security defects in this change. The fixes below are about test coverage and docs. I read the code but didn't build or run anything, because the sandbox is read-only.

**How each check came out**

1. **The move doesn't change what `corbanu` does.** I compared the old and new files line by line. `level.rs` changed only in visibility (`pub(crate)` became `pub`) and doc comments. `nested_launch` was split into `candidate_homes()`, `decide()` and `describe()`. The home order, the fail-closed account lookup, the error messages and "refuse wins" are all identical. Making these items public is safe: it creates no new trust boundary, since any code could already read or write these files. `CORBANU_TEST_ACCOUNT_HOME` is still honoured only in debug builds, and no Cargo profile turns debug assertions on for release.
2. **Ordering is right in all four binaries.** In each one, the check runs inside `arg0_dispatch_or_else`, after clap parsing and before any config, auth or runtime work. In `codex-app-server`, the only earlier step is `take_remote_control_disabled_env`, which reads an environment variable and nothing else. The `codex-linux-sandbox`, `apply_patch` and execve-wrapper aliases, and the argv[1] helpers, still dispatch before the check, so they keep working. They don't start an agent and they exist in `corbanu` too. `load_dotenv` also runs first, but it can only set variables that the agent command could set itself.
3. **No bypass through these binaries.** All four refuse in both `refuse` and `pass` mode. The shipped packages are `corbanu`, `pfterminal`, `codex`, the `-debug` aliases, `*-acp`, `*-walletd`, `codex-code-mode-host` and `codex-app-server`. The CLI variants share `cli/src/main.rs`, and `corbanu-acp` drives `corbanu app-server`, so both are already checked. `exec-server`, `thread-manager-sample` and the app-server test client aren't shipped.
4. **The tests are meaningful but have gaps** (findings 1–3).
5. **Build hygiene is fine.**
   - The crate is in the workspace members and workspace dependencies.
   - `BUILD.bazel` follows the same pattern as `security-policy`.
   - `MODULE.bazel.lock` needs no change, because there are no new third-party crates.
   - Every dependency of the new crate is used, with `libc` correctly limited to Unix.
   - `codex-tui` still uses `sha2`, `libc`, `tempfile`, `toml`, `serde` and `codex-utils-home-dir`, so cargo-shear has nothing to flag.
   - No TUI code refers to the items that are now private.
   - Bazel puts a crate's own binaries in its test data, so `cargo_bin("codex-tui")?` will resolve.

**Findings**

1. **Low–Medium: the integration tests pass without testing anything when run as root.**
   - **Where:** `exec/tests/suite/nested_launch.rs:34-37,81-83` and `tui/tests/suite/nested_launch.rs:34-37,71-73`.
   - **Problem:** as root, the tests `return Ok(())`, which is common in containers and remote Bazel execution. The refusal logic in `standalone_nested_refusal` (`security-level/src/nested.rs:66-82`) also has no unit test.
   - **Fix:** split out a pure function, `standalone_refusal_for(binary, kind, &origins)`, that `standalone_nested_refusal` calls. Unit-test every kind in both `Refuse` and `Pass` in `codex-security-level`; that works as root. Print a note when the integration tests skip.

2. **Low: `codex-app-server` and `codex-mcp-server` have no integration test.**
   - **Where:** `app-server/src/main.rs:79-90` and `mcp-server/src/main.rs:10-21`.
   - **Fix:** add the same refuse/pass test to each suite, checking for the "never allowed there" message and a non-zero exit before any output on stdout.

3. **Low: the "no origin" test is weak, and the new crate has no tests of its own.**
   - **Where:** `exec/tests/suite/nested_launch.rs:97-108`.
   - **Problem:** this test passes on any failure that doesn't print the refusal text. Separately, the moved logic is still tested only from `tui/src/security/nested_tests.rs` and `level_tests.rs`, so the crate's own Cargo/Bazel test target covers nothing.
   - **Fix:** assert the expected downstream failure, such as missing auth. Move the `decide`, `nested_origins`, `registry_*`, `load` and `save` tests into `codex-rs/security-level`.

4. **Nit (docs): one sentence now contradicts itself.**
   - **Where:** `qa/security-levels/pf24-followups/nested-launch/README.md:85-86`.
   - **Problem:** "Only `corbanu` builds with this change check. The standalone binaries check too…"
   - **Fix:** reword to: "From #219 `corbanu` checks; since [standalone-nested] the standalone `codex-exec`, `codex-tui`, `codex-app-server` and `codex-mcp-server` do too."

5. **Info: gaps that existed before this PR and don't block it.** Track them separately.
   - **`corbanu stdio-to-uds` is allowed nested.** At `cli/src/nested_security.rs:84` it maps to `None`, so an agent command can use it to reach a running app-server or daemon socket, which amounts to a host launch. Any socket client could do the same, so the real control is the sandbox's socket policy. Consider treating it as Host.
   - **Standalone binaries started by a person don't apply a stored Aggressive level.** Only the TUI's `LaunchPlan` applies it, so their agents' commands aren't sandboxed and the nested check never triggers. This should go in the standalone-nested README "Limits" section.
   - **Some binaries can't be covered.** `*-walletd` starts no agent and is passphrase-gated, which is acceptable. Upstream or older `codex` builds on PATH can't be covered by this change at all.